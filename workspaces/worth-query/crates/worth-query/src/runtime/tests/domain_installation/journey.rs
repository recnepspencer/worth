use super::*;
use crate::authoring::{
    AspectFieldSelector, AuthoredResultShapeField, DetailQueryBuilder, DetailResultShapeBuilder,
};
use crate::domain_installation::{
    WorthQueryInstalledDomainCapabilityKind, WorthQueryInstalledDomainExecutionDriftKind,
    WorthQueryInstalledDomainExecutionNextAction,
};
use crate::ordinary::read::{
    current, project_facts, WorthQueryReadContextDenialSource, WorthQueryReadNextAction,
};
use crate::policy_basis::{
    BranchAccessGrant, PolicyEpoch, PolicyRuleSnapshot, PolicyTenantAdmissionFailureClass,
};
use crate::runtime::{WorthQueryAspectTouch, WorthQueryAuthoredAspectValue};
use crate::session_label::WorthQuerySessionLabel;
use crate::tenant_basis::{SchemaVariantSnapshot, TenantBasisEpoch, TenantBindingSnapshot};

#[test]
fn installed_read_projection_and_receipts_carry_one_authority_witness() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = installed_runtime(request_execution);
    let handle = runtime.domain(InstalledDomain).unwrap();
    let expected = handle.authority_witness().witness_identity().clone();
    let mut workspace = runtime.workspace("installed-domain-read").unwrap();

    let completion = handle
        .read(identity_read)
        .unwrap()
        .using(current())
        .run(request_execution, &mut workspace)
        .unwrap()
        .into_result()
        .unwrap();
    assert_eq!(
        completion.receipt().capability(),
        WorthQueryInstalledDomainCapabilityKind::Read
    );
    assert_eq!(
        completion
            .receipt()
            .installed_authority()
            .witness_identity(),
        &expected
    );

    let projection = completion.project(project_facts().entity_identities());
    assert_eq!(
        projection.receipt().capability(),
        WorthQueryInstalledDomainCapabilityKind::Projection
    );
    assert_eq!(
        projection
            .receipt()
            .installed_authority()
            .witness_identity(),
        &expected
    );
}

#[test]
fn foreign_workspace_denies_before_the_ordinary_read_journey_begins() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let owner = installed_runtime(request_execution);
    let handle = owner.domain(InstalledDomain).unwrap();
    let foreign = installed_runtime(request_execution);
    let mut workspace = foreign.workspace("foreign-installed-domain").unwrap();

    let stop = handle
        .read(identity_read)
        .unwrap()
        .using(current())
        .run(request_execution, &mut workspace)
        .unwrap_err();
    assert_eq!(
        stop.stop().kind(),
        WorthQueryInstalledDomainExecutionDriftKind::ForeignRuntime
    );
    assert_eq!(
        stop.installed_authority().witness_identity(),
        handle.authority_witness().witness_identity()
    );
    assert_eq!(
        stop.stop().next_action(),
        WorthQueryInstalledDomainExecutionNextAction::UseOwningRuntime
    );
    assert_eq!(stop.stop().counters().planning_attempts(), 0);
    assert_eq!(stop.stop().counters().lower_runtime_attempts(), 0);
    assert_eq!(stop.stop().counters().execution_attempts(), 0);
}

