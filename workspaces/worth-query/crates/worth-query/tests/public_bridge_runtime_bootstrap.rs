use crate::support;
use worth_foundational::facade::{CanonicalFieldPath, FieldKey};
use worth_query::facade::foundation::WorthQueryEntityIdentity;
use worth_query::facade::runtime::{
    WorthQueryExistingEntityTarget, WorthQueryExistingTruthBindingAuthorityLabel,
    WorthQueryExistingTruthProbeDenialKind, WorthQueryExistingTruthProbeMode,
    WorthQueryExistingTruthProbeRequest, WorthQueryExistingTruthTargetBinding, WorthQueryLiveView,
    WorthQueryMutationAuthorityIdentity, WorthQueryRuntimeError, WorthQueryUnrefinedLiveShape,
};

use support::aspect_touch as touch;
use support::public_bridge_runtime::{
    public_bridge_runtime_bootstrap_invocation_count, public_graph_support_profile,
    reset_public_bridge_runtime_bootstrap_invocations, PublicBridgeRuntimeBootstrapPath,
    PublicBridgeRuntimeHarness,
};

fn public_entity_verified_profile() -> worth_query::facade::runtime::WorthQueryRuntimeSupportProfile
{
    public_graph_support_profile().with_bridge_backed_verification_support(
        "probe_existing",
        "direct_entity_identity",
        true,
        true,
        None,
    )
}

fn existing_authority(label: &str) -> WorthQueryMutationAuthorityIdentity {
    WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(
        WorthQueryExistingTruthBindingAuthorityLabel::new(label)
            .expect("existing-truth authority label"),
    )
    .expect("existing-truth authority identity")
}

fn existing_task_binding(
    authority_label: &str,
    entity_token: &str,
) -> WorthQueryExistingTruthTargetBinding {
    WorthQueryExistingTruthTargetBinding::from_entity_target(
        WorthQueryExistingEntityTarget::new(
            existing_authority(authority_label),
            WorthQueryEntityIdentity::admit_authored_entity_token(
                worth_query::facade::foundation::QueryExternalIdentityToken::new(
                    std::sync::Arc::from(entity_token),
                ),
            ),
        )
        .expect("existing entity target should build")
        .in_target_collection("Task")
        .expect("existing entity target collection should build"),
    )
    .expect("binding should build")
}

#[test]
fn public_bridge_runtime_common_bootstrap_lane_builds_runtime_backed_live_reads() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness.bridge_backed_runtime(request_execution);
    let mut workspace = runtime
        .workspace("public.bridge-runtime-bootstrap.common-lane")
        .expect("runtime should open a named workspace");
    let tasks: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "public.bridge-runtime-bootstrap.common-lane.tasks",
            |q| {
                q.from("Task")
                    .select([
                        worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "identity", "id",
                        )
                        .unwrap(),
                        worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "title", "value",
                        )
                        .unwrap(),
                    ])
                    .order_by(
                        worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "title", "value",
                        )
                        .unwrap(),
                    )
                    .schema_basis("public-bridge-runtime-bootstrap-common-lane-tasks")
            },
        )
        .expect("task live view should declare");

    workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(touch("identity.id"), authored_text("task-bootstrap"))
                .set_aspect(touch("title.value"), authored_text("Bootstrap task"))
        })
        .expect("insert should execute through the public bridge-backed bootstrap lane");

    let rows = workspace.read(request_execution, &tasks);

    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].scalar_value_at(&field_path("identity.id")),
        Some(&text("task-bootstrap"))
    );
    assert_eq!(
        rows[0].scalar_value_at(&field_path("title.value")),
        Some(&text("Bootstrap task"))
    );
}

#[test]
fn public_bridge_runtime_builder_lane_supports_seeded_existing_truth_probe() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness
        .bridge_backed_runtime_builder()
        .support_profile(public_entity_verified_profile())
        .build(request_execution);
    let workspace = runtime
        .workspace("public.bridge-runtime-bootstrap.builder-lane")
        .expect("runtime should open a named workspace");
    let binding = existing_task_binding("authority:task-bootstrap", "public-entity-1");
    let seed = harness.seed_backend_authoritative_truth(
        &binding,
        touch("title.value"),
        text("Seeded bootstrap task"),
    );
    assert_eq!(seed.binding_digest(), binding.binding_digest());
    assert_eq!(seed.target_collection(), "Task");
    assert_eq!(
        seed.admitted_aspect_touch_reporting_projection(),
        "title:value"
    );

    let probe = workspace
        .probe_existing_intent(
            WorthQueryExistingTruthProbeRequest::new(binding, [touch("title.value")])
                .expect("probe request should build"),
        )
        .execute(request_execution)
        .expect("probe should execute through the public builder bootstrap lane")
        .probe()
        .clone();

    assert_eq!(
        probe.mode(),
        WorthQueryExistingTruthProbeMode::BackendVerifiedProbe
    );
    assert_eq!(
        probe
            .field_for_touch(&touch("title.value"))
            .expect("title field should exist")
            .foundational_value(),
        &text("Seeded bootstrap task")
    );
}

