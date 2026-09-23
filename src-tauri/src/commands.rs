//! The bridge between the Svelte front end and the processing pipeline.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::image_io::{self, OutputFormat};
use crate::processing;

/// Shared flag the front end can raise to stop a running batch.
#[derive(Default)]
pub struct BatchControl {
    cancelled: Arc<AtomicBool>,
}

impl BatchControl {
    fn begin(&self) -> Arc<AtomicBool> {
        self.cancelled.store(false, Ordering::SeqCst);
        Arc::clone(&self.cancelled)
    }
}

/// A scan that has been imported but not yet developed.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedImage {
    pub path: String,
    pub name: String,
    pub bytes: u64,
}

/// Before/after thumbnails for one scan, generated from a single decode.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub path: String,
    pub original: String,
    pub developed: String,
    pub width: u32,
    pub height: u32,
}

/// Everything the info dialog shows about one file: its header, what the
/// filesystem records, and where it sits.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageMetadata {
    pub path: String,
    pub name: String,
    pub directory: String,
    pub bytes: u64,
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub color: String,
    pub orientation: Option<String>,
    /// Milliseconds since the epoch, so the front end can format them in the
    /// user's own locale. `None` where the filesystem does not record one.
    pub modified: Option<u64>,
    pub created: Option<u64>,
}

/// What to write, and where.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputSettings {
    pub directory: String,
    pub format: OutputFormat,
    pub quality: u8,
    pub overwrite: bool,
}

/// Emitted after each image in a batch, successful or not.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchProgress {
    pub path: String,
    pub name: String,
    pub completed: usize,
    pub total: usize,
    pub output: Option<String>,
    pub error: Option<String>,
}

/// The tally the status bar shows when a batch ends.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchReport {
    pub succeeded: usize,
    pub failed: usize,
    pub cancelled: bool,
    pub failures: Vec<BatchFailure>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchFailure {
    pub path: String,
    pub name: String,
    pub error: String,
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unnamed")
        .to_string()
}

/// Expands whatever the user dropped or picked into a flat list of scans.
#[tauri::command]
pub async fn import_paths(paths: Vec<String>) -> Result<Vec<ImportedImage>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let roots: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
        image_io::collect_images(&roots)
            .into_iter()
            .map(|path| ImportedImage {
                name: file_name(&path),
                bytes: std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0),
                path: path.to_string_lossy().into_owned(),
            })
            .collect()
    })
    .await
    .map_err(|e| format!("import failed: {e}"))
}

/// Decodes one scan and returns both the untouched and the developed preview.
///
/// The clipping points are measured on the full-resolution image and then
/// applied to the downscaled copy, so the preview shows the same correction
/// that developing the file would write.
#[tauri::command]
pub async fn build_preview(path: String, max_edge: u32) -> Result<Preview, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let source = PathBuf::from(&path);
        let full = image_io::load_rgb(&source)?;
        let (width, height) = (full.width(), full.height());

        let original = image_io::downscale(&full, max_edge);
        let mut developed = original.clone();

        // Only its statistics are needed from here on.
        let levels = processing::measure(&full);
        drop(full);

        processing::develop_with_levels(&mut developed, &levels);

        Ok(Preview {
            path,
            original: image_io::to_data_url(&original)?,
            developed: image_io::to_data_url(&developed)?,
            width,
            height,
        })
    })
    .await
    .map_err(|e| format!("preview failed: {e}"))?
}

/// Reads one scan's properties without decoding it.
///
/// The dimensions come from the header rather than from the preview, so this
/// answers for an image whose preview failed or has not arrived yet.
#[tauri::command]
pub async fn image_metadata(path: String) -> Result<ImageMetadata, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let source = PathBuf::from(&path);
        let probe = image_io::probe(&source)?;
        let stat = std::fs::metadata(&source).ok();

        Ok(ImageMetadata {
            name: file_name(&source),
            directory: source
                .parent()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default(),
            bytes: stat.as_ref().map(std::fs::Metadata::len).unwrap_or(0),
            format: probe.format,
            width: probe.width,
            height: probe.height,
            color: probe.color,
            orientation: probe.orientation,
            modified: stat.as_ref().and_then(|m| epoch_millis(m.modified().ok())),
            created: stat.as_ref().and_then(|m| epoch_millis(m.created().ok())),
            path,
        })
    })
    .await
    .map_err(|e| format!("metadata failed: {e}"))?
}