#[test]
fn installed_read_preserves_authority_across_an_ordinary_cross_basis_stop() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = installed_runtime(request_execution);
    let handle = runtime.domain(InstalledDomain).unwrap();
    let expected = handle.authority_witness().witness_identity().clone();
    let mut workspace = runtime.workspace("installed-domain-cross-basis").unwrap();
    let policy = PolicyRuleSnapshot::synthetic_authority_with_query_admission(
        "installed-domain-policy",
        "installed-domain-rules",
        PolicyEpoch::Synthetic(1),
        true,
    );
    let branch = BranchAccessGrant::synthetic_granted("main", &policy);
    let context = current().under_policy_tenant(
        policy,
        TenantBindingSnapshot::synthetic_direct(
            "tenant-a",
            "other-branch",
            "schema-a",
            TenantBasisEpoch::Synthetic(7),
        ),
        branch,
        SchemaVariantSnapshot::synthetic_authority("tenant-a", "schema-a", "exact"),
    );

    let stopped = handle
        .read(identity_read)
        .unwrap()
        .using(context)
        .run(request_execution, &mut workspace)
        .unwrap()
        .into_result();
    let Err(stop) = stopped else {
        panic!("cross-basis context must stop before planning")
    };

    assert_eq!(stop.installed_authority().witness_identity(), &expected);
    assert_eq!(
        stop.capability(),
        WorthQueryInstalledDomainCapabilityKind::Read
    );
    assert_eq!(
        stop.stop().next_action(),
        WorthQueryReadNextAction::SupplyFreshBasis
    );
    assert_eq!(stop.stop().journey_counters().planning_attempt_count(), 0);
    assert_eq!(
        stop.stop()
            .journey_counters()
            .lower_runtime_execution_attempt_count(),
        0
    );
}

#[test]
fn installed_read_preserves_authority_across_changed_policy_denial() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = installed_runtime(request_execution);
    let handle = runtime.domain(InstalledDomain).unwrap();
    let expected = handle.authority_witness().witness_identity().clone();
    let mut workspace = runtime
        .workspace("installed-domain-changed-policy")
        .unwrap();
    let prior_policy = PolicyRuleSnapshot::synthetic_authority_with_query_admission(
        "installed-domain-policy",
        "installed-domain-rules",
        PolicyEpoch::Synthetic(1),
        true,
    );
    let stale_branch = BranchAccessGrant::synthetic_granted("main", &prior_policy);
    let current_policy = PolicyRuleSnapshot::synthetic_authority_with_query_admission(
        "installed-domain-policy",
        "installed-domain-rules",
        PolicyEpoch::Synthetic(2),
        true,
    );
    let context = current().under_policy_tenant(
        current_policy,
        TenantBindingSnapshot::synthetic_direct(
            "tenant-a",
            "main",
            "schema-a",
            TenantBasisEpoch::Synthetic(7),
        ),
        stale_branch,
        SchemaVariantSnapshot::synthetic_authority("tenant-a", "schema-a", "exact"),
    );

    let stopped = match handle
        .read(identity_read)
        .unwrap()
        .using(context)
        .run(request_execution, &mut workspace)
        .unwrap()
        .into_result()
    {
        Err(stopped) => stopped,
        Ok(_) => panic!("changed policy authority must deny before planning"),
    };
    let denial = stopped
        .stop()
        .context_denial()
        .expect("changed policy must remain a contextual denial");
    let WorthQueryReadContextDenialSource::PolicyTenant(policy_denial) = denial.source() else {
        panic!("changed policy must deny at policy admission")
    };

    assert_eq!(stopped.installed_authority().witness_identity(), &expected);
    assert_eq!(
        policy_denial.failure_class(),
        PolicyTenantAdmissionFailureClass::StalePolicyAuthority
    );
    assert_eq!(
        stopped.stop().next_action(),
        WorthQueryReadNextAction::SupplyPolicyAuthority
    );
    assert_eq!(
        stopped.stop().journey_counters().planning_attempt_count(),
        0
    );
    assert_eq!(
        stopped
            .stop()
            .journey_counters()
            .lower_runtime_execution_attempt_count(),
        0
    );
}

