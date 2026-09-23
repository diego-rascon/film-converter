//! The one error type every fallible function returns.

use std::io;

use serde::{Serialize, Serializer};

/// Something that went wrong between a path and a developed file.
///
/// Each message is what the user reads — the notice panel shows it, and a
/// run's failure list puts it beside the file's name — so it says what went
/// wrong rather than which function noticed. Per-file failures are lower case
/// because they follow a name; the run-level ones stand alone and are not.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not open: {0}")]
    Open(#[source] io::Error),

    #[error("could not read: {0}")]
    Read(#[source] io::Error),

    #[error("unsupported image: {0}")]
    Unsupported(#[source] image::ImageError),

    #[error("could not decode: {0}")]
    Decode(#[source] image::ImageError),

    #[error("image is {width}x{height}, larger than JPEG's 65535 pixel limit")]
    TooLargeForJpeg { width: u32, height: u32 },

    #[error("could not encode JPEG: {0}")]
    EncodeJpeg(#[from] jpeg_encoder::EncodingError),

    #[error("could not encode {format}: {source}")]
    Encode {
        format: &'static str,
        source: image::ImageError,
    },

    #[error("could not create file: {0}")]
    CreateFile(#[source] io::Error),

    #[error("could not finish writing: {0}")]
    Write(#[source] io::Error),

    #[error("No images to develop")]
    NothingToDevelop,

    #[error("No output folder selected")]
    NoOutputFolder,

    #[error("Could not use the output folder: {0}")]
    OutputFolder(#[source] io::Error),

    /// The blocking task a command handed its work to never reported back.
    #[error("The work was interrupted: {0}")]
    Interrupted(#[from] tauri::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Commands reject with the message itself, so the front end receives the
/// same plain string whichever command failed.
impl Serialize for Error {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
