use super::super::super::support::*;
use super::fixtures::{
    admitted_policy_tenant_inputs, bounded_descendant_manager_read, local_identity_read,
    local_manager_relationship_read,
};
use crate::authoring::RelationName;
use crate::ordinary::read::{
    current, declare, WorthQueryReadContextDenialSource, WorthQueryReadNextAction,
    WorthQueryReadRelationshipProof, WorthQueryReadRelationshipProofs,
};
use crate::policy_basis::{
    BranchAccessGrant, PolicyEpoch, PolicyRuleSnapshot, PolicyTenantAdmissionFailureClass,
};
use crate::relationship_proof::RelationshipProofFailureClass;
use crate::tenant_basis::{SchemaVariantSnapshot, TenantBasisEpoch, TenantBindingSnapshot};

#[test]
fn cross_basis_context_denies_before_planning_or_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let inputs = admitted_policy_tenant_inputs(1, true);
    let tenant = TenantBindingSnapshot::synthetic_direct(
        "tenant-a",
        "other-branch",
        "schema-a",
        TenantBasisEpoch::Synthetic(7),
    );
    let stop = run_identity_with_context(
        request_execution,
        current().under_policy_tenant(inputs.policy, tenant, inputs.branch, inputs.schema),
    );

    assert_policy_failure(&stop, PolicyTenantAdmissionFailureClass::BasisMismatch);
    assert_eq!(
        stop.next_action(),
        WorthQueryReadNextAction::SupplyFreshBasis
    );
    assert_no_successor(&stop);
}

#[test]
fn stale_policy_authority_denies_before_planning_or_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let old_policy = policy(1);
    let stale_branch = BranchAccessGrant::synthetic_granted("main", &old_policy);
    let stop = run_identity_with_context(
        request_execution,
        current().under_policy_tenant(
            policy(2),
            tenant("tenant-a", "main", 7),
            stale_branch,
            schema("tenant-a"),
        ),
    );

    assert_policy_failure(
        &stop,
        PolicyTenantAdmissionFailureClass::StalePolicyAuthority,
    );
    assert_eq!(
        stop.next_action(),
        WorthQueryReadNextAction::SupplyPolicyAuthority
    );
    assert_no_successor(&stop);
}

#[test]
fn cross_tenant_context_denies_before_planning_or_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let current_policy = policy(1);
    let branch = BranchAccessGrant::synthetic_granted("main", &current_policy);
    let stop = run_identity_with_context(
        request_execution,
        current().under_policy_tenant(
            current_policy,
            tenant("tenant-b", "main", 7),
            branch,
            schema("tenant-a"),
        ),
    );

    assert_policy_failure(&stop, PolicyTenantAdmissionFailureClass::CrossTenant);
    assert_eq!(
        stop.next_action(),
        WorthQueryReadNextAction::SupplyTenantAuthority
    );
    assert_no_successor(&stop);
}

#[test]
fn relationship_proof_for_another_query_denies_without_a_successor() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = declare(local_manager_relationship_read)
        .expect("relationship declaration should canonicalize");
    let inputs = admitted_policy_tenant_inputs(1, true);
    let proofs = WorthQueryReadRelationshipProofs::bounded(
        [WorthQueryReadRelationshipProof::direct_edge(
            RelationName::new("owner").expect("proof relation should author"),
        )],
        1,
        1,
    )
    .expect("bounded proof should declare");
    let context = current()
        .under_policy_tenant(inputs.policy, inputs.tenant, inputs.branch, inputs.schema)
        .with_relationship_proofs(proofs);
    let mut workspace = read_runtime(request_execution)
        .workspace("ordinary-read-mismatched-relationship-proof")
        .expect("workspace should open");
    let stop = declaration
        .using(context)
        .run(request_execution, &mut workspace)
        .into_result()
        .expect_err("proof for another relationship must deny");

    let denial = stop
        .context_denial()
        .expect("denial must remain contextual");
    let WorthQueryReadContextDenialSource::RelationshipProof(error) = denial.source() else {
        panic!(
            "expected relationship-proof denial, got {:?}",
            denial.source()
        );
    };
    assert_eq!(
        error.failure_class(),
        RelationshipProofFailureClass::QueryShapeMismatch
    );
    assert_eq!(
        stop.next_action(),
        WorthQueryReadNextAction::SupplyRelationshipProofAuthority
    );
    assert_no_successor(&stop);
}

