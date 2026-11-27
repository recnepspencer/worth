//! A candidate's verdict with no marks read, for a family selection's
//! re-resolution at a certification basis (public #429, plan §3.4 step 3).
//!
//! It runs the read's own two comparisons in the read's order: the equal
//! chain in an aligned image, and the full comparison where no image covers
//! `selected` or the chain cannot answer. Only the read retains what a
//! comparison recorded, so the compared rows are dropped here.

use super::*;
use crate::domain_computation::primary_graph::output_lineage::invalidation::EqualityRecoveryImage;
use crate::domain_computation::primary_graph::output_lineage::invalidation::ObservedRows;

impl ConsumedOutputEvidence {
    /// The aligned image a fact's candidates are compared in, or `None` when
    /// no image covers `selected` (the read's own fallback condition).
    pub fn mark_free_image(
        owner: &SourceInvalidationOwner,
        selected: &PositionedRelationalSnapshot,
    ) -> Result<Option<EqualityRecoveryImage>, ConsumedOutputVerificationStop> {
        match owner.equality_recovery_image(selected) {
            Ok(image) => Ok(Some(image)),
            Err(SettlementVerificationStop::Alignment) => Ok(None),
            Err(stop) => Err(map_verification_stop(stop)),
        }
    }

    /// The driving verdict of `evidence` at `selected`, as the read's
    /// `verify_at_observation` maps it, with the equal chain followed in
    /// `image` instead of through marks.
    pub fn compare_mark_free(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        evidence: EvidenceView<'_>,
        image: Option<&EqualityRecoveryImage>,
        runtime: &RelationalRuntime,
        basis: CertificationBasis<'_>,
        selected: &PositionedRelationalSnapshot,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop> {
        let chained = match image {
            Some(image) => Self::compare_equal_chain_in(
                execution,
                evidence,
                image,
                &mut ObservedRows::new(),
                runtime,
                basis,
                selected,
                &mut Vec::new(),
            ),
            None => Err(ConsumedOutputVerificationStop::Unavailable),
        };
        match chained {
            Err(ConsumedOutputVerificationStop::Unavailable) => {
                Self::compare_in_full(execution, evidence, runtime, basis, selected)
            }
            other => other,
        }?
        .driving()
    }
}
