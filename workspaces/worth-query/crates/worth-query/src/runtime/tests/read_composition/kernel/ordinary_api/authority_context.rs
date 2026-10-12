use super::super::super::support::*;
use super::fixtures::{
    admitted_policy_tenant_inputs, local_identity_read, local_manager_relationship_read,
    run_policy_context, run_relationship_context,
};
use crate::ordinary::read::{
    current, declare, WorthQueryReadNextAction, WorthQueryReadRelationshipProof,
    WorthQueryReadRelationshipProofs,
};
use crate::policy_basis::BranchAccessGrant;
use crate::tenant_basis::{TenantBasisEpoch, TenantBindingSnapshot};

#[test]
fn ordinary_read_lowers_policy_tenant_and_relationship_authority_once() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = declare(local_manager_relationship_read)
        .expect("relationship declaration should canonicalize");
    let policy_tenant = admitted_policy_tenant_inputs(1, true);
    let relationships = WorthQueryReadRelationshipProofs::bounded(
        [WorthQueryReadRelationshipProof::direct_edge(
            manager_relation_name(),
        )],
        1,
        1,
    )
    .expect("bounded relationship proof should declare");
    let context = current()
        .under_policy_tenant(
            policy_tenant.policy,
            policy_tenant.tenant,
            policy_tenant.branch,
            policy_tenant.schema,
        )
        .with_relationship_proofs(relationships);
    let mut workspace = read_runtime(request_execution)
        .workspace("ordinary-read-policy-tenant-relationship")
        .expect("ordinary workspace should open");

    let completion = declaration
        .using(context)
        .run(request_execution, &mut workspace)
        .into_result()
        .expect("fully admitted read should execute");
    let context_receipt = completion.context_receipt();
    let context_counters = context_receipt.counters();
    let executed_authority_receipt = completion
        .result()
        .receipt()
        .graph_read_access_admission()
        .expect("executed graph read must carry its admitted access plan")
        .authority_receipt();

    assert_eq!(context_counters.canonical_query_identity_read_count(), 1);
    assert_eq!(context_counters.policy_tenant_admission_attempt_count(), 1);
    assert_eq!(context_counters.policy_tenant_admitted_count(), 1);
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
        executed_authority_receipt.policy_tenant_digest(),
        context_receipt.policy_tenant_admission_digest()
    );
    assert_eq!(
        executed_authority_receipt.relationship_proof_digest(),
        context_receipt.relationship_proof_admission_digest()
    );
    assert_eq!(
        executed_authority_receipt.digest(),
        context_receipt.graph_authority_admission_digest()
    );
}

#[test]
fn equivalent_relationship_declaration_order_converges_on_context_identity() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let first = run_relationship_context(
        request_execution,
        [
            WorthQueryReadRelationshipProof::direct_edge(manager_relation_name()),
            WorthQueryReadRelationshipProof::tenant_membership(),
        ],
    );
    let second = run_relationship_context(
        request_execution,
        [
            WorthQueryReadRelationshipProof::tenant_membership(),
            WorthQueryReadRelationshipProof::direct_edge(manager_relation_name()),
        ],
    );

    assert_eq!(first.context_receipt(), second.context_receipt());
    assert_eq!(
        first
            .context_receipt()
            .relationship_proof_admission_digest(),
        second
            .context_receipt()
            .relationship_proof_admission_digest()
    );
    assert_eq!(first.result(), second.result());
}

#[test]
fn denied_policy_stops_before_graph_authority_admission() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = declare(local_identity_read).expect("read declaration should canonicalize");
    let policy_tenant = admitted_policy_tenant_inputs(1, false);
    let context = current().under_policy_tenant(
        policy_tenant.policy,
        policy_tenant.tenant,
        policy_tenant.branch,
        policy_tenant.schema,
    );
    let mut workspace = read_runtime(request_execution)
        .workspace("ordinary-read-policy-denied")
        .expect("ordinary workspace should open");

    let stop = declaration
        .using(context)
        .run(request_execution, &mut workspace)
        .into_result()
        .expect_err("denied policy must stop the read");
    let counters = stop
        .context_denial()
        .expect("policy denial must remain a context denial")
        .counters();

    assert_eq!(
        stop.next_action(),
        WorthQueryReadNextAction::SupplyPolicyAuthority
    );
    assert_eq!(counters.canonical_query_identity_read_count(), 1);
    assert_eq!(counters.policy_tenant_admission_attempt_count(), 1);
    assert_eq!(counters.policy_tenant_admitted_count(), 0);
    assert_eq!(counters.relationship_proof_admission_attempt_count(), 0);
    assert_eq!(counters.relationship_proof_admitted_count(), 0);
    assert_eq!(counters.graph_authority_admission_attempt_count(), 0);
    assert_eq!(counters.graph_authority_admitted_count(), 0);
    assert_eq!(stop.journey_counters().planning_attempt_count(), 0);
    assert_eq!(stop.journey_counters().planning_completed_count(), 0);
    assert!(stop.context_receipt().is_none());
}