#[test]
fn ancestor_proof_cannot_authorize_a_descendant_read() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = declare(bounded_descendant_manager_read)
        .expect("descendant declaration should canonicalize");
    let inputs = admitted_policy_tenant_inputs(1, true);
    let proofs = WorthQueryReadRelationshipProofs::bounded(
        [WorthQueryReadRelationshipProof::bounded_ancestor(
            RelationName::new("manager").expect("proof relation should author"),
            crate::ordinary::read::WorthQueryReadRelationshipDepth::new(2)
                .expect("proof depth should be bounded"),
        )],
        1,
        2,
    )
    .expect("bounded proof should declare");
    let context = current()
        .under_policy_tenant(inputs.policy, inputs.tenant, inputs.branch, inputs.schema)
        .with_relationship_proofs(proofs);
    let mut workspace = read_runtime(request_execution)
        .workspace("ordinary-read-wrong-direction-proof")
        .expect("workspace should open");
    let stop = declaration
        .using(context)
        .run(request_execution, &mut workspace)
        .into_result()
        .expect_err("ancestor authority must not authorize descendant traversal");

    let denial = stop
        .context_denial()
        .expect("denial must remain contextual");
    let WorthQueryReadContextDenialSource::RelationshipProof(error) = denial.source() else {
        panic!(
            "expected relationship-proof denial, got {:?}",
            denial.source()
        );
    };
    assert_eq!(
        error.failure_class(),
        RelationshipProofFailureClass::QueryShapeMismatch
    );
    assert_eq!(
        stop.next_action(),
        WorthQueryReadNextAction::SupplyRelationshipProofAuthority
    );
    assert_no_successor(&stop);
}

#[test]
fn changed_tenant_basis_generation_invalidates_only_that_context() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let generation_seven = run_tenant_generation(request_execution, 7);
    let repeated_generation_seven = run_tenant_generation(request_execution, 7);
    let generation_eight = run_tenant_generation(request_execution, 8);

    assert_eq!(generation_seven, repeated_generation_seven);
    assert_eq!(
        generation_seven.canonical_query_digest(),
        generation_eight.canonical_query_digest()
    );
    assert_ne!(generation_seven, generation_eight);
    assert_ne!(
        generation_seven.policy_tenant_admission_digest(),
        generation_eight.policy_tenant_admission_digest()
    );
}

fn run_identity_with_context(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    context: crate::ordinary::read::WorthQueryCurrentPolicyTenantReadContext,
) -> crate::ordinary::read::WorthQueryReadStop {
    let declaration = declare(local_identity_read).expect("declaration should canonicalize");
    let mut workspace = read_runtime(request_execution)
        .workspace("ordinary-read-hostile-context")
        .expect("workspace should open");
    declaration
        .using(context)
        .run(request_execution, &mut workspace)
        .into_result()
        .expect_err("hostile context must deny")
}

fn run_tenant_generation(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    generation: u64,
) -> crate::ordinary::read::WorthQueryReadContextReceipt {
    let current_policy = policy(1);
    let branch = BranchAccessGrant::synthetic_granted("main", &current_policy);
    let context = current().under_policy_tenant(
        current_policy,
        tenant("tenant-a", "main", generation),
        branch,
        schema("tenant-a"),
    );
    let declaration = declare(local_identity_read).expect("declaration should canonicalize");
    let mut workspace = read_runtime(request_execution)
        .workspace(format!("ordinary-read-generation-{generation}"))
        .expect("workspace should open");
    declaration
        .using(context)
        .run(request_execution, &mut workspace)
        .into_result()
        .expect("fresh context should execute")
        .context_receipt()
        .clone()
}

fn policy(epoch: u64) -> PolicyRuleSnapshot {
    PolicyRuleSnapshot::synthetic_authority_with_query_admission(
        "ordinary-policy",
        "ordinary-rules",
        PolicyEpoch::Synthetic(epoch),
        true,
    )
}

fn tenant(identity: &str, branch: &str, generation: u64) -> TenantBindingSnapshot {
    TenantBindingSnapshot::synthetic_direct(
        identity,
        branch,
        "schema-a",
        TenantBasisEpoch::Synthetic(generation),
    )
}

fn schema(tenant: &str) -> SchemaVariantSnapshot {
    SchemaVariantSnapshot::synthetic_authority(tenant, "schema-a", "exact")
}

fn assert_policy_failure(
    stop: &crate::ordinary::read::WorthQueryReadStop,
    expected: PolicyTenantAdmissionFailureClass,
) {
    let denial = stop
        .context_denial()
        .expect("denial must remain contextual");
    let WorthQueryReadContextDenialSource::PolicyTenant(error) = denial.source() else {
        panic!("expected policy/tenant denial, got {:?}", denial.source());
    };
    assert_eq!(error.failure_class(), expected);
}

fn assert_no_successor(stop: &crate::ordinary::read::WorthQueryReadStop) {
    assert!(stop.context_receipt().is_none());
    assert_eq!(stop.journey_counters().planning_attempt_count(), 0);
    assert_eq!(
        stop.journey_counters()
            .lower_runtime_execution_attempt_count(),
        0
    );
}
