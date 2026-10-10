use super::super::super::support::*;
use super::fixtures::{
    display_name_field, handle_field, local_policy_projection_read, local_policy_result_read,
    narrowing_policy_tenant_inputs, PolicyTenantInputs,
};
use super::policy_runtime::read_runtime_with_permissive_policy_row;
use crate::authorized_projection::{AuthorizedProjectionFailureClass, PolicyAspectMask};
use crate::ordinary::live::{
    declare_live, WorthQueryLiveOpenOutcome, WorthQueryManagedLiveCloseOutcome,
};
use crate::ordinary::read::{
    current, declare, WorthQueryReadCompletion, WorthQueryReadContextDenialSource,
    WorthQueryReadContextKind, WorthQueryReadNextAction,
};
use crate::policy_narrowing::PolicyNarrowingFailureClass;

#[test]
fn ordinary_policy_narrowing_drives_planning_and_execution_evidence() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let visible = run_policy_projection(
        request_execution,
        1,
        PolicyAspectMask::allow_all(),
        "ordinary-policy-visible",
    );
    let narrowed = run_policy_projection(
        request_execution,
        1,
        PolicyAspectMask::allow_all()
            .with_masked(display_name_field().source_field_key().clone())
            .with_masked(handle_field().source_field_key().clone()),
        "ordinary-policy-narrowed",
    );

    assert_eq!(
        visible.context_receipt().canonical_query_digest(),
        narrowed.context_receipt().canonical_query_digest()
    );
    assert_ne!(
        visible.context_receipt().policy_narrowing_digest(),
        narrowed.context_receipt().policy_narrowing_digest()
    );
    assert_eq!(
        visible
            .result()
            .receipt()
            .breadth()
            .planned_read_surface_count(),
        narrowed
            .result()
            .receipt()
            .breadth()
            .planned_read_surface_count()
            + 2
    );
    assert_eq!(
        visible
            .result()
            .receipt()
            .breadth()
            .execution_query_projection_count(),
        3
    );
    assert_eq!(
        narrowed
            .result()
            .receipt()
            .breadth()
            .execution_query_projection_count(),
        1
    );
    assert_narrowed_completion(&narrowed);
}

#[test]
fn query_enforces_authorized_projection_against_a_permissive_source_adapter() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = declare(local_policy_projection_read)
        .expect("policy projection declaration should canonicalize");
    let context = narrowed_context(narrowing_policy_tenant_inputs(
        1,
        PolicyAspectMask::allow_all()
            .with_masked(display_name_field().source_field_key().clone())
            .with_masked(handle_field().source_field_key().clone()),
    ));
    let mut workspace = read_runtime_with_permissive_policy_row(request_execution)
        .workspace("ordinary-policy-permissive-source")
        .expect("ordinary workspace should open");

    let completion = declaration
        .using(context)
        .run(request_execution, &mut workspace)
        .into_result()
        .expect("admitted policy narrowing should execute");
    let delivered_fields = completion.result().rows()[0]
        .terminal_field_value_projection()
        .into_keys()
        .collect::<Vec<_>>();

    assert_eq!(delivered_fields, ["identity.id"]);
    assert_eq!(
        completion
            .result()
            .receipt()
            .breadth()
            .execution_query_projection_count(),
        1
    );
}

#[test]
fn managed_live_promotion_preserves_the_policy_narrowed_read_graph() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime_with_permissive_policy_row(request_execution)
        .workspace("managed-live-policy-narrowing")
        .expect("managed policy workspace should open");
    let one_shot = declare(local_policy_projection_read)
        .expect("one-shot policy parity read should declare")
        .using(narrowed_context(narrowing_policy_tenant_inputs(
            1,
            live_policy_narrowing_mask(),
        )))
        .run(request_execution, &mut workspace)
        .into_result()
        .expect("one-shot policy parity read should execute")
        .into_result();
    let opened = match declare_live("users.narrowed", local_policy_projection_read)
        .expect("managed policy read should declare")
        .using(narrowed_context(narrowing_policy_tenant_inputs(
            1,
            live_policy_narrowing_mask(),
        )))
        .open(request_execution, &mut workspace)
    {
        WorthQueryLiveOpenOutcome::Opened(opened) => opened,
        WorthQueryLiveOpenOutcome::Stopped(stop) => {
            panic!("managed policy live open stopped: {:?}", stop.source())
        }
    };
    let read = opened
        .handle()
        .read(request_execution, &mut workspace)
        .expect("managed policy live read should execute");
    let delivered_fields = read.rows()[0]
        .terminal_field_value_projection()
        .into_keys()
        .collect::<Vec<_>>();

    assert_eq!(delivered_fields, ["identity.id"]);
    assert_eq!(read.rows(), one_shot.rows());
    assert_eq!(
        read.receipt().snapshot_identity(),
        one_shot.receipt().snapshot_identity()
    );
    assert!(opened.context_receipt().policy_narrowing_digest().is_some());
    assert!(matches!(
        opened.into_handle().close(&mut workspace),
        WorthQueryManagedLiveCloseOutcome::Closed(_)
    ));
}

fn live_policy_narrowing_mask() -> PolicyAspectMask {
    PolicyAspectMask::allow_all()
        .with_masked(display_name_field().source_field_key().clone())
        .with_masked(handle_field().source_field_key().clone())
}

