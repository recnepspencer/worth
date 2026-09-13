use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use worth_foundational::facade::{FieldKey, InternedString};
use worth_query::facade::{domain, foundation};

#[path = "workflow_projection_lifecycle/projection_lifecycle_assertions.rs"]
mod projection_lifecycle_assertions;
mod providers;
use projection_lifecycle_assertions::assert_zero_lifecycle_work;
pub(super) use providers::{LifecycleWorkflowCompute, RequestedTrigger};

use super::conditional_node_contract::{conditional_node_result, dependency, ManualRefresh};
use super::installed_operation_fixture::{
    conditional_installation, controlled_workflow_workspace,
    operation_conditional_workflow_workspace_with, stage_conditional_workflow_workspace_with,
    workflow_workspace, GeometryDomain, ReadFamily, WorkflowRead,
};

type SettledWorkflow = domain::WorthQuerySettledWorkflowProjection<
    GeometryDomain,
    WorkflowRead,
    ReadFamily,
    foundation::ObservationLaneWitness,
>;

#[test]
fn workflow_promotion_preserves_settled_authority_native_plan_and_trace() {
    let mut workspace = workflow_workspace("workflow-lifecycle-parity").unwrap();
    let (settled, key) = settle_workflow(&mut workspace);
    let settled_identity = settled.identity().to_string();
    let trace_identity = settled.trace().identity().to_string();
    let source_identity = settled.authority().source_identity().as_str().to_string();
    let authority_contract = settled.authority().consumer_contract().clone();
    let current = settled.into_lifecycle();
    let predecessor = current.identity().to_string();

    let domain::WorthQueryWorkflowProjectionPromotionOutcome::Promoted(live) =
        current.promote(&mut workspace)
    else {
        panic!("settled workflow projection did not promote")
    };

    assert_eq!(live.predecessor_identity(), predecessor);
    assert_eq!(live.receipt().settled_identity(), settled_identity);
    assert_eq!(live.snapshot().trace().identity(), trace_identity);
    assert_eq!(
        live.snapshot().authority().source_identity().as_str(),
        source_identity
    );
    assert_eq!(
        live.snapshot().authority().consumer_contract(),
        &authority_contract
    );
    let native = live.snapshot().native_value(&key, 0).unwrap();
    assert_eq!(
        native.fact().as_interned_string(),
        Ok(&InternedString::Raw("synthetic-anchor".into()))
    );
    assert_eq!(native.counters().indexed_accesses, 1);
    assert_eq!(native.counters().fact_scans, 0);
    let refreshed = live.refresh(&mut workspace).unwrap();
    assert_eq!(
        refreshed.authority().consumer_contract(),
        &authority_contract
    );
    let refreshed_native = refreshed.native_value(&key, 0).unwrap();
    assert_eq!(
        refreshed_native.fact().as_interned_string(),
        Ok(&InternedString::Raw("synthetic-anchor".into()))
    );
    assert_eq!(refreshed_native.counters().indexed_accesses, 1);
    assert_eq!(refreshed_native.counters().fact_scans, 0);
    assert_eq!(refreshed_native.counters().row_scans, 0);
    assert_eq!(refreshed_native.counters().path_parses, 0);
    assert_eq!(refreshed_native.counters().view_registry_inspections, 0);
    assert_eq!(refreshed_native.counters().domain_registry_inspections, 0);
    assert_eq!(refreshed.work().native_rebind_calls(), 1);
    assert_eq!(live.receipt().counters().lifecycle_attempts, 1);
    assert_eq!(live.receipt().counters().planning_attempts, 1);
    assert_eq!(live.receipt().counters().lower_runtime_contacts, 1);
    assert_eq!(live.receipt().counters().managed_resource_registrations, 1);
}

