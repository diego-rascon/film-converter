//! Reading scans from disk, and writing developed positives back out.

use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Cursor, Write};
use std::path::Path;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use image::codecs::png::{CompressionType, FilterType as PngFilter, PngEncoder};
use image::codecs::tiff::TiffEncoder;
use image::metadata::Orientation;
use image::{
    DynamicImage, ExtendedColorType, ImageDecoder, ImageEncoder, ImageFormat, ImageReader, Limits,
    RgbImage,
};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// The most memory one decode may claim. `image` refuses anything over
/// 512 MB by default, which a 16-bit scan of a single medium-format frame
/// can pass; this leaves room for a 4×5 sheet scanned at 5000 dpi, while a
/// corrupt header claiming more than any scanner writes still fails as a
/// decode error instead of exhausting memory.
const MAX_DECODE_BYTES: u64 = 4 << 30;

/// Previews are re-encoded on every load, so they trade a little fidelity for
/// a much smaller payload across the IPC bridge.
const PREVIEW_QUALITY: u8 = 88;

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

/// Opens `path` for decoding, its format read from the content rather than
/// trusted to the extension.
fn open(path: &Path) -> Result<ImageReader<BufReader<File>>> {
    let mut reader = ImageReader::open(path)
        .map_err(Error::Open)?
        .with_guessed_format()
        .map_err(Error::Read)?;

    let mut limits = Limits::default();
    limits.max_alloc = Some(MAX_DECODE_BYTES);
    reader.limits(limits);
    Ok(reader)
}

/// Decodes `path` to RGB8, the same starting point as PIL's
/// `Image.open(path).convert("RGB")`.
///
/// Unlike the Python, EXIF orientation is honoured — negatives photographed
/// with a camera instead of a flatbed are routinely tagged sideways, and
/// rotation cannot affect the colour pipeline that follows.
pub fn load_rgb(path: &Path) -> Result<RgbImage> {
    let mut decoder = open(path)?.into_decoder().map_err(Error::Unsupported)?;
    let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);

    let mut image = DynamicImage::from_decoder(decoder).map_err(Error::Decode)?;
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
    pub orientation: Option<&'static str>,
}

/// Reads the header of `path`: format, size, colour and orientation.
///
/// The dimensions are the ones the app works in, so a file tagged sideways is
/// reported the way `load_rgb` will hand it over rather than the way it is
/// stored.
pub fn probe(path: &Path) -> Result<Probe> {
    let reader = open(path)?;
    // `into_decoder` consumes the reader, so the format is taken first.
    let format = reader.format();
    let mut decoder = reader.into_decoder().map_err(Error::Unsupported)?;

    let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);
    let (width, height) = decoder.dimensions();
    let (width, height) = if turns_sideways(orientation) {
        (height, width)
    } else {
        (width, height)
    };

    Ok(Probe {
        format: format.map_or_else(|| "Unknown".to_string(), describe_format),
        width,
        height,
        color: describe_color(decoder.original_color_type()),
        orientation: describe_orientation(orientation),
    })
}

