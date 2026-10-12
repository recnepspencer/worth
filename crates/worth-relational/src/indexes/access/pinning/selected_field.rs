//! Selected view admission freezes its generation before calling collection.
use super::IndexAccess;
use crate::identity::data::KindId;
use crate::indexes::data::{
    BoundedEntityFieldLookupAdmissionStop as Stop, BoundedEntityFieldLookupDenial,
    BoundedEntityFieldLookupDenialKind as Denial, BoundedEntityFieldLookupOutcome,
    BoundedIndexParityMode, DerivedIndexId, SelectedIndexReadWork, MAX_BOUNDED_INDEX_CANDIDATES,
};
use crate::runtime::VisibilityProjectionView;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
fn denied<E>(kind: Denial, index: DerivedIndexId) -> Stop<E> {
    Stop::Lookup(BoundedEntityFieldLookupDenial::new(kind, index))
}
impl IndexAccess<'_> {
    /// The exact selected-basis variant of the ordinary bounded lookup. The
    /// issued view supplies snapshot and schema authority; each index read,
    /// native candidate comparison and Certification storage visit uses the
    /// same caller admission before its work or allocation. The callback budgets
    /// Operation against logical work; OrderedNavigation is one operation plus
    /// a single capacity-bounded keyed descent, never candidates or payload.
    pub(in crate::indexes::access) fn collect_admitted_entity_field_lookup<E>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        view: &VisibilityProjectionView,
        index: DerivedIndexId,
        kind: KindId,
        locator: &AspectFieldLocator,
        value: &AspectValue,
        limit: usize,
        parity: BoundedIndexParityMode,
        mut prepare: impl FnMut(SelectedIndexReadWork, u64) -> Result<(), E>,
    ) -> Result<BoundedEntityFieldLookupOutcome, Stop<E>> {
        if !view.is_exact_basis() || !view.is_from_runtime(self.runtime) {
            return Err(Stop::ExactBasisRequired);
        }
        if limit == 0 || limit > MAX_BOUNDED_INDEX_CANDIDATES {
            return Err(denied(Denial::InvalidCandidateLimit, index));
        }
        self.runtime
            .performance_access()
            .count_query_index_attempt();
        let read = self
            .runtime
            .read_truth()
            .pin_selected_field_read_capability(view, index, locator, execution, &mut prepare)?;
        read.collect_admitted_entity_field_lookup(
            execution, index, kind, locator, value, limit, parity, prepare,
        )
    }
}
