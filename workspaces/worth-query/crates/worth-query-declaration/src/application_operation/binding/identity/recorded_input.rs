//! Capture and decode the existing canonical input stream as DATA, never authority.
mod buffered_enum;
mod compound;
mod deserializer;
mod reader;
#[cfg(test)]
mod tests;
use super::{
    canonical_identity_with_capture, encoder, ApplicationCanonicalInput, ApplicationEncodedInput,
    INPUT_DOMAIN,
};
use crate::application_schema::{ApplicationStructuredValueBinding, ApplicationValueEncodeDenial};
use reader::{CanonicalInputDecodeError, Reader};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use worth_foundational::facade::{CanonicalDigestId, ValueCodecDenial, ValueDecodeBounds};

/// Typed refusal to retain or reconstruct a canonical input value.
/// The returned input is fresh value data; it carries no original authorization.
#[derive(Debug)]
pub enum ApplicationRetainedInputDenial {
    Codec(ValueCodecDenial),
    UnsupportedProtocol,
    BindingMismatch,
    IdentityMismatch,
    Encoding(ApplicationValueEncodeDenial),
}
impl From<CanonicalInputDecodeError> for ApplicationRetainedInputDenial {
    fn from(cause: CanonicalInputDecodeError) -> Self {
        Self::Codec(cause.0)
    }
}
impl<Binding> ApplicationEncodedInput<Binding>
where
    Binding: ApplicationStructuredValueBinding,
    Binding::Value: Serialize,
{
    /// Retain the original canonical stream. An existing capture is shared; otherwise
    /// this explicitly encodes the same value and refuses if its identity changed.
    pub fn capture_for_retention(
        &self,
    ) -> Result<ApplicationCanonicalInput, ApplicationRetainedInputDenial> {
        if let Some(captured) = self.canonical_input() {
            return Ok(captured.clone());
        }
        let captured = canonical_identity_with_capture(
            INPUT_DOMAIN,
            Binding::IDENTITY.as_str(),
            self.input(),
            encoder::HashingSink::new().capturing(true),
        )
        .map_err(capture_denial)?;
        if captured.identity() != *self.identity() {
            return Err(ApplicationRetainedInputDenial::IdentityMismatch);
        }
        captured
            .capture
            .ok_or(ApplicationRetainedInputDenial::Codec(
                ValueCodecDenial::InternalInconsistency,
            ))
    }
}
impl<Binding> ApplicationEncodedInput<Binding>
where
    Binding: ApplicationStructuredValueBinding,
    Binding::Value: Serialize + DeserializeOwned,
{
    /// Decode recorded bytes under the supplied representation contract, then validate
    /// and canonically encode the fresh value under this exact binding. This issues
    /// neither admission nor a historical capability. No handler or inverse runs.
    pub fn decode_retained(
        bytes: &[u8],
        expected: CanonicalDigestId,
        bounds: ValueDecodeBounds,
    ) -> Result<Self, ApplicationRetainedInputDenial> {
        let payload = admit_frame(bytes, Binding::IDENTITY.as_str(), expected, bounds)?;
        let mut reader = Reader::new(payload, bounds);
        let value = Binding::Value::deserialize(&mut reader)?;
        reader.finished()?;
        let encoded = Self::encode(value).map_err(ApplicationRetainedInputDenial::Encoding)?;
        if encoded.identity() != expected.bytes() {
            return Err(ApplicationRetainedInputDenial::IdentityMismatch);
        }
        Ok(encoded)
    }
}
fn admit_frame<'a>(
    bytes: &'a [u8],
    binding: &str,
    expected: CanonicalDigestId,
    bounds: ValueDecodeBounds,
) -> Result<&'a [u8], ApplicationRetainedInputDenial> {
    if bounds
        .maximum_wire_bytes()
        .is_some_and(|maximum| bytes.len() > maximum)
    {
        return Err(ApplicationRetainedInputDenial::Codec(
            ValueCodecDenial::WireBytes,
        ));
    }
    let mut cursor = bytes;
    if frame_text(&mut cursor)? != INPUT_DOMAIN {
        return Err(ApplicationRetainedInputDenial::UnsupportedProtocol);
    }
    if frame_text(&mut cursor)? != binding {
        return Err(ApplicationRetainedInputDenial::BindingMismatch);
    }
    let digest: [u8; 32] = Sha256::digest(bytes).into();
    if &digest != expected.bytes() {
        return Err(ApplicationRetainedInputDenial::IdentityMismatch);
    }
    Ok(cursor)
}
fn frame_text<'a>(cursor: &mut &'a [u8]) -> Result<&'a str, CanonicalInputDecodeError> {
    let length = cursor
        .get(..8)
        .ok_or(CanonicalInputDecodeError(ValueCodecDenial::Malformed))?;
    let length = usize::try_from(u64::from_be_bytes(length.try_into().expect("exact length")))
        .map_err(|_| CanonicalInputDecodeError(ValueCodecDenial::WireBytes))?;
    *cursor = &cursor[8..];
    let text = cursor
        .get(..length)
        .ok_or(CanonicalInputDecodeError(ValueCodecDenial::Malformed))?;
    *cursor = &cursor[length..];
    std::str::from_utf8(text).map_err(|_| CanonicalInputDecodeError(ValueCodecDenial::Malformed))
}
fn capture_denial(cause: encoder::CanonicalEncodeError) -> ApplicationRetainedInputDenial {
    use encoder::CanonicalEncodeError as Cause;
    ApplicationRetainedInputDenial::Codec(match cause {
        Cause::Serialization => ValueCodecDenial::UnsupportedShape,
        Cause::CapacityOverflow => ValueCodecDenial::WireBytes,
        Cause::Allocation => ValueCodecDenial::Allocation,
        Cause::Admission(cause) => match cause {},
        Cause::AdmissionDeferred => ValueCodecDenial::InternalInconsistency,
    })
}
