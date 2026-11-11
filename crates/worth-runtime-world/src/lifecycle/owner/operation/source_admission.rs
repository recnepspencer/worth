//! Publication rejects stale Home authority before reservation or lowering.
use super::*;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn admit_publication_source(
        &self,
        expected: &ProductBranchObservation,
    ) -> Result<crate::branch::ProductBranchReferenceSnapshot, NoEffectCompositePublication> {
        if let Err(cause) = self
            .owner_identity()
            .admit_incarnation(expected.owner_identity())
        {
            let cause = match cause {
                worth_proof::HomeIncarnationDenial::StaleIncarnation => {
                    NoEffectCause::StaleIncarnation
                }
                worth_proof::HomeIncarnationDenial::ForeignHome => {
                    NoEffectCause::OwnerDeniedBeforeEffect
                }
            };
            return Err(NoEffectCompositePublication::new(
                cause,
                Some(expected.clone()),
            ));
        }
        if self.owner_identity() != expected.owner_identity() {
            return Err(NoEffectCompositePublication::new(
                NoEffectCause::OwnerDeniedBeforeEffect,
                Some(expected.clone()),
            ));
        }
        if !self.is_open_and_bootstrapped() {
            return Err(NoEffectCompositePublication::new(
                NoEffectCause::OwnerUnavailable,
                Some(expected.clone()),
            ));
        }
        self.state
            .branches
            .branch_cell(expected.branch_identity())
            .map(|cell| cell.atomic_snapshot())
            .ok_or_else(|| {
                NoEffectCompositePublication::new(
                    NoEffectCause::OwnerUnavailable,
                    Some(expected.clone()),
                )
            })
    }
}
