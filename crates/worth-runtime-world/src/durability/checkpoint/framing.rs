//! World1 is one bounded, owner-framed inline Home section.
use crate::durability::codec::fields::Emit;
use crate::durability::{WorldDurableFrameDenial as Frame, WorldDurableMediumDenial as Denial};
use worth_execution::{
    charge_framed_bytes, decode_framed_envelope, encode_framed_array, ExecutionImmutableBytes,
    ExecutionRequest, FramedEnvelopeError, FramedEnvelopeSpec, FRAMED_ENVELOPE_HEADER_BYTES,
};
#[cfg(test)]
mod tests;

pub(super) const MAXIMUM: usize = 1 << 30;
const VERSION: worth_foundational::facade::BoundaryProtocolVersion =
    super::WORLD_CHECKPOINT_VERSION;
const SPEC: FramedEnvelopeSpec = FramedEnvelopeSpec {
    magic: *b"WWRLDCP1",
    supported_versions: &[VERSION.get()],
    max_payload_bytes: (MAXIMUM - FRAMED_ENVELOPE_HEADER_BYTES) as u64,
};

pub(super) fn charge(request: &ExecutionRequest<'_, '_>, bytes: usize) -> Result<(), Denial> {
    charge_framed_bytes(request, bytes).map_err(Denial::Request)
}

pub(super) fn encode(
    request: &ExecutionRequest<'_, '_>,
    payload: &mut dyn FnMut(&mut Emit<'_>) -> Result<(), Denial>,
) -> Result<ExecutionImmutableBytes, Denial> {
    encode_framed_array(
        SPEC,
        VERSION.get(),
        request,
        |sink| crate::durability::framed_denial::forward_payload(payload, sink),
        Denial::Backing,
    )
    .map(|array| array.into_immutable_bytes())
}

pub(super) fn payload<'a>(
    bytes: &'a [u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<&'a [u8], Denial> {
    decode_framed_envelope(SPEC, bytes, request)
        .map(|(_, payload)| payload)
        .map_err(frame_denial)
}

fn frame_denial(cause: FramedEnvelopeError) -> Denial {
    match cause {
        FramedEnvelopeError::Execution(stop) => Denial::Request(stop),
        FramedEnvelopeError::PayloadTooLarge { .. } => Frame::Malformed.into(),
        cause => crate::durability::framed_denial::wire(cause, VERSION).into(),
    }
}
