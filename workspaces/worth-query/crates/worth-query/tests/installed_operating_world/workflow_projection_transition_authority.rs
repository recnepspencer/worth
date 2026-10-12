use std::sync::{atomic::AtomicU64, Arc};

use worth_query::facade::domain;

use super::installed_operation_fixture::{
    conditional_installation, operation_conditional_workflow_workspace_with,
    required_domain_workflow_workspace, stage_conditional_workflow_workspace_with, AuxiliaryDomain,
    GeometryDomain,
};
use super::workflow_projection_lifecycle::{
    conditional_node, promoted, settle_workflow, stage_conditional_node, LifecycleWorkflowCompute,
    RequestedTrigger,
};

#[test]
fn workflow_rebind_requires_every_required_domain_owner_receipt() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace =
        required_domain_workflow_workspace(request_execution, "workflow-required-domain-rebind")
            .unwrap();
    let prior_geometry = workspace.domain(GeometryDomain).unwrap();
    let prior_auxiliary = workspace.domain(AuxiliaryDomain).unwrap();
    let (settled, _) = settle_workflow(&mut workspace);
    let live = promoted(request_execution, settled, &mut workspace);

    workspace.advance_domain_installation_generation().unwrap();
    let geometry = workspace
        .rebind_domain(prior_geometry.rebind_request())
        .unwrap();
    let auxiliary = workspace
        .rebind_domain(prior_auxiliary.rebind_request())
        .unwrap();
    let (candidate, _) = settle_workflow(&mut workspace);
    let candidate = candidate.into_lifecycle();

    assert!(live
        .rebind_witness_for(&candidate, geometry.receipt().clone())
        .is_err());
    let witness = live
        .rebind_witness_for_with_required_domains(
            &candidate,
            geometry.receipt().clone(),
            vec![auxiliary.receipt().clone()],
        )
        .unwrap();
    let rebound = match live.rebind_with(request_execution, candidate, witness, &mut workspace) {
        domain::WorthQueryWorkflowProjectionRebindOutcome::Rebound(rebound) => rebound,
        _ => panic!("workflow required-domain rebind did not converge"),
    };
    assert!(
        rebound
            .rebind_witness()
            .counters()
            .required_domain_rebind_receipts_inspected
            > 0
    );
}

#[test]
fn workflow_replacement_mints_fresh_operation_and_publication_signals() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let operation_node = conditional_node("workflow-transition-operation");
    let mut operation_installation = conditional_installation(&operation_node);
    operation_installation.providers =
        worth_runtime_bridge::facade::BridgeConditionalProviderSet::new().trigger(RequestedTrigger);
    let operation_versions = Arc::new(AtomicU64::new(0));
    let mut operation_workspace = operation_conditional_workflow_workspace_with(
        request_execution,
        "workflow-transition-operation",
        operation_node,
        operation_installation,
        LifecycleWorkflowCompute(Arc::clone(&operation_versions)),
    )
    .unwrap();
    let (operation_settled, _) = settle_workflow(&mut operation_workspace);
    let operation_live = promoted(
        request_execution,
        operation_settled,
        &mut operation_workspace,
    );
    let prior_operation_signal = operation_live.conditional_provenance()[0]
        .signal_projection()
        .label()
        .to_string();
    let (operation_candidate, _) = settle_workflow(&mut operation_workspace);
    let operation_candidate = operation_candidate.into_lifecycle();
    let operation_witness = operation_live
        .replacement_witness_for(&operation_candidate)
        .unwrap();
    let operation_replaced = match operation_live.replace_with(
        request_execution,
        operation_candidate,
        operation_witness,
        &mut operation_workspace,
    ) {
        domain::WorthQueryWorkflowProjectionReplacementOutcome::Replaced(replaced) => replaced,
        _ => panic!("operation-conditional workflow replacement did not converge"),
    };
    assert_ne!(
        operation_replaced.conditional_provenance()[0]
            .signal_projection()
            .label()
            .as_ref(),
        prior_operation_signal.as_str()
    );

    let publication_node = stage_conditional_node(
        "workflow-transition-publication",
        domain::WorthQueryWorkflowValueContract::Projection,
    );
    let mut publication_installation = conditional_installation(&publication_node);
    publication_installation.providers =
        worth_runtime_bridge::facade::BridgeConditionalProviderSet::new().trigger(RequestedTrigger);
    let publication_versions = Arc::new(AtomicU64::new(0));
    let mut publication_workspace = stage_conditional_workflow_workspace_with(
        request_execution,
        "workflow-transition-publication",
        publication_node,
        "publish",
        publication_installation,
        LifecycleWorkflowCompute(Arc::clone(&publication_versions)),
    )
    .unwrap();
    let (publication_settled, _) = settle_workflow(&mut publication_workspace);
    let publication_live = promoted(
        request_execution,
        publication_settled,
        &mut publication_workspace,
    );
    let prior_publication_signal = publication_live.conditional_provenance()[0]
        .signal_projection()
        .label()
        .to_string();
    let (publication_candidate, _) = settle_workflow(&mut publication_workspace);
    let publication_candidate = publication_candidate.into_lifecycle();
    let publication_witness = publication_live
        .replacement_witness_for(&publication_candidate)
        .unwrap();
    let publication_replaced = match publication_live.replace_with(
        request_execution,
        publication_candidate,
        publication_witness,
        &mut publication_workspace,
    ) {
        domain::WorthQueryWorkflowProjectionReplacementOutcome::Replaced(replaced) => replaced,
        _ => panic!("publication-conditional workflow replacement did not converge"),
    };
    assert_ne!(
        publication_replaced.conditional_provenance()[0]
            .signal_projection()
            .label()
            .as_ref(),
        prior_publication_signal.as_str()
    );
}
