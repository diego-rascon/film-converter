//! Before/after previews of one scan, from a single decode.

use std::path::Path;

use serde::Serialize;

use crate::error::Result;
use crate::{image_io, processing};

/// The untouched scan and its developed result, as JPEG `data:` URLs, with
/// the full-resolution dimensions of the file they came from.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub original: String,
    pub developed: String,
    pub width: u32,
    pub height: u32,
}

/// Decodes `path` and returns both previews, their longest edge `max_edge`.
///
/// The clipping points are measured on every pixel of the full-resolution
/// scan and only then applied to the downscaled copy, so the preview shows the
/// same correction that developing the file would write — measuring the copy
/// instead would pick different levels.
pub fn build(path: &Path, max_edge: u32) -> Result<Preview> {
    let full = image_io::load_rgb(path)?;
    let (width, height) = full.dimensions();
    let levels = processing::measure(&full);

    // Takes the full scan by value, so its memory goes as soon as the small
    // copy exists rather than at the end of the preview.
    let original = image_io::downscale(full, max_edge);
    let mut developed = original.clone();
    processing::develop_with_levels(&mut developed, &levels);

    Ok(Preview {
        original: image_io::jpeg_data_url(&original)?,
        developed: image_io::jpeg_data_url(&developed)?,
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbImage;

    #[test]
    fn reports_the_scans_own_size_and_both_pictures() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("frame.png");
        RgbImage::from_fn(300, 200, |x, y| {
            image::Rgb([150 + (x % 60) as u8, 110 + (y % 50) as u8, 80])
        })
        .save(&path)
        .unwrap();

        let preview = build(&path, 64).expect("preview");
        assert_eq!(
            (preview.width, preview.height),
            (300, 200),
            "full resolution, not the preview's"
        );
        assert!(preview.original.starts_with("data:image/jpeg;base64,"));
        assert!(preview.developed.starts_with("data:image/jpeg;base64,"));
        assert_ne!(
            preview.original, preview.developed,
            "the developed side was developed"
        );
    }

    #[test]
    fn an_unreadable_file_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("broken.jpg");
        std::fs::write(&path, b"not a jpeg").unwrap();
        assert!(build(&path, 64).is_err());
    }
}