#[test]
fn public_bridge_runtime_builder_lane_missing_existing_truth_probe_fails_closed() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness
        .bridge_backed_runtime_builder()
        .support_profile(public_entity_verified_profile())
        .build(request_execution);
    let workspace = runtime
        .workspace("public.bridge-runtime-bootstrap.builder-lane-missing-probe")
        .expect("runtime should open a named workspace");
    let binding =
        existing_task_binding("authority:task-bootstrap-missing", "public-entity-missing");

    let error = workspace
        .probe_existing_intent(
            WorthQueryExistingTruthProbeRequest::new(binding, [touch("title.value")])
                .expect("probe request should build"),
        )
        .execute(request_execution)
        .expect_err("backend-verified probe must deny missing authoritative truth");

    match error {
        WorthQueryRuntimeError::ExistingTruthProbeDenied(denial) => {
            assert_eq!(
                denial.kind(),
                WorthQueryExistingTruthProbeDenialKind::MissingProbedAspect
            );
        }
        other => panic!("expected typed missing probe denial, got {other:?}"),
    }
}

#[test]
fn public_bridge_runtime_common_lane_fail_closes_existing_truth_probe_without_verification_support()
{
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness.bridge_backed_runtime(request_execution);
    let workspace = runtime
        .workspace("public.bridge-runtime-bootstrap.common-lane-unsupported-probe")
        .expect("runtime should open a named workspace");
    let binding = existing_task_binding("authority:task-bootstrap", "public-entity-1");

    let error = workspace
        .probe_existing_intent(
            WorthQueryExistingTruthProbeRequest::new(binding, [touch("title.value")])
                .expect("probe request should build"),
        )
        .execute(request_execution)
        .expect_err("common bootstrap lane should deny undeclared verification support");

    match error {
        WorthQueryRuntimeError::ExistingTruthProbeDenied(denial) => {
            assert_eq!(
                denial.kind(),
                WorthQueryExistingTruthProbeDenialKind::BackendProbeUnsupported
            );
        }
        other => panic!("expected typed probe denial, got {other:?}"),
    }
}

#[test]
fn public_bridge_runtime_builder_lane_usage_stays_explicit() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    reset_public_bridge_runtime_bootstrap_invocations();

    let harness = PublicBridgeRuntimeHarness::new();
    let _runtime = harness
        .bridge_backed_runtime_builder()
        .support_profile(public_graph_support_profile())
        .build(request_execution);

    assert_eq!(
        public_bridge_runtime_bootstrap_invocation_count(PublicBridgeRuntimeBootstrapPath::Common),
        0
    );
    assert_eq!(
        public_bridge_runtime_bootstrap_invocation_count(PublicBridgeRuntimeBootstrapPath::Builder),
        1
    );
}

fn authored_text(
    value: impl Into<String>,
) -> worth_query::facade::runtime::WorthQueryAuthoredAspectValue {
    worth_query::facade::runtime::WorthQueryAuthoredAspectValue::string(value)
}

fn text(value: impl Into<String>) -> worth_foundational::facade::AspectValue {
    worth_foundational::facade::AspectValue::String(value.into().into())
}

fn field_path(path: &str) -> CanonicalFieldPath {
    CanonicalFieldPath::new(
        path.split('.').map(|segment| {
            FieldKey::new(segment).expect("test field path segment should be valid")
        }),
    )
    .expect("test field path should be non-empty")
}

#[test]
fn public_bridge_runtime_common_and_builder_bootstrap_counts_stay_lane_local() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    reset_public_bridge_runtime_bootstrap_invocations();

    let harness = PublicBridgeRuntimeHarness::new();
    let _common_runtime = harness.bridge_backed_runtime(request_execution);
    let _builder_runtime = harness
        .bridge_backed_runtime_builder()
        .support_profile(public_entity_verified_profile())
        .build(request_execution);
    let _second_common_runtime = harness.bridge_backed_runtime(request_execution);

    assert_eq!(
        public_bridge_runtime_bootstrap_invocation_count(PublicBridgeRuntimeBootstrapPath::Common),
        2
    );
    assert_eq!(
        public_bridge_runtime_bootstrap_invocation_count(PublicBridgeRuntimeBootstrapPath::Builder),
        1
    );
}
