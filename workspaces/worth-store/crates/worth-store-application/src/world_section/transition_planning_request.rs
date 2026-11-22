//! Actual Home-owner request issuance before plan authoring or journal effects.
use super::readmission::WorldReadmittedApplicationHome;
use crate::open::readmission::transition::HomeTransitionPlanningOrigin;
use worth_execution::ExecutionRequest;
use worth_proof::AdmittedHomeProgramTransitionPlanningRequest;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    durability::{
        ReadmittedRelationalHomeProgramCatalogs, RefusedRelationalHomeProgramCatalogReadmission,
    },
    runtime::{PreparedRelationalHomeProgramContracts, RelationalHomeProgramPlanningSource},
};
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
    NativeRecoveryRequired,
    Catalogs(Box<RefusedRelationalHomeProgramCatalogReadmission>),
    PlanningRoot(
        Box<worth_relational::facade::durability::RelationalHomeProgramCatalogReadmissionDenial>,
    ),
    /// A resumed plan must have been compiled on its readmitted planning root.
    FreshCompilationBasis,
}

/// Store's decision of which predecessor the author plans against.
pub enum ApplicationHomeTransitionPlanningRequest {
    /// No intent is recorded: the author plans on the current head.
    Fresh(AdmittedHomeProgramTransitionPlanningRequest<RelationalHomeProgramPlanningSource>),
    /// The recorded pending intent must be recomputed on its original
    /// planning root. Only this Home mints the requirement.
    Resume(ApplicationHomeTransitionResumeRequired),
}
pub struct ApplicationHomeTransitionResumeRequired {
    _owner: (),
}

/// The pending intent's original planning root, readmitted by Native with the
/// successor's executors installed on its durable prefix. Custody of those
/// roots stays with the caller until the transition finishes or refuses.
pub struct ResumedApplicationHomeTransitionPlanning {
    admitted: AdmittedHomeProgramTransitionPlanningRequest<RelationalHomeProgramPlanningSource>,
    catalogs: ReadmittedRelationalHomeProgramCatalogs,
}
impl ResumedApplicationHomeTransitionPlanning {
    /// The original planning root is `catalogs.predecessor()`.
    pub fn into_parts(
        self,
    ) -> (
        AdmittedHomeProgramTransitionPlanningRequest<RelationalHomeProgramPlanningSource>,
        ReadmittedRelationalHomeProgramCatalogs,
    ) {
        (self.admitted, self.catalogs)
    }
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
        ApplicationHomeTransitionPlanningRequest,
        ApplicationHomeTransitionPlanningRequestDenial,
    > {
        use ApplicationHomeTransitionPlanningRequestDenial as Denial;
        request.consult().map_err(Denial::Execution)?;
        let context = &self.home.context;
        match context
            .transition
            .planning_origin(&context.descriptor.binding)
            .map_err(Denial::Transition)?
        {
            HomeTransitionPlanningOrigin::Fresh => {
                context.opening_observation.observe_transition_for_test(
                    crate::ApplicationHomeTransitionOpeningObservation::Fresh,
                );
                let basis = expected.basis().relational_basis();
                self.admit_planning_source(expected, basis, request)
                    .map(ApplicationHomeTransitionPlanningRequest::Fresh)
            }
            HomeTransitionPlanningOrigin::Resume => {
                self.require_current_source(expected)?;
                Ok(ApplicationHomeTransitionPlanningRequest::Resume(
                    ApplicationHomeTransitionResumeRequired { _owner: () },
                ))
            }
        }
    }

    /// Readmit the pending intent's original planning root and its durable
    /// prefix through Native. `target` must be the recorded successor's exact
    /// prepared catalogs, compiled on `compilation`, whose planning root is
    /// installed as the author's lens without a second readmission. Nothing
    /// here opens, validates or executes a step.
    pub fn readmit_home_program_transition_planning_root(
        &self,
        _required: ApplicationHomeTransitionResumeRequired,
        compilation: &super::ApplicationHomeTransitionCompilationBasis,
        expected: &ProductBranchObservation,
        target: &PreparedRelationalHomeProgramContracts,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        ResumedApplicationHomeTransitionPlanning,
        ApplicationHomeTransitionPlanningRequestDenial,
    > {
        use ApplicationHomeTransitionPlanningRequestDenial as Denial;
        request.consult().map_err(Denial::Execution)?;
        let root = compilation
            .planning_root()
            .ok_or(Denial::FreshCompilationBasis)?;
        let native = self.home.relational.runtime();
        let authority = self
            .home
            .relational
            .world_recovery_authority()
            .ok_or(Denial::NativeRecoveryRequired)?;
        let catalogs = {
            let grant =
                crate::open::readmission::transition::admit_catalog(&self.home.context, request)
                    .map_err(Denial::Transition)?;
            authority
                .readmit_home_program_catalogs(request, native, &grant, root, target)
                .map_err(|cause| Denial::Catalogs(Box::new(cause)))?
        };
        let admitted = self.admit_planning_source(expected, catalogs.predecessor(), request)?;
        Ok(ResumedApplicationHomeTransitionPlanning { admitted, catalogs })
    }

    fn admit_planning_source(
        &self,
        expected: &ProductBranchObservation,
        basis: &AdmittedRelationalBranchBasis,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        AdmittedHomeProgramTransitionPlanningRequest<RelationalHomeProgramPlanningSource>,
        ApplicationHomeTransitionPlanningRequestDenial,
    > {
        use ApplicationHomeTransitionPlanningRequestDenial as Denial;
        request.consult().map_err(Denial::Execution)?;
        let actor = self.require_current_source(expected)?;
        let source = RelationalHomeProgramPlanningSource::from_owner_source(
            actor,
            basis.admission_identity().clone(),
        );
        let admitted = self.home.context.transition.admit_planning_request(source);
        self.home
            .relational
            .runtime()
            .admit_home_program_transition_planning_request(&admitted, basis)
            .map_err(Denial::Native)?;
        Ok(admitted)
    }

    fn require_current_source(
        &self,
        expected: &ProductBranchObservation,
    ) -> Result<
        worth_foundational::facade::DurableWorldOwnerIdentity,
        ApplicationHomeTransitionPlanningRequestDenial,
    > {
        use ApplicationHomeTransitionPlanningRequestDenial as Denial;
        let owner = self.world.owner_identity();
        let actor = owner.durable().ok_or(Denial::DurableActorUnavailable)?;
        if actor.home() != self.home.context.descriptor.identity
            || expected.owner_identity() != owner
        {
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
        Ok(actor)
    }
}
