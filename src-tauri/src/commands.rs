//! The commands the front end invokes. Each is a thin, typed doorway: it moves
//! the work off the async runtime and hands it to the module that does it, so
//! nothing here decodes, walks or writes anything itself.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};

use crate::batch::{self, BatchControl, BatchProgress, BatchReport, OutputSettings};
use crate::discovery::{self, Found, display_name};
use crate::error::Result;
use crate::image_io;
use crate::preview::{self, Preview};

/// Runs blocking work — decoding, walking folders, writing files — on the
/// blocking pool, so a large scan never stalls the async runtime.
async fn blocking<T>(work: impl FnOnce() -> Result<T> + Send + 'static) -> Result<T>
where
    T: Send + 'static,
{
    tauri::async_runtime::spawn_blocking(work).await?
}

/// A scan that has been imported but not yet developed.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedImage {
    pub path: String,
    pub name: String,
    pub bytes: u64,
}

impl From<Found> for ImportedImage {
    fn from(found: Found) -> Self {
        Self {
            name: display_name(&found.path),
            path: found.path.to_string_lossy().into_owned(),
            bytes: found.bytes,
        }
    }
}

/// Everything the properties dialog shows about one file: its header, what
/// the filesystem records, and where it sits.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageMetadata {
    pub directory: String,
    pub bytes: u64,
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub color: String,
    pub orientation: Option<&'static str>,
    /// Milliseconds since the epoch, so the front end can format them in the
    /// user's own locale. `None` where the filesystem does not record one.
    pub modified: Option<u64>,
    pub created: Option<u64>,
}

impl ImageMetadata {
    /// Reads the header rather than decoding the pixels, so this answers for
    /// an image whose preview failed or has not arrived yet.
    fn read(path: &Path) -> Result<Self> {
        let probe = image_io::probe(path)?;
        let stat = std::fs::metadata(path).ok();

        Ok(Self {
            directory: path
                .parent()
                .map(|parent| parent.to_string_lossy().into_owned())
                .unwrap_or_default(),
            bytes: stat.as_ref().map_or(0, std::fs::Metadata::len),
            format: probe.format,
            width: probe.width,
            height: probe.height,
            color: probe.color,
            orientation: probe.orientation,
            modified: stat.as_ref().and_then(|m| epoch_millis(m.modified())),
            created: stat.as_ref().and_then(|m| epoch_millis(m.created())),
        })
    }
}

/// A timestamp as milliseconds since the epoch, dropping one the filesystem
/// does not keep or the clock puts before 1970.
fn epoch_millis(time: std::io::Result<SystemTime>) -> Option<u64> {
    let since_epoch = time.ok()?.duration_since(UNIX_EPOCH).ok()?;
    u64::try_from(since_epoch.as_millis()).ok()
}

/// Expands whatever the user dropped or picked into a flat list of scans.
#[tauri::command]
pub async fn import_paths(paths: Vec<PathBuf>) -> Result<Vec<ImportedImage>> {
    blocking(move || {
        Ok(discovery::collect_images(&paths)
            .into_iter()
            .map(ImportedImage::from)
            .collect())
    })
    .await
}

/// The extensions `import_paths` takes, for the file dialog's filter.
#[tauri::command]
pub fn supported_extensions() -> &'static [&'static str] {
    discovery::SUPPORTED_EXTENSIONS
}

/// Decodes one scan and returns both the untouched and the developed preview.
#[tauri::command]
pub async fn build_preview(path: PathBuf, max_edge: u32) -> Result<Preview> {
    blocking(move || preview::build(&path, max_edge)).await
}

/// Reads one scan's properties without decoding it.
#[tauri::command]
pub async fn image_metadata(path: PathBuf) -> Result<ImageMetadata> {
    blocking(move || ImageMetadata::read(&path)).await
}

/// Develops every path in `paths` and writes the results to the output folder.
///
/// Progress arrives over `progress` per image, so the UI can update as results
/// land rather than waiting for the whole batch; the report comes back once
/// the run is over.
#[tauri::command]
pub async fn develop_batch(
    control: State<'_, BatchControl>,
    paths: Vec<PathBuf>,
    settings: OutputSettings,
    progress: Channel<BatchProgress>,
) -> Result<BatchReport> {
    let cancelled = control.begin();
    blocking(move || {
        batch::run(&paths, &settings, &cancelled, |update| {
            // A window closed mid-run has nobody left to tell; the run itself
            // still finishes what it started.
            let _ = progress.send(update);
        })
    })
    .await
}

/// Asks a running batch to stop. Images already in flight still finish.
#[tauri::command]
pub fn cancel_batch(control: State<'_, BatchControl>) {
    control.cancel();
}

/// Files and folders named on the command line, so scans can be opened from
/// a file manager or a shell (`film-converter roll/*.jpg`). An argument that
/// is not valid Unicode cannot cross to the front end as text, so it is
/// skipped rather than mangled.
#[tauri::command]
pub fn startup_paths() -> Vec<String> {
    std::env::args_os()
        .skip(1)
        .filter_map(|argument| argument.into_string().ok())
        .filter(|argument| !argument.starts_with('-') && Path::new(argument).exists())
        .collect()
}

/// Where the output dialog opens until the user has chosen a folder once:
/// the system's pictures folder, as the desktop names it in the user's own
/// language.
#[tauri::command]
pub fn default_output_dir(app: AppHandle) -> String {
    let paths = app.path();
    paths
        .picture_dir()
        .or_else(|_| paths.home_dir())
        .unwrap_or_else(|_| std::env::temp_dir())
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_describes_the_file_it_read() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("frame.png");
        image::RgbImage::new(12, 7).save(&path).unwrap();

        let facts = ImageMetadata::read(&path).expect("metadata");
        assert_eq!(facts.format, "PNG");
        assert_eq!((facts.width, facts.height), (12, 7));
        assert_eq!(facts.directory, dir.path().to_string_lossy());
        assert_eq!(facts.bytes, std::fs::metadata(&path).unwrap().len());
        assert!(facts.modified.is_some(), "every filesystem here records it");
    }

    #[test]
    fn an_imported_image_is_listed_under_its_file_name() {
        let image = ImportedImage::from(Found {
            path: PathBuf::from("/scans/roll-01/frame 07.tif"),
            bytes: 42,
        });
        assert_eq!(image.name, "frame 07.tif");
        assert_eq!(image.path, "/scans/roll-01/frame 07.tif");
        assert_eq!(image.bytes, 42);
    }
}
