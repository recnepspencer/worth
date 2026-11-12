//! A final Home binding borrows the exact selected product source through I/O.
use super::*;
use crate::lifecycle::{PreparedWorldHomeProgramBinding, WorldHomeProgramBindingDenial as Denial};

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn bind_current_home_program(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        expected: &ProductBranchObservation,
        required: &worth_proof::HomeProgramTransitionRequired,
        successor: &worth_proof::AdmittedHomeProgramTransitionSuccessor<
            &worth_foundational::facade::ApplicationHomeBinding,
        >,
        handoff: &mut dyn for<'owner> FnMut(PreparedWorldHomeProgramBinding<'owner>),
    ) -> Result<(), Denial> {
        let _call = self.try_admit_call().ok_or(Denial::OwnerUnavailable)?;
        if !self.branch_service_is_available() {
            return Err(Denial::OwnerUnavailable);
        }
        self.owner_identity()
            .admit_incarnation(expected.owner_identity())
            .map_err(|cause| match cause {
                worth_proof::HomeIncarnationDenial::ForeignHome => Denial::ForeignOwner,
                worth_proof::HomeIncarnationDenial::StaleIncarnation => Denial::StaleIncarnation,
            })?;
        let _operation = self
            .reserve_creation_operation()
            .map_err(|_| Denial::OwnerUnavailable)?;
        let medium = self.state.durable.as_ref().ok_or(Denial::NotHomeBound)?;
        let prepared = PreparedWorldHomeProgramBinding::prepare(
            medium, expected, required, successor, request,
        )?;
        let cell = self
            .state
            .branches
            .branch_cell(expected.branch_identity())
            .ok_or(Denial::SourceChanged)?;
        // No product head moves in this callback. Its only effect is the Home
        // descriptor binding; it must not reenter this reference cell. Native's
        // original epoch/source guards are acquired before entering this owner.
        cell.while_current(expected, prepared, |prepared| handoff(prepared))
            .map_err(|_| Denial::SourceChanged)
    }
}