/// A `SystemTime` as milliseconds since the epoch, dropping anything the
/// clock puts before it.
fn epoch_millis(time: Option<std::time::SystemTime>) -> Option<u64> {
    time?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_millis() as u64)
}

/// Develops every path in `paths` and writes the results to the output folder.
///
/// Progress is reported per image over the `develop://progress` event so the
/// UI can update as results land rather than waiting for the whole batch.
#[tauri::command]
pub async fn develop_batch(
    app: AppHandle,
    control: State<'_, BatchControl>,
    paths: Vec<String>,
    settings: OutputSettings,
) -> Result<BatchReport, String> {
    if paths.is_empty() {
        return Err("No images to develop".into());
    }
    let output_dir = PathBuf::from(&settings.directory);
    if settings.directory.trim().is_empty() {
        return Err("No output folder selected".into());
    }
    std::fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Could not use the output folder: {e}"))?;

    let cancelled = control.begin();

    tauri::async_runtime::spawn_blocking(move || {
        let total = paths.len();
        let completed = std::sync::atomic::AtomicUsize::new(0);
        let failures = std::sync::Mutex::new(Vec::new());

        paths.par_iter().for_each(|path| {
            if cancelled.load(Ordering::SeqCst) {
                return;
            }

            let source = PathBuf::from(path);
            let name = file_name(&source);
            let result = develop_one(&source, &output_dir, &settings);

            // Count every attempt, so progress reaches `total` even when some
            // images fail.
            let done = completed.fetch_add(1, Ordering::SeqCst) + 1;

            let (output, error) = match result {
                Ok(written) => (Some(written.to_string_lossy().into_owned()), None),
                Err(message) => {
                    failures.lock().unwrap().push(BatchFailure {
                        path: path.clone(),
                        name: name.clone(),
                        error: message.clone(),
                    });
                    (None, Some(message))
                }
            };

            let _ = app.emit(
                "develop://progress",
                BatchProgress {
                    path: path.clone(),
                    name,
                    completed: done,
                    total,
                    output,
                    error,
                },
            );
        });

        let mut failures = failures.into_inner().unwrap();
        failures.sort_by(|a, b| a.name.cmp(&b.name));

        let attempted = completed.load(Ordering::SeqCst);
        let failed = failures.len();
        BatchReport {
            succeeded: attempted - failed,
            failed,
            cancelled: cancelled.load(Ordering::SeqCst),
            failures,
        }
    })
    .await
    .map_err(|e| format!("develop failed: {e}"))
}

fn develop_one(
    source: &Path,
    output_dir: &Path,
    settings: &OutputSettings,
) -> Result<PathBuf, String> {
    let mut img = image_io::load_rgb(source)?;
    processing::develop(&mut img);

    let destination = image_io::output_path(source, output_dir, settings.format, settings.overwrite);
    image_io::save_image(&img, &destination, settings.format, settings.quality)?;
    Ok(destination)
}

/// Asks a running batch to stop. Images already in flight still finish.
#[tauri::command]
pub fn cancel_batch(control: State<'_, BatchControl>) {
    control.cancelled.store(true, Ordering::SeqCst);
}

/// Files and folders named on the command line, so scans can be opened from
/// a file manager or a shell (`film-converter roll/*.jpg`).
#[tauri::command]
pub fn startup_paths() -> Vec<String> {
    std::env::args()
        .skip(1)
        .filter(|argument| !argument.starts_with('-'))
        .filter(|argument| Path::new(argument).exists())
        .collect()
}

/// A sensible default output folder, used until the user picks one.
#[tauri::command]
pub fn default_output_dir() -> String {
    let base = dirs_pictures().unwrap_or_else(std::env::temp_dir);
    base.join("Film Converter").to_string_lossy().into_owned()
}

