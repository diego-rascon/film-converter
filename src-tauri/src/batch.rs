//! A develop run: every requested scan developed and written, in parallel,
//! with progress reported as each one lands.

use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::discovery::display_name;
use crate::error::{Error, Result};
use crate::image_io::{self, OutputFormat};
use crate::processing;

/// Shared flag the front end can raise to stop a running batch.
#[derive(Default)]
pub struct BatchControl {
    cancelled: Arc<AtomicBool>,
}

impl BatchControl {
    /// Lowers the flag for a new run and hands the run its own handle on it.
    pub fn begin(&self) -> Arc<AtomicBool> {
        self.cancelled.store(false, Ordering::Relaxed);
        Arc::clone(&self.cancelled)
    }

    /// Asks the running batch to stop. Images already in flight still finish.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

/// What to write, and where.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputSettings {
    pub directory: PathBuf,
    pub format: OutputFormat,
    pub quality: u8,
    pub overwrite: bool,
}

/// Sent after each image in a run, whether it was written or failed.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchProgress {
    pub path: String,
    /// Images attempted so far in this run, this one included.
    pub completed: usize,
    pub output: Option<String>,
    pub error: Option<String>,
}

/// How a run ended.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchReport {
    pub succeeded: usize,
    pub cancelled: bool,
    /// Ordered by name, for the notice panel to list.
    pub failures: Vec<BatchFailure>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchFailure {
    pub path: String,
    pub name: String,
    pub error: String,
}

/// Develops every scan in `sources` into `settings.directory`, calling
/// `report` as each one is written or fails.
///
/// Scans run in parallel across every core. Raising `cancelled` stops the run
/// from starting any more; the ones already in flight finish.
pub fn run(
    sources: &[PathBuf],
    settings: &OutputSettings,
    cancelled: &AtomicBool,
    report: impl Fn(BatchProgress) + Sync,
) -> Result<BatchReport> {
    if sources.is_empty() {
        return Err(Error::NothingToDevelop);
    }
    if settings.directory.as_os_str().is_empty() {
        return Err(Error::NoOutputFolder);
    }
    std::fs::create_dir_all(&settings.directory).map_err(Error::OutputFolder)?;

    let destinations = plan_outputs(sources, settings);
    let completed = AtomicUsize::new(0);

    let mut failures: Vec<BatchFailure> = sources
        .par_iter()
        .zip(&destinations)
        .filter_map(|(source, destination)| {
            if cancelled.load(Ordering::Relaxed) {
                return None;
            }

            let result = develop_one(source, destination, settings);
            // Count every attempt, so progress reaches the total even when
            // some images fail.
            let completed = completed.fetch_add(1, Ordering::Relaxed) + 1;
            let path = source.to_string_lossy().into_owned();

            match result {
                Ok(()) => {
                    report(BatchProgress {
                        path,
                        completed,
                        output: Some(destination.to_string_lossy().into_owned()),
                        error: None,
                    });
                    None
                }
                Err(error) => {
                    let error = error.to_string();
                    report(BatchProgress {
                        path: path.clone(),
                        completed,
                        output: None,
                        error: Some(error.clone()),
                    });
                    Some(BatchFailure {
                        path,
                        name: display_name(source),
                        error,
                    })
                }
            }
        })
        .collect();

    failures.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(BatchReport {
        succeeded: completed.into_inner() - failures.len(),
        cancelled: cancelled.load(Ordering::Relaxed),
        failures,
    })
}

fn develop_one(source: &Path, destination: &Path, settings: &OutputSettings) -> Result<()> {
    let mut scan = image_io::load_rgb(source)?;
    processing::develop(&mut scan);
    image_io::save_image(
        &scan,
        destination,
        settings.format,
        settings.quality,
        settings.overwrite,
    )
}