#[test]
fn installed_mutation_and_workflow_receipts_retain_the_package_witness() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = installed_runtime(request_execution);
    let handle = runtime.domain(InstalledDomain).unwrap();
    let expected = handle.authority_witness().witness_identity().clone();
    let mut workspace = runtime.workspace("installed-domain-mutation").unwrap();
    let context = crate::ordinary::mutation::authoritative(request_execution, &workspace).unwrap();
    let mutation = handle
        .mutation(|builder| {
            builder
                .set_aspect(
                    WorthQueryAspectTouch::from_authoring_ingress_text("identity.id")?,
                    WorthQueryAuthoredAspectValue::string("installed-task"),
                )
                .build_insert("Task")
        })
        .unwrap()
        .using(context)
        .run(request_execution, &mut workspace)
        .unwrap();
    let crate::domain_installation::WorthQueryInstalledDomainMutationOutcome::Completed(mutation) =
        mutation
    else {
        panic!("installed mutation must complete")
    };
    assert_eq!(
        mutation.receipt().installed_authority().witness_identity(),
        &expected
    );

    let label = WorthQuerySessionLabel::scoped_strs("installed-domain", ["workflow"]).unwrap();
    let context =
        crate::ordinary::workflow::preview(request_execution, &workspace, label.clone()).unwrap();
    let workflow = handle
        .mutation(|builder| {
            builder
                .set_aspect(
                    WorthQueryAspectTouch::from_authoring_ingress_text("identity.id")?,
                    WorthQueryAuthoredAspectValue::string("installed-preview-task"),
                )
                .build_insert("Task")
        })
        .unwrap()
        .workflow(label)
        .using(context)
        .run(request_execution, &mut workspace)
        .unwrap();
    let crate::domain_installation::WorthQueryInstalledDomainWorkflowOutcome::Completed(workflow) =
        workflow
    else {
        panic!("installed workflow must complete")
    };
    assert_eq!(
        workflow.receipt().installed_authority().witness_identity(),
        &expected
    );
}

#[test]
fn installed_operational_and_rich_inspection_share_operational_evidence() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = installed_runtime(request_execution);
    let handle = runtime.domain(InstalledDomain).unwrap();
    let expected = handle.authority_witness().witness_identity().clone();
    let mut workspace = runtime.workspace("installed-domain-inspection").unwrap();
    let operational_source = handle
        .read(identity_read)
        .unwrap()
        .using(current())
        .run(request_execution, &mut workspace)
        .unwrap()
        .into_result()
        .unwrap();
    let rich_source = handle
        .read(identity_read)
        .unwrap()
        .using(current())
        .run(request_execution, &mut workspace)
        .unwrap()
        .into_result()
        .unwrap();
    let basis = crate::basis_lifecycle::basis_lifecycle()
        .historical_snapshot("installed-domain-inspection", true)
        .inspect()
        .unwrap();
    let operational = operational_source
        .inspect()
        .using(crate::ordinary::inspection::inspection_basis(basis.clone()))
        .run(&workspace)
        .unwrap();
    let rich = rich_source
        .inspect()
        .with_rich_inspection()
        .using(crate::ordinary::inspection::inspection_basis(basis))
        .run(&workspace)
        .unwrap();
    assert_eq!(
        operational
            .receipt()
            .installed_authority()
            .witness_identity(),
        &expected
    );
    assert_eq!(
        operational.outcome().settled().unwrap().receipt(),
        rich.outcome().settled().unwrap().receipt()
    );
    assert!(operational
        .outcome()
        .settled()
        .unwrap()
        .materialization()
        .is_none());
    assert!(rich
        .outcome()
        .settled()
        .unwrap()
        .materialization()
        .is_some());
}

fn identity_read<Output>(
    read: WorthQueryReadBuilder<Output>,
) -> Result<Output, crate::runtime::WorthQueryReadDenial> {
    read.local_detail(
        "user",
        schema(),
        |query: DetailQueryBuilder| {
            query.project(AspectFieldSelector::new("identity", "id").unwrap())
        },
        |shape: DetailResultShapeBuilder| {
            shape.field(AuthoredResultShapeField::new("identity", "id", "id").unwrap())
        },
    )
}
