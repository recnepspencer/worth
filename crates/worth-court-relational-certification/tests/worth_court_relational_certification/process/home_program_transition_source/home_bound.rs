//! A receiver whose medium is bound to a Home transition owner, and the
//! one-step recorded plan that owner admits. The court mints the owner because
//! no real Home can durably hold a Native tail its own plan does not cover.
use super::*;
use std::num::NonZeroU64;
use worth_foundational::facade::{
    CompositeAttemptAssociation, DurableCompositeCommitIdentity,
    DurableCompositePublicationAttemptIdentity, DurableProductBranchIdentity,
    DurableProductBranchIncarnation, DurableProductBranchReference,
    DurableProductReferenceGeneration, DurableWorldOwnerIdentity, HomeIdentity,
    HomeProgramTransitionCatalogDigest, HomeProgramTransitionIdentity,
    HomeProgramTransitionPlanDescription, HomeProgramTransitionPlanDigest,
    HomeProgramTransitionSchemaCarrierDigest, HomeProgramTransitionSchemaDigest,
    HomeProgramTransitionTargetMeaning, WorldOwnerSlotIdentity,
};
use worth_proof::{
    AdmittedHomeProgramTransitionCatalog, HomeIncarnationFence, HomeProgramTransitionAuthority,
    HomeProgramTransitionOwner,
};
use worth_relational::facade::{config::RelationalRuntimeProfile, runtime::RelationalRuntimeApi};

#[derive(Debug)]
struct HomeBound {
    inner: Box<dyn RelationalDurableMedium>,
    authority: HomeProgramTransitionAuthority,
}
impl RelationalDurableMedium for HomeBound {
    fn home_transition_authority(&self) -> Option<&HomeProgramTransitionAuthority> {
        Some(&self.authority)
    }
    fn publish_initial_inputs(
        &self,
        artifact: &RelationalSourceInitialInputsArtifact,
        request: &ExecutionRequest<'_, '_>,
    ) -> DurablePublicationFate<RelationalInitialInputsPublicationDetail> {
        self.inner.publish_initial_inputs(artifact, request)
    }
    fn preflight_source_retirement(
        &self,
        source: &dyn RelationalDurableColdSource,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalMediumRetirementDenial> {
        self.inner.preflight_source_retirement(source, request)
    }
    fn advance_source_retirement(
        &mut self,
        source: &dyn RelationalDurableColdSource,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RelationalMediumRetirementProgress, RelationalMediumRetirementDenial> {
        self.inner.advance_source_retirement(source, request)
    }
    fn checkpoint_selection(
        &self,
        subject: &OpaqueCapturedCheckpointSubject,
    ) -> CheckpointSelectionObservation {
        self.inner.checkpoint_selection(subject)
    }
    fn cold_source(&self) -> Arc<dyn RelationalDurableColdSource> {
        self.inner.cold_source()
    }
    fn read_transition(
        &self,
        sequence: RelationalBranchTransitionSequence,
    ) -> Result<RelationalBranchTransitionArtifact, RelationalDurableTransitionReadFailure> {
        self.inner.read_transition(sequence)
    }
    fn publish(
        &self,
        artifact: &RelationalCanonicalCommitArtifact,
    ) -> DurablePublicationFate<String> {
        self.inner.publish(artifact)
    }
    fn publish_transition(
        &self,
        artifact: &RelationalBranchTransitionArtifact,
    ) -> DurablePublicationFate<String> {
        self.inner.publish_transition(artifact)
    }
}

pub(super) fn owner() -> HomeProgramTransitionOwner {
    HomeProgramTransitionOwner::fresh(
        [1; 16],
        HomeIncarnationFence::from_owner(NonZeroU64::new(1).unwrap()),
    )
}

/// The same journal as `initial_input_source::receiver`, bound to `owner`.
pub(super) fn receiver(
    medium: &Arc<TestDurableMedium>,
    owner: &HomeProgramTransitionOwner,
) -> RelationalRuntime {
    let mut runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(worth_testkit_relational::durable_publication::schema())
        .build_durable(Box::new(HomeBound {
            inner: medium.attachment(),
            authority: owner.authority(),
        }))
        .unwrap();
    runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(initial_input_source::additions())
        .unwrap();
    runtime
}

/// A one-step plan recorded by `owner`'s Home, with one World assignment.
pub(super) struct RecordedPlan {
    pub(super) plan: HomeProgramTransitionPlanDescription,
    pub(super) assignments: [CompositeAttemptAssociation; 1],
}
pub(super) fn one_step_plan() -> RecordedPlan {
    let home = HomeIdentity::from_observed_bytes([1; 16]).unwrap();
    let world =
        DurableWorldOwnerIdentity::from_recorded(home, WorldOwnerSlotIdentity::from_recorded(1));
    let branch = DurableProductBranchIdentity::from_recorded(world, "main".to_owned()).unwrap();
    let incarnation = DurableProductBranchIncarnation::from_recorded(world, 1);
    let predecessor = DurableProductBranchReference::from_recorded(
        branch.clone(),
        incarnation,
        DurableProductReferenceGeneration::from_recorded(1),
        DurableCompositeCommitIdentity::from_recorded(world, 1),
    )
    .unwrap();
    let plan = HomeProgramTransitionPlanDescription::from_recorded(
        HomeProgramTransitionIdentity::from_recorded(home, NonZeroU64::new(1).unwrap()),
        predecessor,
        HomeProgramTransitionPlanDigest::from_recorded([7; 32]),
        1,
        HomeProgramTransitionTargetMeaning::from_recorded(
            0,
            HomeProgramTransitionCatalogDigest::from_recorded([9; 32]),
            HomeProgramTransitionSchemaDigest::from_recorded([11; 32]),
            HomeProgramTransitionSchemaCarrierDigest::from_recorded([13; 32]),
        ),
    )
    .unwrap();
    let assignment = CompositeAttemptAssociation::from_recorded(
        DurableCompositePublicationAttemptIdentity::from_recorded(world, 1),
        branch,
        incarnation,
    )
    .unwrap();
    RecordedPlan {
        plan,
        assignments: [assignment],
    }
}

/// `owner`'s catalog grant for `recorded`, whose one step's caller digest is
/// `caller`. Nothing here describes what Native actually made durable.
pub(super) fn grant<'intent>(
    owner: &HomeProgramTransitionOwner,
    recorded: &'intent RecordedPlan,
    predecessor: &'intent RecordedRelationalComponentReference,
    caller: [u8; 32],
    request: &ExecutionRequest<'_, '_>,
) -> AdmittedHomeProgramTransitionCatalog<RelationalHomeProgramCatalogReadmissionSource<'intent>> {
    let mut callers = worth_execution::ExecutionRequestArrayBuilder::allocate(*request, 1).unwrap();
    callers.push(caller).unwrap();
    owner.admit_catalog(
        recorded.plan.commitment(),
        RelationalHomeProgramCatalogReadmissionSource::from_recorded(
            &recorded.plan,
            predecessor,
            &recorded.assignments,
            callers.seal().unwrap(),
        ),
    )
}
