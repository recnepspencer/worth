use super::{WorldDurableFrameDenial as Frame, WorldDurableMediumDenial as Denial};
use worth_execution::FramedEnvelopeError as Error;
use worth_foundational::facade::{BoundaryProtocolCompatibilityWindow, BoundaryProtocolVersion};

pub(super) fn wire(error: Error, version: BoundaryProtocolVersion) -> Frame {
    match error {
        Error::UnsupportedVersion { found } => BoundaryProtocolVersion::try_new(found)
            .ok()
            .and_then(|found| {
                BoundaryProtocolCompatibilityWindow::inclusive(version, version)
                    .admit(found)
                    .err()
            })
            .map_or(Frame::Malformed, Frame::UnsupportedVersion),
        Error::ChecksumMismatch => Frame::DigestMismatch,
        Error::PayloadTooLarge { .. } | Error::Allocation => Frame::SizeLimit,
        _ => Frame::Malformed,
    }
}

impl From<Error> for Denial {
    fn from(error: Error) -> Self {
        match error {
            Error::Execution(cause) => Self::Request(cause),
            other => Self::Frame(wire(other, super::artifact::WORLD_ARTIFACT_VERSION)),
        }
    }
}

// The World field grammar can refuse malformed data; the byte sink can also
// stop for the original request or backing. Preserve the actual source.
pub(super) fn forward_payload<E: Into<Denial>>(
    payload: &mut dyn FnMut(&mut super::codec::fields::Emit<'_>) -> Result<(), E>,
    sink: &mut dyn FnMut(&[u8]) -> Result<(), Denial>,
) -> Result<(), Denial> {
    let mut refusal = None;
    let result = payload(&mut |bytes| {
        sink(bytes).map_err(|cause| {
            refusal = Some(cause);
            Frame::Malformed
        })
    });
    match refusal {
        Some(cause) => Err(cause),
        None => result.map_err(Into::into),
    }
}