fn turns_sideways(orientation: Orientation) -> bool {
    matches!(
        orientation,
        Orientation::Rotate90
            | Orientation::Rotate270
            | Orientation::Rotate90FlipH
            | Orientation::Rotate270FlipH
    )
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

/// Scales `img` down so its longest edge is at most `max_edge`. An image
/// already that small is handed back as it is.
pub fn downscale(img: RgbImage, max_edge: u32) -> RgbImage {
    let (width, height) = img.dimensions();
    let longest = width.max(height);
    if max_edge == 0 || longest <= max_edge {
        return img;
    }

    let scale = f64::from(max_edge) / f64::from(longest);
    let target_width = ((f64::from(width) * scale).round() as u32).max(1);
    let target_height = ((f64::from(height) * scale).round() as u32).max(1);

    image::imageops::thumbnail(&img, target_width, target_height)
}

/// Encodes a preview as a JPEG `data:` URL for the webview to display.
pub fn jpeg_data_url(img: &RgbImage) -> Result<String> {
    let mut buffer = Vec::new();
    encode_jpeg(img, Cursor::new(&mut buffer), PREVIEW_QUALITY)?;
    Ok(format!("data:image/jpeg;base64,{}", BASE64.encode(&buffer)))
}

fn encode_jpeg(img: &RgbImage, writer: impl Write, quality: u8) -> Result<()> {
    let (Ok(width), Ok(height)) = (u16::try_from(img.width()), u16::try_from(img.height())) else {
        return Err(Error::TooLargeForJpeg {
            width: img.width(),
            height: img.height(),
        });
    };

    let mut encoder = jpeg_encoder::Encoder::new(writer, quality);
    // PIL's `subsampling=0`: keep full chroma resolution. Film grain and dye
    // clouds are chroma-heavy, and 4:2:0 smears them.
    encoder.set_sampling_factor(jpeg_encoder::SamplingFactor::R_4_4_4);
    encoder.encode(img.as_raw(), width, height, jpeg_encoder::ColorType::Rgb)?;
    Ok(())
}

/// PIL's `compress_level = int((100 - quality) / 10)`. PNG is lossless, so
/// this only trades file size against time.
fn png_compression(quality: u8) -> CompressionType {
    match (100u8.saturating_sub(quality) / 10).min(9) {
        0 => CompressionType::Uncompressed,
        level => CompressionType::Level(level),
    }
}

/// Writes `img` to `path` in `format`, interpreting `quality` the way the
/// Python's `save_image` does.
///
/// Without `overwrite` the file must not exist yet: a name that was free when
/// the run planned it but has been taken since fails here, rather than being
/// written over.
pub fn save_image(
    img: &RgbImage,
    path: &Path,
    format: OutputFormat,
    quality: u8,
    overwrite: bool,
) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true);
    if overwrite {
        options.create(true).truncate(true);
    } else {
        options.create_new(true);
    }
    let file = options.open(path).map_err(Error::CreateFile)?;
    let mut writer = BufWriter::new(file);

    let (width, height) = img.dimensions();
    match format {
        OutputFormat::Jpg => encode_jpeg(img, &mut writer, quality)?,
        OutputFormat::Png => {
            PngEncoder::new_with_quality(
                &mut writer,
                png_compression(quality),
                PngFilter::Adaptive,
            )
            .write_image(img.as_raw(), width, height, ExtendedColorType::Rgb8)
            .map_err(|source| Error::Encode {
                format: "PNG",
                source,
            })?;
        }
        // The Python passes `compression="none"`, which is also what the TIFF
        // encoder does by default. It seeks back to patch offsets as it goes,
        // which the buffered writer allows by flushing first.
        OutputFormat::Tiff => TiffEncoder::new(&mut writer)
            .write_image(img.as_raw(), width, height, ExtendedColorType::Rgb8)
            .map_err(|source| Error::Encode {
                format: "TIFF",
                source,
            })?,
    }

    writer.flush().map_err(Error::Write)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gradient(width: u32, height: u32) -> RgbImage {
        RgbImage::from_fn(width, height, |x, y| {
            image::Rgb([(x * 8) as u8, (y * 16) as u8, 90])
        })
    }

    #[test]
    fn downscale_preserves_aspect_and_bounds() {
        let small = downscale(RgbImage::new(4000, 2000), 640);
        assert_eq!(small.dimensions(), (640, 320));

        let tall = downscale(RgbImage::new(1000, 3000), 300);
        assert_eq!(tall.dimensions(), (100, 300));

        // Already small enough: handed back untouched.
        let tiny = gradient(100, 50);
        assert_eq!(downscale(tiny.clone(), 640).as_raw(), tiny.as_raw());
    }

    #[test]
    fn downscale_averages_rather_than_samples() {
        // Alternating black and white columns average to a mid grey; a
        // nearest-neighbour sample would keep only one of the two.
        let stripes = RgbImage::from_fn(64, 64, |x, _| {
            image::Rgb([if x % 2 == 0 { 0 } else { 255 }; 3])
        });
        let halved = downscale(stripes, 32);
        for pixel in halved.pixels() {
            assert!((120..=135).contains(&pixel.0[0]), "got {:?}", pixel.0);
        }
    }

    #[test]
    fn png_compression_level_matches_pil_formula() {
        // PIL: int((100 - quality) / 10), and level 0 means stored.
        assert_eq!(png_compression(100), CompressionType::Uncompressed);
        assert_eq!(png_compression(95), CompressionType::Uncompressed);
        assert_eq!(png_compression(90), CompressionType::Level(1));
        assert_eq!(png_compression(50), CompressionType::Level(5));
        assert_eq!(png_compression(0), CompressionType::Level(9));
    }

    #[test]
    fn round_trips_every_output_format() {
        let dir = tempfile::tempdir().unwrap();
        let img = gradient(32, 16);

        for format in [OutputFormat::Jpg, OutputFormat::Png, OutputFormat::Tiff] {
            let path = dir.path().join(format!("out.{}", format.extension()));
            save_image(&img, &path, format, 95, false).expect("save");

            let read = load_rgb(&path).expect("reload");
            assert_eq!(read.dimensions(), (32, 16), "{format:?}");
            if format != OutputFormat::Jpg {
                assert_eq!(read.as_raw(), img.as_raw(), "{format:?} is lossless");
            }
        }
    }

    #[test]
    fn saving_without_overwrite_refuses_an_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("taken.png");
        std::fs::write(&path, b"someone else's").unwrap();

        let img = gradient(8, 8);
        assert!(matches!(
            save_image(&img, &path, OutputFormat::Png, 95, false),
            Err(Error::CreateFile(_))
        ));
        assert_eq!(std::fs::read(&path).unwrap(), b"someone else's");

        save_image(&img, &path, OutputFormat::Png, 95, true).expect("overwrite");
        assert_eq!(load_rgb(&path).unwrap().as_raw(), img.as_raw());
    }

    #[test]
    fn jpeg_refuses_what_it_cannot_encode() {
        let too_wide = RgbImage::new(u32::from(u16::MAX) + 1, 1);
        assert!(matches!(
            jpeg_data_url(&too_wide),
            Err(Error::TooLargeForJpeg {
                width: 65536,
                height: 1
            })
        ));
    }

    #[test]
    fn probe_reads_the_header_and_rejects_a_broken_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("scan.png");
        save_image(&gradient(24, 10), &path, OutputFormat::Png, 95, false).unwrap();

        let facts = probe(&path).expect("probe");
        assert_eq!(facts.format, "PNG");
        assert_eq!((facts.width, facts.height), (24, 10));
        assert_eq!(facts.color, "RGB, 24-bit");
        assert!(facts.orientation.is_none(), "an untagged file is upright");

        let broken = dir.path().join("broken.png");
        std::fs::write(&broken, b"not actually a png").unwrap();
        assert!(
            probe(&broken).is_err(),
            "a corrupt file must fail, not panic"
        );
    }
}