#[test]
fn equivalent_mask_declaration_order_converges_before_planning() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let display_then_handle = run_policy_projection(
        request_execution,
        1,
        PolicyAspectMask::allow_all()
            .with_masked(display_name_field().source_field_key().clone())
            .with_masked(handle_field().source_field_key().clone()),
        "ordinary-policy-mask-order",
    );
    let handle_then_display = run_policy_projection(
        request_execution,
        1,
        PolicyAspectMask::allow_all()
            .with_masked(handle_field().source_field_key().clone())
            .with_masked(display_name_field().source_field_key().clone()),
        "ordinary-policy-mask-order",
    );

    assert_eq!(
        display_then_handle.context_receipt(),
        handle_then_display.context_receipt()
    );
    assert_eq!(display_then_handle.result(), handle_then_display.result());
}

#[path = "policy_narrowing/masked_result_field_denies_before_graph_planning_or_runtime.rs"]
mod masked_result_field_denies_before_graph_planning_or_runtime;

#[test]
fn changed_policy_epoch_forces_a_fresh_narrowed_handoff() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let first = run_policy_projection(
        request_execution,
        1,
        PolicyAspectMask::allow_all().with_masked(display_name_field().source_field_key().clone()),
        "ordinary-policy-epoch-1",
    );
    let second = run_policy_projection(
        request_execution,
        2,
        PolicyAspectMask::allow_all().with_masked(display_name_field().source_field_key().clone()),
        "ordinary-policy-epoch-2",
    );

    assert_eq!(
        first.context_receipt().canonical_query_digest(),
        second.context_receipt().canonical_query_digest()
    );
    assert_ne!(first.context_receipt(), second.context_receipt());
    assert_ne!(
        first.context_receipt().policy_narrowing_digest(),
        second.context_receipt().policy_narrowing_digest()
    );
    assert_ne!(
        first.result().receipt().read_graph_digest(),
        second.result().receipt().read_graph_digest()
    );
}

#[test]
fn different_masks_in_the_same_policy_epoch_cannot_share_a_handoff() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let display_name_masked = run_policy_projection(
        request_execution,
        1,
        PolicyAspectMask::allow_all().with_masked(display_name_field().source_field_key().clone()),
        "ordinary-policy-mask-identity-display-name",
    );
    let handle_masked = run_policy_projection(
        request_execution,
        1,
        PolicyAspectMask::allow_all().with_masked(handle_field().source_field_key().clone()),
        "ordinary-policy-mask-identity-handle",
    );

    assert_ne!(
        display_name_masked
            .context_receipt()
            .policy_tenant_admission_digest(),
        handle_masked
            .context_receipt()
            .policy_tenant_admission_digest()
    );
    assert_ne!(
        display_name_masked
            .context_receipt()
            .policy_narrowing_digest(),
        handle_masked.context_receipt().policy_narrowing_digest()
    );
    assert_ne!(
        display_name_masked.result().receipt().read_graph_digest(),
        handle_masked.result().receipt().read_graph_digest()
    );
}

fn run_policy_projection(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    epoch: u64,
    projection_mask: PolicyAspectMask,
    workspace_name: &str,
) -> WorthQueryReadCompletion {
    let declaration = declare(local_policy_projection_read)
        .expect("policy projection declaration should canonicalize");
    let context = narrowed_context(narrowing_policy_tenant_inputs(epoch, projection_mask));
    let mut workspace = read_runtime(request_execution)
        .workspace(workspace_name)
        .expect("ordinary workspace should open");
    declaration
        .using(context)
        .run(request_execution, &mut workspace)
        .into_result()
        .expect("admitted policy narrowing should execute")
}

fn narrowed_context(
    policy_tenant: PolicyTenantInputs,
) -> crate::ordinary::read::WorthQueryCurrentPolicyTenantReadContext {
    current().under_policy_tenant(
        policy_tenant.policy,
        policy_tenant.tenant,
        policy_tenant.branch,
        policy_tenant.schema,
    )
}

fn assert_narrowed_completion(completion: &WorthQueryReadCompletion) {
    let context = completion.context_receipt();
    let context_counters = context.counters();
    let receipt = completion.result().receipt();
    let journey = completion.journey_counters();

    assert_eq!(
        context.context_kind(),
        WorthQueryReadContextKind::CurrentPolicyTenant
    );
    assert_eq!(context_counters.policy_tenant_admission_attempt_count(), 1);
    assert_eq!(context_counters.policy_tenant_admitted_count(), 1);
    assert_eq!(context_counters.policy_narrowing_attempt_count(), 1);
    assert_eq!(context_counters.policy_narrowing_admitted_count(), 1);
    assert_eq!(
        context_counters.relationship_proof_admission_attempt_count(),
        1
    );
    assert_eq!(context_counters.relationship_proof_admitted_count(), 1);
    assert_eq!(
        context_counters.graph_authority_admission_attempt_count(),
        1
    );
    assert_eq!(context_counters.graph_authority_admitted_count(), 1);
    assert_eq!(
        context.policy_narrowing_digest(),
        receipt.policy_narrowing_digest()
    );
    assert!(receipt.policy_aware_plan_digest().is_some());
    assert!(receipt.policy_execution_seam_identity().is_some());
    assert_eq!(receipt.policy_executor_semantic_rediscovery_count(), 0);
    assert_eq!(journey.planning_attempt_count(), 1);
    assert_eq!(journey.planning_completed_count(), 1);
    assert_eq!(journey.lower_runtime_execution_attempt_count(), 1);
    assert_eq!(journey.lower_runtime_execution_completed_count(), 1);
}
