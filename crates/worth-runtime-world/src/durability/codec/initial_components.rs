//! Length-delimited owner recipes never translate or snapshot Signal's schema.
use super::{
    fields::{number, Emit, Read},
    owner_value,
};
use crate::durability::{
    RecordedWorldInitialComponents as Components, WorldDurableFrameDenial as Frame,
    WorldDurableMediumDenial as Medium,
};
use worth_execution::ExecutionRequest;
pub(super) fn write(out: &mut Emit<'_>, components: &Components) -> Result<(), Frame> {
    json_component(out, &components.relational)?;
    number(out, components.signal.as_bytes().len() as u64)?;
    out(components.signal.as_bytes())?;
    json_component(out, &components.bridge)
}
fn json_component<T: serde::Serialize>(out: &mut Emit<'_>, component: &T) -> Result<(), Frame> {
    let mut length = 0u64;
    owner_value::emit(
        &mut |bytes| {
            length = length
                .checked_add(bytes.len() as u64)
                .ok_or(Frame::SizeLimit)?;
            Ok(())
        },
        component,
    )?;
    number(out, length)?;
    owner_value::emit(out, component)
}
pub(super) fn read(
    input: &mut Read<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Components, Medium> {
    let relational =
        worth_relational::facade::branch::RecordedRelationalComponentReference::from_json_in(
            request,
            component(input)?,
        )
        .map_err(|cause| Medium::NativeDescription(Box::new(cause)))?;
    let signal_bytes = component(input)?;
    if signal_bytes.len()
        > worth_signal::facade::durability::SignalRecordedSourcePoint::MAXIMUM_ENCODED_BYTES
    {
        return Err(Frame::SizeLimit.into());
    }
    let signal =
        crate::durability::artifact::WorldSignalEvidence::from_bytes(signal_bytes, request)?;
    let bridge = worth_runtime_bridge::facade::RecordedRuntimeWorldBaseline::from_json_in(
        request,
        component(input)?,
    )
    .map_err(bridge_decode_denial)?;
    Ok(Components {
        relational,
        signal,
        bridge,
    })
}
fn component<'a>(input: &mut Read<'a>) -> Result<&'a [u8], Frame> {
    let length = usize::try_from(input.number()?).map_err(|_| Frame::SizeLimit)?;
    input.take(length)
}

/// Decoder refusal classification carries the original request stop unchanged.
pub(in crate::durability) fn bridge_decode_denial(
    cause: worth_runtime_bridge::facade::RuntimeWorldBaselineRefusal,
) -> Medium {
    use worth_runtime_bridge::facade::RuntimeWorldBaselineRefusal as Refusal;
    match cause {
        Refusal::Request(cause) => Medium::Request(cause),
        Refusal::Description
        | Refusal::UnsupportedVersion { .. }
        | Refusal::RegisteredSourceRequired
        | Refusal::BaselineCorrespondenceRequired
        | Refusal::RecordedMeaningMismatch
        | Refusal::SignalGraphBasisRequired
        | Refusal::CurrentBasisRequired => Medium::Frame(Frame::Malformed),
    }
}
