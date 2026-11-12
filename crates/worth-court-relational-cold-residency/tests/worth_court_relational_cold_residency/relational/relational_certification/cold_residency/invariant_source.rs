//! The actual cold producer history under one declared native rule.
use std::sync::{atomic::Ordering, Arc};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    config::RelationalRuntimeProfile,
    durability::*,
    identity::EntityId,
    runtime::{
        InvariantCatalog, InvariantRegistration, InvariantRule, RecordKindTag, RelationalRuntime,
        RelationalRuntimeApi,
    },
};
use worth_testkit_relational::{
    cold_recovery::source_with_history,
    durable_medium::{immutable_members::ImmutableCheckpointMemberSource, TestDurableMedium},
    durable_publication::schema,
};
pub(super) fn recovered(
    request: &ExecutionRequest<'_, '_>,
    damaged: bool,
) -> (
    RelationalRuntime,
    Arc<ImmutableCheckpointMemberSource>,
    EntityId,
    Arc<TestDurableMedium>,
) {
    let original = source_with_history(request, 3, false);
    original.source.damaged.store(damaged, Ordering::Relaxed);
    let medium = TestDurableMedium::new();
    let mut runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(schema())
        .invariant_catalog(InvariantCatalog {
            registrations: vec![InvariantRegistration::mutation_sensitive_blocking(
                InvariantRule::LiveRecordRequiresSidecar(RecordKindTag::Entity),
            )],
        })
        .build_durable(medium.attachment())
        .unwrap();
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
    (runtime, original.source, original.entity, medium)
}