#[test]
fn workflow_promotion_denies_foreign_and_stale_authority_before_lifecycle_work() {
    let mut owner = workflow_workspace("workflow-lifecycle-owner").unwrap();
    let (settled, _) = settle_workflow(&mut owner);
    let mut foreign = workflow_workspace("workflow-lifecycle-foreign").unwrap();
    let domain::WorthQueryWorkflowProjectionPromotionOutcome::Denied(stop) =
        settled.into_lifecycle().promote(&mut foreign)
    else {
        panic!("foreign runtime did not deny workflow promotion")
    };
    assert_eq!(
        stop.kind(),
        domain::WorthQueryProjectionPromotionDenialKind::ForeignRuntime
    );
    assert_zero_lifecycle_work(stop.counters());
    assert!(matches!(
        stop.into_current().promote(&mut owner),
        domain::WorthQueryWorkflowProjectionPromotionOutcome::Promoted(_)
    ));

    let mut controlled = controlled_workflow_workspace("workflow-lifecycle-stale").unwrap();
    let (settled, _) = settle_workflow(&mut controlled);
    controlled.advance_domain_installation_generation().unwrap();
    let domain::WorthQueryWorkflowProjectionPromotionOutcome::Stale(stale) =
        settled.into_lifecycle().promote(&mut controlled)
    else {
        panic!("stale workflow generation did not retain stale-readable state")
    };
    assert_zero_lifecycle_work(stale.counters());
    assert!(!stale.snapshot().trace().identity().is_empty());

    let mut live_controlled =
        controlled_workflow_workspace("workflow-lifecycle-live-stale").unwrap();
    let (settled, _) = settle_workflow(&mut live_controlled);
    let live = promoted(settled, &mut live_controlled);
    live_controlled
        .advance_domain_installation_generation()
        .unwrap();
    let stop = match live.refresh(&mut live_controlled) {
        Err(domain::WorthQueryLiveProjectionRefreshError::Authority(stop)) => stop,
        _ => panic!("stale workflow live projection reached maintenance or delivery"),
    };
    assert_eq!(
        stop.kind(),
        domain::WorthQueryDomainHandleDenialKind::StaleInstallationGeneration
    );
    assert_eq!(stop.work().authority_checks(), 1);
    assert_eq!(stop.work().drain_calls(), 0);
    assert_eq!(stop.work().delivery_batches(), 0);
    assert_eq!(stop.work().maintenance_batches(), 0);
    assert_eq!(stop.work().read_calls(), 0);
    assert_eq!(stop.work().projection_calls(), 0);
    assert_eq!(stop.work().native_rebind_calls(), 0);
}

#[test]
fn workflow_promotion_reevaluates_only_operation_and_publication_conditionals() {
    let operation_node = conditional_node("workflow-lifecycle-operation");
    let mut installation = conditional_installation(&operation_node);
    installation.providers =
        worth_runtime_bridge::facade::BridgeConditionalProviderSet::new().trigger(RequestedTrigger);
    let operation_versions = Arc::new(AtomicU64::new(0));
    let mut operation_workspace = operation_conditional_workflow_workspace_with(
        "workflow-lifecycle-operation",
        operation_node,
        installation,
        LifecycleWorkflowCompute(Arc::clone(&operation_versions)),
    )
    .unwrap();
    let (operation_settled, _) = settle_workflow(&mut operation_workspace);
    let prior_operation_signal = operation_settled.trace().operation_conditional_provenance()[0]
        .signal_projection()
        .label()
        .to_string();
    let operation_live = promoted(operation_settled, &mut operation_workspace);
    assert_eq!(operation_live.conditional_provenance().len(), 1);
    assert_ne!(
        operation_live.conditional_provenance()[0]
            .signal_projection()
            .label()
            .as_ref(),
        prior_operation_signal.as_str()
    );

    let publication_node = stage_conditional_node(
        "workflow-lifecycle-publication",
        domain::WorthQueryWorkflowValueContract::Projection,
    );
    let mut publication_installation = conditional_installation(&publication_node);
    publication_installation.providers =
        worth_runtime_bridge::facade::BridgeConditionalProviderSet::new().trigger(RequestedTrigger);
    let publication_versions = Arc::new(AtomicU64::new(0));
    let mut publication_workspace = stage_conditional_workflow_workspace_with(
        "workflow-lifecycle-publication",
        publication_node,
        "publish",
        publication_installation,
        LifecycleWorkflowCompute(Arc::clone(&publication_versions)),
    )
    .unwrap();
    let (publication_settled, _) = settle_workflow(&mut publication_workspace);
    let prior_publication_signal = publication_settled
        .trace()
        .stage_receipts()
        .iter()
        .find(|receipt| receipt.stage_identity() == "publish")
        .unwrap()
        .conditional_provenance()[0]
        .signal_projection()
        .label()
        .to_string();
    let publication_live = promoted(publication_settled, &mut publication_workspace);
    assert_eq!(publication_live.conditional_provenance().len(), 1);
    assert_ne!(
        publication_live.conditional_provenance()[0]
            .signal_projection()
            .label()
            .as_ref(),
        prior_publication_signal.as_str()
    );

    assert_eq!(
        publication_live
            .receipt()
            .counters()
            .conditional_lowerings_checked,
        1
    );
}

