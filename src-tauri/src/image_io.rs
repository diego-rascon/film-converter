//! Loading scans from disk, and writing developed positives back out.

use std::fs::File;
use std::io::{BufWriter, Cursor};
use std::path::{Path, PathBuf};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use image::codecs::png::{CompressionType, FilterType as PngFilter, PngEncoder};
use image::codecs::tiff::TiffEncoder;
use image::metadata::Orientation;
use image::{
    DynamicImage, ExtendedColorType, ImageDecoder, ImageEncoder, ImageFormat, ImageReader, RgbImage,
};
use serde::{Deserialize, Serialize};

/// Extensions we offer to import. Matches the Python file dialog's filter,
/// plus the other still formats the decoder already supports.
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "tif", "tiff", "bmp", "webp",
];

pub fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            let e = e.to_ascii_lowercase();
            SUPPORTED_EXTENSIONS.contains(&e.as_str())
        })
        .unwrap_or(false)
}

/// Output container, mirroring the Python's `format_choice`.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Jpg,
    Png,
    Tiff,
}

impl OutputFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Jpg => "jpg",
            Self::Png => "png",
            Self::Tiff => "tiff",
        }
    }
}

/// Walks `roots`, expanding directories, and returns every supported image
/// found, de-duplicated and in a stable order.
pub fn collect_images(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for root in roots {
        if root.is_dir() {
            collect_from_dir(root, &mut found);
        } else if root.is_file() && is_supported(root) {
            found.push(root.clone());
        }
    }

    // The same file can be reached two ways -- picked directly and again via
    // an enclosing folder -- so identity is the canonical path, not the
    // spelling. `retain` keeps the first spelling of each file.
    let mut seen = std::collections::HashSet::new();
    found.retain(|p| seen.insert(p.canonicalize().unwrap_or_else(|_| p.clone())));
    found.sort();
    found
}

fn collect_from_dir(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    // Read the whole directory first so recursion happens in a stable order.
    let mut entries: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    entries.sort();

    for entry in entries {
        if entry.is_dir() {
            collect_from_dir(&entry, out);
        } else if is_supported(&entry) {
            out.push(entry);
        }
    }
}

/// Decodes `path` to RGB8, the same starting point as PIL's
/// `Image.open(path).convert("RGB")`.
///
/// Unlike the Python, EXIF orientation is honoured — negatives photographed
/// with a camera instead of a flatbed are routinely tagged sideways, and
/// rotation cannot affect the colour pipeline that follows.
pub fn load_rgb(path: &Path) -> Result<RgbImage, String> {
    let reader = ImageReader::open(path)
        .map_err(|e| format!("could not open: {e}"))?
        .with_guessed_format()
        .map_err(|e| format!("could not read: {e}"))?;

    let mut decoder = reader
        .into_decoder()
        .map_err(|e| format!("unsupported image: {e}"))?;
    let orientation = decoder.orientation().unwrap_or(image::metadata::Orientation::NoTransforms);

    let mut image =
        DynamicImage::from_decoder(decoder).map_err(|e| format!("could not decode: {e}"))?;
    image.apply_orientation(orientation);

    Ok(image.into_rgb8())
}

/// What a file's header says about itself, read without decoding the pixels.
pub struct Probe {
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub color: String,
    /// The EXIF rotation `load_rgb` will apply, described; `None` when the
    /// file is upright or carries no tag.
    pub orientation: Option<String>,
}

/// Reads the header of `path`: format, size, colour and orientation.
///
/// The dimensions are the ones the app works in, so a file tagged sideways is
/// reported the way `load_rgb` will hand it over rather than the way it is
/// stored.
pub fn probe(path: &Path) -> Result<Probe, String> {
    let reader = ImageReader::open(path)
        .map_err(|e| format!("could not open: {e}"))?
        .with_guessed_format()
        .map_err(|e| format!("could not read: {e}"))?;

    // `into_decoder` consumes the reader, so the format is taken first.
    let format = reader.format();
    let mut decoder = reader
        .into_decoder()
        .map_err(|e| format!("unsupported image: {e}"))?;

    let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);
    let (mut width, mut height) = decoder.dimensions();
    if matches!(
        orientation,
        Orientation::Rotate90
            | Orientation::Rotate270
            | Orientation::Rotate90FlipH
            | Orientation::Rotate270FlipH
    ) {
        std::mem::swap(&mut width, &mut height);
    }

    Ok(Probe {
        format: format.map_or_else(|| "Unknown".to_string(), describe_format),
        width,
        height,
        color: describe_color(decoder.original_color_type()),
        orientation: describe_orientation(orientation).map(str::to_string),
    })
}

fn describe_format(format: ImageFormat) -> String {
    match format {
        ImageFormat::Jpeg => "JPEG".into(),
        ImageFormat::Png => "PNG".into(),
        ImageFormat::Tiff => "TIFF".into(),
        ImageFormat::Bmp => "BMP".into(),
        ImageFormat::WebP => "WebP".into(),
        other => format!("{other:?}"),
    }
}

