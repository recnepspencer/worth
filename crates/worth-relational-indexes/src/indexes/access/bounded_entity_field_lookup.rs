use super::selected_index::PinnedIndexLookup;
use crate::branch::indexes::data::bounded_entity_field_lookup::{
    BoundedEntityFieldLookupAdmissionStop as Stop, BoundedEntityFieldLookupDenial,
    BoundedEntityFieldLookupDenialKind, BoundedEntityFieldLookupDenialOwnerPort as _,
    BoundedEntityFieldLookupOutcome, BoundedEntityFieldLookupRequest, BoundedIndexParityMode,
};

pub fn lookup(
    read: PinnedIndexLookup<'_>,
    request: BoundedEntityFieldLookupRequest,
    parity_mode: BoundedIndexParityMode,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<BoundedEntityFieldLookupOutcome, BoundedEntityFieldLookupDenial> {
    let unavailable = || {
        BoundedEntityFieldLookupDenial::new(
            BoundedEntityFieldLookupDenialKind::SnapshotUnavailable,
            request.index_id(),
        )
    };
    if request.snapshot() != read.snapshot {
        return Err(unavailable());
    }
    let prepared = super::pinned_entity_field_lookup::prepare(
        read.runtime_identity,
        read.resident,
        read.instrumentation,
        read.indexes,
        request.index_id(),
        request.entity_kind(),
        request.field_locator(),
        execution,
    )?;
    prepared
        .collect_prepared_entity_field_lookup(
            request.value(),
            request.candidate_limit(),
            parity_mode,
            execution,
            || Ok::<_, std::convert::Infallible>(()),
        )
        .map_err(|stop| match stop {
            Stop::Lookup(denial) => denial,
            Stop::Admission(never) => match never {},
            Stop::ExactBasisRequired | Stop::AccountingOverflow => unavailable(),
        })
}
