//! Candidate meaning uses the original product occurrence coordinate and
//! the actual proposed field/edge revisions. It issues no Current source.
use super::*;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

impl WorthQueryObservedSourceMeaningRegistry {
    pub(in crate::domain_computation::primary_graph::application_query) fn intern_applied_compared(
        &self,
        query: &[u8; 32],
        parameters: &[u8; 32],
        footprint: WorthQueryObservedSourceFootprint,
        before: &worth_runtime_world::facade::ProductBranchObservation,
        applied: &worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        equal: &mut impl FnMut(
            &WorthQueryObservedSourceFootprint,
            &WorthQueryObservedSourceFootprint,
        ) -> Result<bool, Cause>,
    ) -> Result<Option<Arc<WorthQueryObservedSourceMeaning>>, Cause> {
        if applied.proposal().branch_observation() != before.basis().relational_basis().reference()
        {
            return Ok(None);
        }
        // This genuine before selection supplies only its product incarnation
        // to the existing coordinate encoder. The returned meaning contains no
        // observed-source selection or observation generation. The caller keeps
        // its separate Native candidate qualification through producer execution.
        self.intern_compared(
            query,
            parameters,
            footprint,
            &WorthQueryApplicationBasisSelectionIdentity::Product(
                crate::basis::WorthQueryProductBranchReadIdentity::from_observation(before),
            ),
            equal,
        )
    }
}