fn dirs_pictures() -> Option<PathBuf> {
    // Avoids a `dirs` dependency for the one path we need.
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("USERPROFILE").map(|home| PathBuf::from(home).join("Pictures"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        let home = PathBuf::from(std::env::var_os("HOME")?);
        let pictures = home.join("Pictures");
        Some(if pictures.is_dir() { pictures } else { home })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scratch directory of this test's own, cleaned up at the end.
    fn scratch(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "fc-{label}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// A low-contrast, orange-biased frame, the way a masked negative scans.
    fn write_negative(path: &Path) {
        let img = image::RgbImage::from_fn(48, 32, |x, y| {
            image::Rgb([
                150 + (x as u8 % 40),
                110 + (y as u8 % 30),
                70 + ((x + y) as u8 % 20),
            ])
        });
        img.save(path).unwrap();
    }

    fn settings(dir: &Path, format: OutputFormat, overwrite: bool) -> OutputSettings {
        OutputSettings {
            directory: dir.to_string_lossy().into_owned(),
            format,
            quality: 95,
            overwrite,
        }
    }

    #[test]
    fn develops_a_scan_into_the_output_folder() {
        let dir = scratch("develop");
        let source = dir.join("roll01.png");
        write_negative(&source);

        let out = dir.join("out");
        let written = develop_one(&source, &out, &settings(&out, OutputFormat::Png, true))
            .expect("develop");

        assert_eq!(written.file_name().unwrap(), "roll01_positive.png");
        assert!(written.exists(), "the file was actually written");

        // Developing inverts and expands, so the result must span far more of
        // the range than the low-contrast source did.
        let result = image_io::load_rgb(&written).unwrap();
        let reds: Vec<u8> = result.pixels().map(|p| p.0[0]).collect();
        let spread = reds.iter().max().unwrap() - reds.iter().min().unwrap();
        assert!(spread > 200, "expected a full-range result, got a spread of {spread}");
    }

    #[test]
    fn creates_the_output_folder_when_it_is_missing() {
        let dir = scratch("mkdir");
        let source = dir.join("roll02.png");
        write_negative(&source);

        let nested = dir.join("a").join("b");
        assert!(!nested.exists());

        develop_one(&source, &nested, &settings(&nested, OutputFormat::Jpg, true))
            .expect("develop");
        assert!(nested.join("roll02_positive.jpg").exists());
    }

    #[test]
    fn a_second_run_does_not_clobber_the_first() {
        let dir = scratch("second-run");
        let source = dir.join("roll03.png");
        write_negative(&source);
        let out = dir.join("out");

        let first = develop_one(&source, &out, &settings(&out, OutputFormat::Png, false)).unwrap();
        let second = develop_one(&source, &out, &settings(&out, OutputFormat::Png, false)).unwrap();
        assert_ne!(first, second, "the second run wrote a numbered copy");
        assert!(first.exists() && second.exists());

        // With overwrite on, it goes back to the original name.
        let third = develop_one(&source, &out, &settings(&out, OutputFormat::Png, true)).unwrap();
        assert_eq!(third, first);
    }

    #[test]
    fn an_unreadable_file_reports_an_error_rather_than_panicking() {
        let dir = scratch("bad-file");
        let source = dir.join("truncated.png");
        std::fs::write(&source, b"not actually a png").unwrap();

        let out = dir.join("out");
        let error = develop_one(&source, &out, &settings(&out, OutputFormat::Png, true))
            .expect_err("a corrupt file must fail");
        assert!(!error.is_empty(), "the error should say something");
    }

    #[test]
    fn collect_images_expands_folders_and_skips_other_files() {
        let dir = scratch("collect");
        write_negative(&dir.join("a.png"));
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        write_negative(&dir.join("sub").join("b.png"));
        std::fs::write(dir.join("notes.txt"), b"ignored").unwrap();

        let found = image_io::collect_images(std::slice::from_ref(&dir));
        let names: Vec<String> = found
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["a.png", "b.png"], "sub-folders included, text skipped");

        // Naming a file directly and via its folder imports it once.
        let twice = image_io::collect_images(&[dir.clone(), dir.join("a.png")]);
        assert_eq!(twice.len(), 2, "no duplicate for the file named twice");
    }
}
