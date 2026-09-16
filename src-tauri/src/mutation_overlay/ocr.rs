use std::fmt;

use windows::core::HSTRING;
use windows::Globalization::Language;
use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap};
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::DataWriter;

use super::capture::GrayFrame;

#[derive(Clone, Debug, PartialEq)]
pub enum OcrError {
    Unavailable,
    InvalidFrame,
    Windows(String),
}

impl fmt::Display for OcrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable => f.write_str("Windows OCR English recognizer is unavailable"),
            Self::InvalidFrame => f.write_str("captured Mutation frame is invalid"),
            Self::Windows(message) => write!(f, "Windows OCR failed: {message}"),
        }
    }
}

impl std::error::Error for OcrError {}

pub trait MutationOcr: Send + Sync {
    fn recognize(&self, frame: &GrayFrame) -> Result<String, OcrError>;
}

#[derive(Default)]
pub struct WindowsMutationOcr;

fn map_windows(error: windows::core::Error) -> OcrError {
    OcrError::Windows(error.message().to_string_lossy())
}

fn software_bitmap(frame: &GrayFrame) -> Result<SoftwareBitmap, OcrError> {
    let expected = frame.width as usize * frame.height as usize;
    if frame.width == 0 || frame.height == 0 || frame.pixels.len() != expected {
        return Err(OcrError::InvalidFrame);
    }

    // DataWriter owns an in-memory WinRT buffer. No screenshot or temporary
    // image file is ever written to disk.
    let writer = DataWriter::new().map_err(map_windows)?;
    writer.WriteBytes(&frame.pixels).map_err(map_windows)?;
    let buffer = writer.DetachBuffer().map_err(map_windows)?;
    let _ = writer.Close();

    SoftwareBitmap::CreateCopyFromBuffer(
        &buffer,
        BitmapPixelFormat::Gray8,
        frame.width as i32,
        frame.height as i32,
    )
    .map_err(map_windows)
}

fn english_engine() -> Result<OcrEngine, OcrError> {
    let language = Language::CreateLanguage(&HSTRING::from("en-US")).map_err(map_windows)?;
    if !OcrEngine::IsLanguageSupported(&language).map_err(map_windows)? {
        return Err(OcrError::Unavailable);
    }
    OcrEngine::TryCreateFromLanguage(&language).map_err(|_| OcrError::Unavailable)
}

impl MutationOcr for WindowsMutationOcr {
    fn recognize(&self, frame: &GrayFrame) -> Result<String, OcrError> {
        let bitmap = software_bitmap(frame)?;
        let engine = english_engine()?;
        let result = engine
            .RecognizeAsync(&bitmap)
            .map_err(map_windows)?
            .join()
            .map_err(map_windows)?;
        let text = result.Text().map_err(map_windows)?.to_string();
        let _ = bitmap.Close();
        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::{software_bitmap, OcrError};
    use crate::mutation_overlay::capture::GrayFrame;

    #[test]
    fn malformed_frames_are_rejected_before_winrt() {
        let frame = GrayFrame {
            width: 4,
            height: 4,
            pixels: vec![0; 3],
        };
        assert_eq!(software_bitmap(&frame).unwrap_err(), OcrError::InvalidFrame);
    }
}
