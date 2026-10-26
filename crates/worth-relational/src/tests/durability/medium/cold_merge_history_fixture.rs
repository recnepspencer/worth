//! A nonhead source change overlaps the target after another source commit.
use super::super::*;

pub(super) struct Fixture {
    pub runtime: crate::runtime::RelationalRuntime,
    pub source: Arc<MemberSource>,
    pub overlap: crate::identity::data::EntityId,
    pub intermediate: crate::history::data::RelationalCommitReceipt,
    pub position: crate::publication::patch::data::PatchStreamPosition,
}
pub(super) fn branches() -> (BranchId, BranchId) {
    (
        BranchId("main".into()),
        BranchId("intermediate-source".into()),
    )
}
impl Fixture {
    pub(super) fn reopened(execution: &worth_execution::ExecutionRequest<'_, '_>) -> Self {
        let producer = durable_runtime_with_test_schema();
        let a = create_entity(execution, &producer, "merge-a-base");
        let b = create_entity(execution, &producer, "merge-b-base");
        let (target, source_branch) = branches();
        create_branch_from_main(&producer, &source_branch.0);
        let first = update_entity_on_branch(
            execution,
            &producer,
            a,
            "merge-a-source",
            source_branch.clone(),
        );
        let intermediate = first.commit.clone();
        let position = first.patch_position();
        release_test_commit_snapshot(&producer, &first);
        let head =
            update_entity_on_branch(execution, &producer, b, "merge-b-source", source_branch);
        release_test_commit_snapshot(&producer, &head);
        let target_update =
            update_entity_on_branch(execution, &producer, a, "merge-a-target", target);
        release_test_commit_snapshot(&producer, &target_update);
        let (recovery, source, _) = checkpoint_fixture(execution, producer, a, false);
        source.damaged.store(false, Ordering::Relaxed);
        let medium = TestDurableMedium::new().attachment();
        for bytes in source.commits.values() {
            let artifact = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap();
            assert!(matches!(
                medium.publish(&artifact),
                crate::durability::medium::RelationalDurablePublication::Committed
            ));
        }
        let mut runtime = RelationalRuntimeApi::builder()
            .profile(RelationalRuntimeProfile::CertificationCore)
            .schema_registry(test_schema_registry())
            .build_durable(medium);
        let plan = runtime
            .durability()
            .recovery_plan_from_source(
                execution,
                recovery,
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .unwrap();
        runtime
            .durability_recovery()
            .recover(execution, plan)
            .unwrap();
        assert!(runtime
            .history
            .commit_artifact(intermediate.commit_id)
            .unwrap()
            .resident_envelope()
            .is_none());
        Self {
            runtime,
            source,
            overlap: a,
            intermediate,
            position,
        }
    }
}
