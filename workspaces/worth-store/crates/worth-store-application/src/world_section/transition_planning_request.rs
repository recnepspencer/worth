//! Actual Home-owner request issuance before plan authoring or journal effects.
use super::readmission::WorldReadmittedApplicationHome;
use worth_execution::ExecutionRequest;
use worth_relational::facade::runtime::RelationalHomeProgramPlanningSource;
use worth_runtime_world::facade::ProductBranchObservation;

#[derive(Debug)]
pub enum ApplicationHomeTransitionPlanningRequestDenial {
    Execution(worth_execution::WorkCeilingDenial),
    Transition(crate::ApplicationHomeTransitionIntentDenial),
    DurableActorUnavailable,
    ForeignOwner,
    SourceChanged,
    Observation(
        worth_runtime_world::facade::RuntimeWorldServiceDenial<
            worth_runtime_world::facade::RuntimeWorldBranchAdmissionDenial,
        >,
    ),
    Native(worth_relational::facade::runtime::RelationalHomeProgramContractDenial),
}

impl<D, I, E, Ctx, T> WorldReadmittedApplicationHome<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// The caller supplies a selected Source, never actor DATA or a seal.
    /// The returned permission admits planning only, not a step or publication.
    pub fn admit_home_program_transition_planning_request(
        &self,
        expected: &ProductBranchObservation,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        worth_proof::AdmittedHomeProgramTransitionPlanningRequest<
            RelationalHomeProgramPlanningSource,
        >,
        ApplicationHomeTransitionPlanningRequestDenial,
    > {
        use ApplicationHomeTransitionPlanningRequestDenial as Denial;
        request.consult().map_err(Denial::Execution)?;
        let context = &self.home.context;
        context
            .transition
            .require_fresh_plan(&context.descriptor.binding)
            .map_err(Denial::Transition)?;
        let owner = self.world.owner_identity();
        let actor = owner.durable().ok_or(Denial::DurableActorUnavailable)?;
        if actor.home() != context.descriptor.identity || expected.owner_identity() != owner {
            return Err(Denial::ForeignOwner);
        }
        let current = self
            .world
            .observation_port()
            .observe_root_product_branch()
            .map_err(Denial::Observation)?;
        if &current != expected {
            return Err(Denial::SourceChanged);
        }
        let source = RelationalHomeProgramPlanningSource::from_owner_source(
            actor,
            expected
                .basis()
                .relational_basis()
                .admission_identity()
                .clone(),
        );
        let admitted = context.transition.admit_planning_request(source);
        self.home
            .relational
            .runtime()
            .admit_home_program_transition_planning_request(
                &admitted,
                expected.basis().relational_basis(),
            )
            .map_err(Denial::Native)?;
        Ok(admitted)
    }
}
