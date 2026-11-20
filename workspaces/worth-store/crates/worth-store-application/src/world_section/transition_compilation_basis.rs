//! The root a transition's successor contracts are compiled against.
use super::readmission::WorldReadmittedApplicationHome;
use super::ApplicationHomeTransitionPlanningRequestDenial as Denial;
use crate::open::readmission::transition::HomeTransitionPlanningOrigin;
use worth_execution::ExecutionRequest;
use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_runtime_world::facade::ProductBranchObservation;

impl<D, I, E, Ctx, T> WorldReadmittedApplicationHome<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// A fresh plan compiles its successor on the current head. A recorded
    /// pending intent recompiles on its original planning predecessor, which
    /// Native readmits exactly from this recovery; a head already past the
    /// target step is never the compilation source.
    pub fn admit_home_program_transition_compilation_basis(
        &self,
        expected: &ProductBranchObservation,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<AdmittedRelationalBranchBasis, Denial> {
        request.consult().map_err(Denial::Execution)?;
        let context = &self.home.context;
        match context
            .transition
            .planning_origin(&context.descriptor.binding)
            .map_err(Denial::Transition)?
        {
            HomeTransitionPlanningOrigin::Fresh => Ok(expected.basis().relational_basis().clone()),
            HomeTransitionPlanningOrigin::Resume => {
                let intent = context
                    .transition
                    .committed_intent(&context.descriptor.binding)
                    .map_err(Denial::Transition)?
                    .ok_or(Denial::Transition(
                        crate::ApplicationHomeTransitionIntentDenial::IntentNotCommitted,
                    ))?;
                let authority = self
                    .home
                    .relational
                    .world_recovery_authority()
                    .ok_or(Denial::NativeRecoveryRequired)?;
                authority
                    .admit_recorded_component(
                        request,
                        self.home.relational.runtime(),
                        intent.native_predecessor(),
                    )
                    .map(|recovered| recovered.into_basis())
                    .map_err(|cause| Denial::PlanningRoot(Box::new(cause)))
            }
        }
    }
}
