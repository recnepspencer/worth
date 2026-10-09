use crate::indexes::data::{
    BoundedEntityFieldLookupAdmissionStop as Stop, BoundedEntityFieldLookupDenial,
    BoundedEntityFieldLookupDenialKind, BoundedEntityFieldLookupOutcome,
    BoundedEntityFieldLookupRequest, BoundedIndexParityMode,
};
use crate::visibility::snapshot_states::resolve_snapshot_handle;

use super::IndexAccess;

impl IndexAccess<'_> {
    pub fn execute_bounded_entity_field_lookup(
        &self,
        request: BoundedEntityFieldLookupRequest,
        parity_mode: BoundedIndexParityMode,
    ) -> Result<BoundedEntityFieldLookupOutcome, BoundedEntityFieldLookupDenial> {
        self.runtime
            .performance_access()
            .count_query_index_attempt();
        let unavailable = || {
            BoundedEntityFieldLookupDenial::new(
                BoundedEntityFieldLookupDenialKind::SnapshotUnavailable,
                request.index_id(),
            )
        };
        let snapshot =
            resolve_snapshot_handle(self.runtime, request.snapshot()).ok_or_else(unavailable)?;
        let view = self
            .runtime
            .read_truth()
            .project_snapshot(&snapshot)
            .ok_or_else(unavailable)?;
        let prepared = self.prepare_entity_field_lookup(
            &view,
            request.index_id(),
            request.entity_kind(),
            request.field_locator(),
        )?;
        self.collect_prepared_entity_field_lookup(
            &prepared,
            request.value(),
            request.candidate_limit(),
            parity_mode,
            || Ok::<_, std::convert::Infallible>(()),
        )
        .map_err(|stop| match stop {
            Stop::Lookup(denial) => denial,
            Stop::Admission(never) => match never {},
            Stop::ExactBasisRequired | Stop::AccountingOverflow => unavailable(),
        })
    }
}