pub(super) fn settle_workflow(
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
) -> (SettledWorkflow, domain::WorthQueryNativeAccessKey) {
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, WorkflowRead)
        .unwrap();
    let mut builder = bound
        .consumer_projection_contract()
        .unwrap()
        .projection_request();
    let selection = builder
        .select_display_native_field(FieldKey::new("id").unwrap())
        .unwrap();
    let request = builder.build().unwrap();
    let key = request.resolve_native_key(&selection).unwrap().into_key();
    let run = bound
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &*workspace,
        )
        .unwrap()
        .start_workflow(workspace)
        .unwrap();
    let trace = run
        .advance(
            "start",
            domain::WorthQueryWorkflowValue::NotRequired,
            workspace,
        )
        .unwrap()
        .advance(
            "left",
            domain::WorthQueryWorkflowValue::Text("start".into()),
            workspace,
        )
        .unwrap()
        .advance(
            "right",
            domain::WorthQueryWorkflowValue::Text("start".into()),
            workspace,
        )
        .unwrap()
        .advance(
            "publish",
            domain::WorthQueryWorkflowValue::Text("join".into()),
            workspace,
        )
        .unwrap()
        .complete()
        .unwrap();
    let settled = trace
        .publish()
        .unwrap()
        .consume_bound(request)
        .unwrap()
        .settle()
        .unwrap();
    (settled, key)
}

pub(super) fn promoted(
    settled: SettledWorkflow,
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
) -> domain::WorthQueryLiveBoundWorkflowProjection<
    GeometryDomain,
    WorkflowRead,
    ReadFamily,
    foundation::ObservationLaneWitness,
> {
    match settled.into_lifecycle().promote(workspace) {
        domain::WorthQueryWorkflowProjectionPromotionOutcome::Promoted(live) => live,
        domain::WorthQueryWorkflowProjectionPromotionOutcome::Deferred(stop) => panic!(
            "workflow lifecycle promotion deferred: {:?} {} {:?}",
            stop.kind(),
            stop.detail(),
            stop.counters()
        ),
        domain::WorthQueryWorkflowProjectionPromotionOutcome::Denied(stop)
        | domain::WorthQueryWorkflowProjectionPromotionOutcome::Failed(stop) => panic!(
            "workflow lifecycle promotion stopped: {:?} {} {:?}",
            stop.kind(),
            stop.detail(),
            stop.counters()
        ),
        domain::WorthQueryWorkflowProjectionPromotionOutcome::Stale(_) => {
            panic!("workflow lifecycle promotion became stale")
        }
        domain::WorthQueryWorkflowProjectionPromotionOutcome::RebindRequired(_) => {
            panic!("workflow lifecycle promotion required rebind")
        }
        domain::WorthQueryWorkflowProjectionPromotionOutcome::AuthorityRevalidationRequired(_) => {
            panic!("workflow lifecycle promotion required authority revalidation")
        }
    }
}

pub(super) fn conditional_node(
    identity: &str,
) -> domain::WorthQueryPortableConditionalNodeDeclaration {
    conditional_node_result(
        identity,
        dependency(domain::WorthQuerySemanticLocality::SourceRecord),
        domain::WorthQueryConditionalEvaluationCondition::on_demand(),
        domain::WorthQueryConditionalTrigger::on_demand::<ManualRefresh>(),
        domain::WorthQueryMaintenancePosture::OnDemandOnly,
    )
    .unwrap()
}

pub(super) fn stage_conditional_node(
    identity: &str,
    contract: domain::WorthQueryWorkflowValueContract,
) -> domain::WorthQueryPortableConditionalNodeDeclaration {
    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    domain::WorthQueryPortableConditionalNodeDeclaration::declare(
        identity,
        domain::WorthQueryConditionalNodeRole::WorkflowStage,
    )
    .dependencies([dependency])
    .outputs([domain::WorthQueryConditionalNodeOutput::WorkflowStageOutput { contract }])
    .required_context([domain::WorthQueryConditionalNodeContext::WorkflowRun])
    .evaluation(
        domain::WorthQueryConditionalEvaluationCondition::on_demand(),
        domain::WorthQueryConditionalTrigger::on_demand::<ManualRefresh>(),
    )
    .comparison(
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue,
    )
    .artifact_policy(
        domain::WorthQueryArtifactReuseEquivalence::NotReusable,
        domain::WorthQueryMaintenancePosture::OnDemandOnly,
        domain::WorthQueryArtifactPosture::Ephemeral,
    )
    .output_relationship(domain::WorthQueryOutputRelationship::IsWorkflowStageOutput)
    .finish()
    .unwrap()
}