/// Gives every scan its output file before any is written, following the
/// Python's `{name}_positive.{ext}` convention.
///
/// Planning up front is what keeps two scans that share a name — frame 01 of
/// two rolls — from being written to one file while the run is in flight: the
/// second gets a numbered name whether or not overwriting is on. Without
/// overwrite a name already on disk is passed over as well, so an earlier
/// run's result is kept rather than clobbered.
fn plan_outputs(sources: &[PathBuf], settings: &OutputSettings) -> Vec<PathBuf> {
    let extension = settings.format.extension();
    let mut claimed = HashSet::new();

    sources
        .iter()
        .map(|source| {
            let stem = source.file_stem().unwrap_or(OsStr::new("image"));
            let mut n = 1;
            let destination = loop {
                let candidate = settings.directory.join(output_name(stem, n, extension));
                if !claimed.contains(&candidate) && (settings.overwrite || !candidate.exists()) {
                    break candidate;
                }
                n += 1;
            };
            claimed.insert(destination.clone());
            destination
        })
        .collect()
}

/// `roll01_positive.jpg` for the first, then `roll01_positive (2).jpg`.
fn output_name(stem: &OsStr, n: u32, extension: &str) -> OsString {
    let mut name = stem.to_os_string();
    name.push("_positive");
    if n > 1 {
        name.push(format!(" ({n})"));
    }
    name.push(".");
    name.push(extension);
    name
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    /// A low-contrast, orange-biased frame, the way a masked negative scans.
    fn write_negative(path: &Path) {
        image::RgbImage::from_fn(48, 32, |x, y| {
            image::Rgb([
                150 + (x % 40) as u8,
                110 + (y % 30) as u8,
                70 + ((x + y) % 20) as u8,
            ])
        })
        .save(path)
        .unwrap();
    }

    fn settings(directory: &Path, format: OutputFormat, overwrite: bool) -> OutputSettings {
        OutputSettings {
            directory: directory.to_path_buf(),
            format,
            quality: 95,
            overwrite,
        }
    }

    fn names(paths: &[PathBuf]) -> Vec<String> {
        paths.iter().map(|path| display_name(path)).collect()
    }

    fn develop(
        sources: &[PathBuf],
        settings: &OutputSettings,
    ) -> (BatchReport, Vec<BatchProgress>) {
        let progress = Mutex::new(Vec::new());
        let report = run(sources, settings, &AtomicBool::new(false), |update| {
            progress.lock().unwrap().push(update);
        })
        .expect("the run starts");
        (report, progress.into_inner().unwrap())
    }

    #[test]
    fn output_names_follow_the_python_convention() {
        let dir = tempfile::tempdir().unwrap();
        let planned = plan_outputs(
            &[PathBuf::from("/in/roll01.tif")],
            &settings(dir.path(), OutputFormat::Jpg, true),
        );
        assert_eq!(planned, [dir.path().join("roll01_positive.jpg")]);
    }

    #[test]
    fn scans_that_share_a_name_get_a_file_each() {
        // Frame 01 of two different rolls, in one run.
        let sources = [
            PathBuf::from("/roll-a/01.tif"),
            PathBuf::from("/roll-b/01.tif"),
        ];
        for overwrite in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let planned = plan_outputs(
                &sources,
                &settings(dir.path(), OutputFormat::Png, overwrite),
            );
            assert_eq!(
                names(&planned),
                ["01_positive.png", "01_positive (2).png"],
                "overwrite {overwrite}"
            );
        }
    }

    #[test]
    fn without_overwrite_an_earlier_result_is_kept() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("roll_positive.jpg"), b"x").unwrap();
        let source = [PathBuf::from("/in/roll.tif")];

        let keep = plan_outputs(&source, &settings(dir.path(), OutputFormat::Jpg, false));
        assert_eq!(names(&keep), ["roll_positive (2).jpg"]);

        let replace = plan_outputs(&source, &settings(dir.path(), OutputFormat::Jpg, true));
        assert_eq!(names(&replace), ["roll_positive.jpg"]);
    }

    #[test]
    fn develops_every_scan_into_a_folder_it_creates() {
        let dir = tempfile::tempdir().unwrap();
        let sources = [dir.path().join("roll01.png"), dir.path().join("roll02.png")];
        for source in &sources {
            write_negative(source);
        }
        let out = dir.path().join("a").join("b");

        let (report, progress) = develop(&sources, &settings(&out, OutputFormat::Png, false));
        assert_eq!(report.succeeded, 2);
        assert!(report.failures.is_empty() && !report.cancelled);

        let mut counts: Vec<usize> = progress.iter().map(|p| p.completed).collect();
        counts.sort_unstable();
        assert_eq!(
            counts,
            [1, 2],
            "each image reported once, counting up to the total"
        );

        // Developing inverts and expands, so the result must span far more of
        // the range than the low-contrast source did.
        let result = image_io::load_rgb(&out.join("roll01_positive.png")).unwrap();
        let reds: Vec<u8> = result.pixels().map(|p| p.0[0]).collect();
        let spread = reds.iter().max().unwrap() - reds.iter().min().unwrap();
        assert!(
            spread > 200,
            "expected a full-range result, got a spread of {spread}"
        );
    }

    #[test]
    fn a_second_run_does_not_clobber_the_first() {
        let dir = tempfile::tempdir().unwrap();
        let source = [dir.path().join("roll03.png")];
        write_negative(&source[0]);
        let out = dir.path().join("out");

        let first = develop(&source, &settings(&out, OutputFormat::Jpg, false)).1;
        let second = develop(&source, &settings(&out, OutputFormat::Jpg, false)).1;
        assert_ne!(
            first[0].output, second[0].output,
            "the second run wrote a numbered copy"
        );
        assert!(out.join("roll03_positive.jpg").exists());
        assert!(out.join("roll03_positive (2).jpg").exists());

        // With overwrite on, it goes back to the original name.
        let third = develop(&source, &settings(&out, OutputFormat::Jpg, true)).1;
        assert_eq!(third[0].output, first[0].output);
    }

    #[test]
    fn an_unreadable_scan_fails_alone() {
        let dir = tempfile::tempdir().unwrap();
        let good = dir.path().join("good.png");
        let bad = dir.path().join("truncated.png");
        write_negative(&good);
        std::fs::write(&bad, b"not actually a png").unwrap();

        let (report, progress) = develop(
            &[bad, good],
            &settings(dir.path(), OutputFormat::Png, false),
        );
        assert_eq!(report.succeeded, 1);
        assert_eq!(report.failures.len(), 1);
        assert_eq!(report.failures[0].name, "truncated.png");
        assert!(
            !report.failures[0].error.is_empty(),
            "the error should say something"
        );
        assert_eq!(progress.len(), 2, "the failure is reported as progress too");
    }

    #[test]
    fn a_cancelled_run_starts_nothing_more() {
        let dir = tempfile::tempdir().unwrap();
        let source = [dir.path().join("roll.png")];
        write_negative(&source[0]);

        let report = run(
            &source,
            &settings(dir.path(), OutputFormat::Png, false),
            &AtomicBool::new(true),
            |_| panic!("nothing should be developed"),
        )
        .unwrap();
        assert!(report.cancelled);
        assert_eq!(report.succeeded, 0);
    }

    #[test]
    fn refuses_a_run_with_nothing_to_do_or_nowhere_to_put_it() {
        let dir = tempfile::tempdir().unwrap();
        let ignore = |_| {};
        let not_cancelled = AtomicBool::new(false);

        let nothing = run(
            &[],
            &settings(dir.path(), OutputFormat::Png, false),
            &not_cancelled,
            ignore,
        );
        assert!(matches!(nothing, Err(Error::NothingToDevelop)));

        let nowhere = run(
            &[dir.path().join("a.png")],
            &settings(Path::new(""), OutputFormat::Png, false),
            &not_cancelled,
            ignore,
        );
        assert!(matches!(nowhere, Err(Error::NoOutputFolder)));
    }
}
