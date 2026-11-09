//! Independent receiving owner over copied actual checkpoint placement and committed prefix.
use super::{
    immutable_recovery_input::copy_input,
    selected_directory::{self, Directory},
};
use std::sync::{atomic::Ordering, Arc};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{durability::*, runtime::RelationalRuntime};
use worth_testkit_relational::{
    cold_recovery::ColdRecoverySourceFixture,
    durable_medium::{immutable_members::ImmutableCheckpointMemberSource, TestDurableMedium},
    durable_publication::durable_runtime,
};
pub(super) struct CheckpointReceiver {
    pub runtime: RelationalRuntime,
    pub medium: Arc<TestDurableMedium>,
    pub source: Arc<ImmutableCheckpointMemberSource>,
    pub directory: Directory,
}
pub(super) fn reopen(
    request: &ExecutionRequest<'_, '_>,
    original: &ColdRecoverySourceFixture,
) -> CheckpointReceiver {
    let (input, source) = copy_input(original);
    let directory = selected_directory::read(&input);
    let medium = TestDurableMedium::new();
    let attachment = medium.attachment();
    for bytes in source.commits.values() {
        let artifact =
            RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap();
        assert!(matches!(
            attachment.publish(&artifact),
            DurablePublicationFate::Committed
        ));
    }
    let mut runtime = durable_runtime(&medium);
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(request, plan)
        .unwrap();
    assert_eq!(source.reads.load(Ordering::Relaxed), 0);
    CheckpointReceiver {
        runtime,
        medium,
        source,
        directory,
    }
}
