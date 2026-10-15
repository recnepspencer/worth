//! Exact selected symbol meaning, borrowed while the native pin retains custody.
use super::{
    denial, WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationObservedFact,
};
use worth_foundational::facade::{AspectFieldLocator, AspectValue, InternedString};
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::{RelationalPinnedRead, RelationalRuntime},
    snapshots::SnapshotHandle,
    symbols::Symbol,
};

pub(super) fn pin(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
) -> Result<RelationalPinnedRead, Denial> {
    use worth_relational::facade::runtime::RelationalPinnedReadAdmissionDenial as Native;
    runtime
        .read_truth()
        .pin_read_capability(snapshot, &[], execution)
        .map_err(|cause| match cause {
            Native::Request(stop) => {
                Denial::relational_read_stopped(stop.into(), "workflow approval text pin")
            }
            Native::SnapshotUnavailable | Native::IndexNotInstalled(_) => {
                denial("workflow approval exact text basis is unavailable")
            }
        })
}

pub(super) fn decode<Output>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    read: &RelationalPinnedRead,
    entity: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
    decode: impl FnOnce(&str) -> Result<Output, Denial>,
) -> Result<Output, Denial> {
    super::super::super::observation::observe_field_value_borrowed(
        runtime,
        execution,
        read.snapshot(),
        entity,
        kind,
        locator,
        |value| {
            let value = value.ok_or_else(|| denial("workflow approval text is unavailable"))?;
            let AspectValue::String(InternedString::Symbol(symbol)) = value else {
                return Err(denial(
                    "workflow approval text is not current native symbol storage",
                ));
            };
            let text = read
                .symbols()
                .resolve(Symbol(symbol.0))
                .ok_or_else(|| denial("workflow approval symbol meaning is unavailable"))?;
            facts.push(WorthQueryApplicationObservedFact::Field {
                entity_id: entity,
                kind,
                locator: locator.clone(),
                value: value.clone(),
            });
            decode(text)
        },
    )
    .map_err(|cause| Denial::relational_read_stopped(cause, "workflow approval text"))?
    .ok_or_else(|| denial("workflow approval text entity is unavailable"))?
}

pub(super) fn decode_lineage(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    lineage: &str,
) -> Result<crate::domain_computation::authorization::workflow_approval_authority::decoding::DecodedApprovalLineage, Denial>{
    if lineage.len() > crate::domain_computation::authorization::MAXIMUM_PORTABLE_BYTES {
        return Err(denial(
            "workflow approval capability lineage exceeds its bound",
        ));
    }
    crate::domain_computation::authorization::workflow_approval_authority::decoding::decode_lineage(
        execution, lineage,
    )
    .map_err(|cause| {
        crate::domain_computation::primary_graph::workflow::actor::decoded_denial(
            cause,
            "workflow approval capability lineage is invalid",
        )
    })
}