/// The shape a file manager states it in: colour model plus bits per *pixel*,
/// so a 16-bit-per-channel scan reads as the 48-bit file it is.
fn describe_color(color: ExtendedColorType) -> String {
    let model = match color {
        ExtendedColorType::A8 => "Alpha",
        ExtendedColorType::Cmyk8 | ExtendedColorType::Cmyk16 => "CMYK",
        _ => match color.channel_count() {
            1 => "Grayscale",
            2 => "Grayscale + alpha",
            3 => "RGB",
            4 => "RGB + alpha",
            _ => "Unknown",
        },
    };
    format!("{model}, {}-bit", color.bits_per_pixel())
}

fn describe_orientation(orientation: Orientation) -> Option<&'static str> {
    Some(match orientation {
        Orientation::NoTransforms => return None,
        Orientation::Rotate90 => "Rotated 90\u{b0} clockwise",
        Orientation::Rotate180 => "Rotated 180\u{b0}",
        Orientation::Rotate270 => "Rotated 90\u{b0} anticlockwise",
        Orientation::FlipHorizontal => "Mirrored horizontally",
        Orientation::FlipVertical => "Mirrored vertically",
        Orientation::Rotate90FlipH => "Rotated 90\u{b0} clockwise and mirrored",
        Orientation::Rotate270FlipH => "Rotated 90\u{b0} anticlockwise and mirrored",
    })
}

/// Scales `img` down so its longest edge is at most `max_edge`. Images already
/// that small are returned untouched.
pub fn downscale(img: &RgbImage, max_edge: u32) -> RgbImage {
    let (w, h) = (img.width(), img.height());
    let longest = w.max(h);
    if longest <= max_edge || max_edge == 0 {
        return img.clone();
    }

    let scale = max_edge as f64 / longest as f64;
    let target_w = ((w as f64 * scale).round() as u32).max(1);
    let target_h = ((h as f64 * scale).round() as u32).max(1);

    image::imageops::thumbnail(img, target_w, target_h)
}

/// Encodes a preview as a JPEG `data:` URL for the webview to display.
pub fn to_data_url(img: &RgbImage) -> Result<String, String> {
    let mut buffer = Vec::new();
    encode_jpeg(&mut Cursor::new(&mut buffer), img, PREVIEW_QUALITY)
        .map_err(|e| format!("could not encode preview: {e}"))?;
    Ok(format!("data:image/jpeg;base64,{}", BASE64.encode(&buffer)))
}

/// Previews are re-encoded on every load, so they trade a little fidelity for
/// a much smaller payload across the IPC bridge.
const PREVIEW_QUALITY: u8 = 88;

fn encode_jpeg<W: std::io::Write>(
    writer: W,
    img: &RgbImage,
    quality: u8,
) -> Result<(), String> {
    let mut encoder = jpeg_encoder::Encoder::new(writer, quality);
    // PIL's `subsampling=0`: keep full chroma resolution. Film grain and dye
    // clouds are chroma-heavy, and 4:2:0 smears them.
    encoder.set_sampling_factor(jpeg_encoder::SamplingFactor::R_4_4_4);
    encoder
        .encode(
            img.as_raw(),
            img.width() as u16,
            img.height() as u16,
            jpeg_encoder::ColorType::Rgb,
        )
        .map_err(|e| e.to_string())
}

/// Writes `img` to `path` in `format`, interpreting `quality` the way the
/// Python's `save_image` does.
pub fn save_image(
    img: &RgbImage,
    path: &Path,
    format: OutputFormat,
    quality: u8,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("could not create output folder: {e}"))?;
    }

    let file = File::create(path).map_err(|e| format!("could not create file: {e}"))?;
    let mut writer = BufWriter::new(file);

    match format {
        OutputFormat::Jpg => {
            if img.width() > u16::MAX as u32 || img.height() > u16::MAX as u32 {
                return Err(format!(
                    "image is {}x{}, larger than JPEG's 65535 pixel limit",
                    img.width(),
                    img.height()
                ));
            }
            encode_jpeg(&mut writer, img, quality)?;
        }
        OutputFormat::Png => {
            // PIL: `compress_level = int((100 - quality) / 10)`. PNG is
            // lossless, so this only trades file size against time.
            let level = ((100u32.saturating_sub(quality as u32)) / 10).min(9) as u8;
            let compression = if level == 0 {
                CompressionType::Uncompressed
            } else {
                CompressionType::Level(level)
            };
            PngEncoder::new_with_quality(&mut writer, compression, PngFilter::Adaptive)
                .write_image(
                    img.as_raw(),
                    img.width(),
                    img.height(),
                    image::ExtendedColorType::Rgb8,
                )
                .map_err(|e| format!("could not encode PNG: {e}"))?;
        }
        OutputFormat::Tiff => {
            // The Python passes `compression="none"`, which is also what the
            // TIFF encoder does by default.
            let file = writer
                .into_inner()
                .map_err(|e| format!("could not flush file: {e}"))?;
            TiffEncoder::new(file)
                .write_image(
                    img.as_raw(),
                    img.width(),
                    img.height(),
                    image::ExtendedColorType::Rgb8,
                )
                .map_err(|e| format!("could not encode TIFF: {e}"))?;
            return Ok(());
        }
    }

    use std::io::Write as _;
    writer
        .flush()
        .map_err(|e| format!("could not finish writing: {e}"))?;
    Ok(())
}

