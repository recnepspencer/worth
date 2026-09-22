//! PNG encoding of a native client-area capture, for retained failure
//! artifacts and for the capture directory a courtroom run may be asked to
//! fill. The capture is the observer-neutral RGBA record every native
//! platform produces, so the encoder names no capture vendor.
use std::fmt;

use crate::external_observation::NativeClientPixelCapture;

#[derive(Debug)]
pub(crate) enum NativeCapturePngFailure {
    /// The capture's byte length is not `width * height * 4`.
    DimensionsMismatchBytes,
    Encode(png::EncodingError),
}

pub(crate) fn encode_native_capture_png(
    capture: &NativeClientPixelCapture,
) -> Result<Vec<u8>, NativeCapturePngFailure> {
    let expected_len = (capture.width() as usize)
        .checked_mul(capture.height() as usize)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(NativeCapturePngFailure::DimensionsMismatchBytes)?;
    if capture.rgba().len() != expected_len {
        return Err(NativeCapturePngFailure::DimensionsMismatchBytes);
    }
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, capture.width(), capture.height());
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .map_err(NativeCapturePngFailure::Encode)?;
    writer
        .write_image_data(capture.rgba())
        .map_err(NativeCapturePngFailure::Encode)?;
    writer.finish().map_err(NativeCapturePngFailure::Encode)?;
    Ok(bytes)
}

impl fmt::Display for NativeCapturePngFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DimensionsMismatchBytes => {
                formatter.write_str("native capture dimensions do not match its bytes")
            }
            Self::Encode(error) => write!(formatter, "encode native capture png: {error}"),
        }
    }
}
