//! A recovered unique index alongside an untouched cold entity partition.
use super::{current_field_meanings::*, current_publications::create_entity, region_publications};
use std::sync::{atomic::Ordering, Arc};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    config::RelationalRuntimeProfile,
    durability::*,
    identity::*,
    runtime::{
        InvariantCatalog, InvariantRegistration, InvariantRule, RelationalRuntime,
        RelationalRuntimeApi,
    },
};
use worth_testkit_relational::{
    cold_recovery::capture_source,
    durable_medium::{immutable_members::ImmutableCheckpointMemberSource, TestDurableMedium},
    durable_publication::schema,
};
pub(super) const REMOTE: PartitionId = PartitionId(41);
pub(super) struct CandidateFixture {
    pub runtime: RelationalRuntime,
    pub source: Arc<ImmutableCheckpointMemberSource>,
    pub medium: Arc<TestDurableMedium>,
    pub remote: EntityId,
    pub floor: usize,
    pub directory: super::selected_directory::Directory,
}
pub(super) fn runtime(medium: &Arc<TestDurableMedium>) -> RelationalRuntime {
    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(schema())
        .invariant_catalog(InvariantCatalog {
            registrations: vec![InvariantRegistration::commit_boundary_blocking(
                InvariantRule::unique_entity_aspect_field(worth_relational::facade::identity::KindId(1), worth_relational::facade::indexes::DerivedIndexId(1), aspect_key("name"), field_key("name")),
            )],
        })
        .build_durable(medium.attachment())
        .unwrap();
    worth_testkit_relational::aspect_storage::install_declared_unique_field_indexes(&runtime);
    runtime
}
pub(super) fn recovered(request: &ExecutionRequest<'_, '_>) -> CandidateFixture {
    let medium = TestDurableMedium::new();
    let producer = runtime(&medium);
    let remote = region_publications::create_entity(request, &producer, "remote-value", REMOTE);
    let entity = create_entity(request, &producer, "member-before");
    let original = capture_source(request, producer, medium, entity, false);
    let directory = super::selected_directory::read(&original.input);
    let medium = TestDurableMedium::new();
    let attachment = medium.attachment();
    for bytes in original.source.commits.values() {
        let artifact =
            RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap();
        assert!(matches!(
            attachment.publish(&artifact),
            DurablePublicationFate::Committed
        ));
    }
    let mut runtime = runtime(&medium);
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            request,
            original.input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(request, plan)
        .unwrap();
    assert_eq!(original.source.reads.load(Ordering::Relaxed), 0);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    assert!(runtime
        .entity_record_at_observation(request, &basis.observation(), entity)
        .unwrap()
        .is_some());
    drop(basis);
    let main_branch = worth_relational::facade::history::BranchId("main".into());
    assert_eq!(
        runtime
            .observe_branch_region_residency(&main_branch, REMOTE)
            .unwrap()
            .full,
        RelationalRegionProbeStatus::Cold
    );
    let floor = runtime
        .observe_branch_partition_slot_census(&main_branch, PartitionId::main())
        .unwrap()
        .entity_slot_count;
    CandidateFixture {
        runtime,
        source: original.source,
        medium,
        remote,
        floor,
        directory,
    }
}
