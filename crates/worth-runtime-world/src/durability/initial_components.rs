//! Transport each owner's mandatory initial-component recipe without copying its schema.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct RecordedWorldInitialComponents {
    pub(crate) relational: worth_relational::facade::branch::RecordedRelationalComponentReference,
    pub(crate) signal: worth_signal::facade::branch::RecordedSignalInitialEmptyRoot,
    pub(crate) bridge: worth_runtime_bridge::facade::RecordedRuntimeWorldBaseline,
}
impl RecordedWorldInitialComponents {
    pub(super) fn from_json_in(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        bytes: &[u8],
    ) -> Result<Self, super::WorldDurableMediumDenial> {
        use super::{WorldDurableFrameDenial as Frame, WorldDurableMediumDenial as Denial};
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Fields<'a> {
            #[serde(borrow)]
            relational: &'a serde_json::value::RawValue,
            #[serde(borrow)]
            signal: &'a serde_json::value::RawValue,
            #[serde(borrow)]
            bridge: &'a serde_json::value::RawValue,
        }
        let work = u64::try_from(bytes.len()).map_err(|_| Denial::Frame(Frame::SizeLimit))?;
        execution
            .run_native_with_checkpoint(|context| context.checkpoint(work))
            .map_err(Denial::Request)?
            .0
            .map_err(|cause| Denial::Request(worth_execution::WorkCeilingDenial::Stopped(cause)))?;
        let scratch = bytes
            .len()
            .checked_mul(4)
            .and_then(|bytes| bytes.checked_add(256))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(Denial::Frame(Frame::SizeLimit))?;
        let _scratch = execution
            .reserve_retained_memory(scratch)
            .map_err(Denial::Request)?;
        let fields: Fields<'_> =
            serde_json::from_slice(bytes).map_err(|_| Denial::Frame(Frame::Malformed))?;
        Ok(Self {
            relational: worth_relational::facade::branch::RecordedRelationalComponentReference::from_json_in(
                execution, fields.relational.get().as_bytes(),
            ).map_err(|cause| Denial::NativeDescription(Box::new(cause)))?,
            signal: worth_signal::facade::branch::RecordedSignalInitialEmptyRoot::from_json_in(
                execution, fields.signal.get().as_bytes(),
            ).map_err(Denial::SignalDescription)?,
            bridge: worth_runtime_bridge::facade::RecordedRuntimeWorldBaseline::from_json_in(
                execution, fields.bridge.get().as_bytes(),
            ).map_err(Denial::BridgeDescription)?,
        })
    }
}
