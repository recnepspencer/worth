//! The current World receiver authenticates Home planning before issuing IDs.
use super::*;
impl<D, I, E, Ctx, T> RuntimeWorldPublicationPort<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn reserve_home_transition_assignments(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        expected: ProductBranchObservation,
        description: worth_foundational::facade::HomeProgramTransitionPlanDescription,
        proof: worth_proof::AdmittedHomeProgramTransitionPlan,
    ) -> Result<
        crate::publication::ReservedWorldHomeTransitionAssignments,
        crate::publication::WorldHomeTransitionReservationRefusal,
    > {
        let owner = self.service().map_err(|_| {
            crate::publication::WorldHomeTransitionReservationRefusal::Source(
                NoEffectCompositePublication::new(
                    NoEffectCause::OwnerUnavailable,
                    Some(expected.clone()),
                ),
            )
        })?;
        owner.reserve_home_transition_assignments(request, expected, description, proof)
    }
}
