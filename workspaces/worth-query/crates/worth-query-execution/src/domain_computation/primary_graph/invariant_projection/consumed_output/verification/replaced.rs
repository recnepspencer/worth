//! Verification of the consumed roots a required successor did not replace.

use super::*;

impl ConsumedOutputEvidence {
    /// Verifies every root but those whose pending equality a successor replaced.
    pub(in crate::domain_computation::primary_graph) fn verify_many_except(
        roots: &[Self],
        replaced: &[Arc<RecordedSettlementIdentity>],
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop> {
        charge_external(admission, roots.len().saturating_mul(replaced.len()))?;
        let rest: Vec<EvidenceView<'_>> = roots
            .iter()
            .filter(|root| !replaced.contains(&root.identity))
            .map(EvidenceView::from)
            .collect();
        Self::verify_views(
            rest.iter().copied(),
            owner,
            runtime,
            snapshot,
            selected,
            admission,
        )
    }
}