#[test]
fn denied_branch_names_the_branch_authority_as_the_next_action() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = declare(local_identity_read).expect("read declaration should canonicalize");
    let policy_tenant = admitted_policy_tenant_inputs(1, true);
    let branch = BranchAccessGrant::synthetic_denied(
        "main",
        "seeded branch authority denial",
        &policy_tenant.policy,
    );
    let context = current().under_policy_tenant(
        policy_tenant.policy,
        policy_tenant.tenant,
        branch,
        policy_tenant.schema,
    );
    let mut workspace = read_runtime(request_execution)
        .workspace("ordinary-read-branch-denied")
        .expect("ordinary workspace should open");

    let stop = declaration
        .using(context)
        .run(request_execution, &mut workspace)
        .into_result()
        .expect_err("denied branch must stop the read");

    assert_eq!(
        stop.next_action(),
        WorthQueryReadNextAction::SupplyBranchAuthority
    );
    assert_eq!(stop.journey_counters().planning_attempt_count(), 0);
    assert_eq!(
        stop.journey_counters()
            .lower_runtime_execution_attempt_count(),
        0
    );
}

#[test]
fn hidden_tenant_filter_names_tenant_authority_as_the_next_action() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = declare(local_identity_read).expect("read declaration should canonicalize");
    let policy_tenant = admitted_policy_tenant_inputs(1, true);
    let tenant = TenantBindingSnapshot::synthetic_hidden_filter(
        "tenant-a",
        "main",
        "schema-a",
        TenantBasisEpoch::Synthetic(7),
    );
    let context = current().under_policy_tenant(
        policy_tenant.policy,
        tenant,
        policy_tenant.branch,
        policy_tenant.schema,
    );
    let mut workspace = read_runtime(request_execution)
        .workspace("ordinary-read-tenant-denied")
        .expect("ordinary workspace should open");

    let stop = declaration
        .using(context)
        .run(request_execution, &mut workspace)
        .into_result()
        .expect_err("hidden tenant authority must stop the read");

    assert_eq!(
        stop.next_action(),
        WorthQueryReadNextAction::SupplyTenantAuthority
    );
    assert_eq!(stop.journey_counters().planning_attempt_count(), 0);
    assert_eq!(
        stop.journey_counters()
            .lower_runtime_execution_attempt_count(),
        0
    );
}

#[test]
fn relationship_read_without_relationship_authority_stops_before_other_admission() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = declare(local_manager_relationship_read)
        .expect("relationship read declaration should canonicalize");
    let mut workspace = read_runtime(request_execution)
        .workspace("ordinary-read-missing-relationship-authority")
        .expect("ordinary workspace should open");

    let stop = declaration
        .using(current())
        .run(request_execution, &mut workspace)
        .into_result()
        .expect_err("relationship read must require relationship authority");
    let counters = stop
        .context_denial()
        .expect("missing relationship authority must be a context denial")
        .counters();

    assert_eq!(
        stop.next_action(),
        WorthQueryReadNextAction::SupplyRelationshipProofAuthority
    );
    assert_eq!(counters.canonical_query_identity_read_count(), 1);
    assert_eq!(counters.policy_tenant_admission_attempt_count(), 0);
    assert_eq!(counters.relationship_proof_admission_attempt_count(), 0);
    assert_eq!(counters.graph_authority_admission_attempt_count(), 0);
    assert_eq!(stop.journey_counters().planning_attempt_count(), 0);
    assert_eq!(stop.journey_counters().planning_completed_count(), 0);
}

#[test]
fn changed_policy_epoch_invalidates_only_context_identity() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let first = run_policy_context(request_execution, 1);
    let second = run_policy_context(request_execution, 2);

    assert_ne!(first.digest(), second.digest());
    assert_eq!(
        first.canonical_query_digest(),
        second.canonical_query_digest()
    );
}