/// Builds the output path for a source file, following the Python's
/// `{name}_positive.{ext}` convention.
///
/// When `overwrite` is false, a numeric suffix is added rather than clobbering
/// a result from an earlier run.
pub fn output_path(
    source: &Path,
    output_dir: &Path,
    format: OutputFormat,
    overwrite: bool,
) -> PathBuf {
    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");
    let extension = format.extension();

    let candidate = output_dir.join(format!("{stem}_positive.{extension}"));
    if overwrite || !candidate.exists() {
        return candidate;
    }

    for n in 2..10_000 {
        let candidate = output_dir.join(format!("{stem}_positive ({n}).{extension}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    candidate
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_supported_extensions() {
        assert!(is_supported(Path::new("scan.JPG")));
        assert!(is_supported(Path::new("scan.tiff")));
        assert!(is_supported(Path::new("/a/b/scan.png")));
        assert!(!is_supported(Path::new("notes.txt")));
        assert!(!is_supported(Path::new("noextension")));
    }

    #[test]
    fn output_paths_follow_the_python_convention() {
        let out = output_path(
            Path::new("/in/roll01.tif"),
            Path::new("/out"),
            OutputFormat::Jpg,
            true,
        );
        assert_eq!(out, Path::new("/out/roll01_positive.jpg"));
    }

    #[test]
    fn downscale_preserves_aspect_and_bounds() {
        let img = RgbImage::new(4000, 2000);
        let small = downscale(&img, 640);
        assert_eq!(small.width(), 640);
        assert_eq!(small.height(), 320);

        // Already small enough: left alone.
        let tiny = RgbImage::new(100, 50);
        let same = downscale(&tiny, 640);
        assert_eq!((same.width(), same.height()), (100, 50));
    }

    #[test]
    fn png_compression_level_matches_pil_formula() {
        // PIL: int((100 - quality) / 10)
        for (quality, expected) in [(100u8, 0u8), (95, 0), (90, 1), (50, 5), (0, 9)] {
            let level = ((100u32.saturating_sub(quality as u32)) / 10).min(9) as u8;
            assert_eq!(level, expected, "quality {quality}");
        }
    }

    #[test]
    fn round_trips_every_output_format() {
        let dir = std::env::temp_dir().join(format!("fc-io-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        let img = RgbImage::from_fn(32, 16, |x, y| {
            image::Rgb([(x * 8) as u8, (y * 16) as u8, 90])
        });

        for (format, quality) in [
            (OutputFormat::Jpg, 95u8),
            (OutputFormat::Png, 95),
            (OutputFormat::Tiff, 95),
        ] {
            let path = dir.join(format!("out.{}", format.extension()));
            save_image(&img, &path, format, quality).expect("save");

            let read = load_rgb(&path).expect("reload");
            assert_eq!((read.width(), read.height()), (32, 16), "{format:?}");

            if format != OutputFormat::Jpg {
                assert_eq!(read.as_raw(), img.as_raw(), "{format:?} is lossless");
            }
        }

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn probe_reads_the_header_and_rejects_a_broken_file() {
        let dir = std::env::temp_dir().join(format!("fc-probe-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        let img = RgbImage::from_fn(24, 10, |x, _| image::Rgb([(x * 10) as u8, 40, 90]));
        let path = dir.join("scan.png");
        save_image(&img, &path, OutputFormat::Png, 95).unwrap();

        let facts = probe(&path).expect("probe");
        assert_eq!(facts.format, "PNG");
        assert_eq!((facts.width, facts.height), (24, 10));
        assert_eq!(facts.color, "RGB, 24-bit");
        assert!(facts.orientation.is_none(), "an untagged file is upright");

        let broken = dir.join("broken.png");
        std::fs::write(&broken, b"not actually a png").unwrap();
        assert!(probe(&broken).is_err(), "a corrupt file must fail, not panic");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn avoids_clobbering_when_overwrite_is_off() {
        let dir = std::env::temp_dir().join(format!("fc-clobber-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        let source = Path::new("/in/roll.tif");
        let first = output_path(source, &dir, OutputFormat::Jpg, false);
        assert_eq!(first.file_name().unwrap(), "roll_positive.jpg");

        std::fs::write(&first, b"x").unwrap();
        let second = output_path(source, &dir, OutputFormat::Jpg, false);
        assert_eq!(second.file_name().unwrap(), "roll_positive (2).jpg");

        // With overwrite on, the original name comes back.
        let overwritten = output_path(source, &dir, OutputFormat::Jpg, true);
        assert_eq!(overwritten, first);

        std::fs::remove_dir_all(&dir).ok();
    }
}
