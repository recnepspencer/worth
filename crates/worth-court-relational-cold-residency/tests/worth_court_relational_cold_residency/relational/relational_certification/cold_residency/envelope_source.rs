//! Original checkpoint or Journal-only history on an explicit external receiver.
use std::sync::{atomic::Ordering, Arc};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    config::RelationalRuntimeProfile,
    durability::*,
    runtime::{RelationalRuntime, RelationalRuntimeApi},
};
use worth_testkit_relational::{
    cold_recovery::source_with_profile,
    durable_medium::{immutable_members::ImmutableCheckpointMemberSource, TestDurableMedium},
    durable_publication::schema,
};
pub(super) struct EnvelopeFixture {
    pub runtime: RelationalRuntime,
    pub source: Arc<ImmutableCheckpointMemberSource>,
    pub medium: Arc<TestDurableMedium>,
}
pub(super) fn reopened(
    request: &ExecutionRequest<'_, '_>,
    updates: usize,
    tail_only: bool,
    profile: RelationalRuntimeProfile,
) -> EnvelopeFixture {
    let original = source_with_profile(request, updates, false, profile);
    let input = if tail_only {
        RelationalDurableRecoverySource::new(
            None,
            original
                .source
                .commits
                .values()
                .map(|bytes| {
                    RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap()
                })
                .collect(),
            vec![],
            original.source.clone(),
        )
    } else {
        original.input
    };
    let medium = TestDurableMedium::new();
    let mut runtime = RelationalRuntimeApi::builder()
        .profile(profile)
        .schema_registry(schema())
        .build_durable(medium.attachment())
        .unwrap();
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
    assert_eq!(original.source.reads.load(Ordering::Relaxed), 0);
    EnvelopeFixture {
        runtime,
        source: original.source,
        medium,
    }
}
