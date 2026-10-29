pub use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
pub use worth_foundational::facade::{FieldKey, InternedString};
pub use worth_query::facade::{domain, foundation};

#[path = "workflow_projection_lifecycle/projection_lifecycle_assertions.rs"]
pub mod projection_lifecycle_assertions;
pub mod providers;
pub use projection_lifecycle_assertions::assert_zero_lifecycle_work;
pub use providers::{LifecycleWorkflowCompute, RequestedTrigger};

pub use super::conditional_node_contract::{conditional_node_result, dependency, ManualRefresh};
pub use super::installed_operation_fixture::{
    conditional_installation, controlled_workflow_workspace,
    operation_conditional_workflow_workspace_with, stage_conditional_workflow_workspace_with,
    workflow_workspace, GeometryDomain, ReadFamily, WorkflowRead,
};

pub type SettledWorkflow = domain::WorthQuerySettledWorkflowProjection<
    GeometryDomain,
    WorkflowRead,
    ReadFamily,
    foundation::ObservationLaneWitness,
>;

pub fn settle_workflow(
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
            crate::facade::query::suite::installed_operation_fixture::execution_resource_request(),
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

pub fn promoted(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    settled: SettledWorkflow,
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
) -> domain::WorthQueryLiveBoundWorkflowProjection<
    GeometryDomain,
    WorkflowRead,
    ReadFamily,
    foundation::ObservationLaneWitness,
> {
    match settled
        .into_lifecycle()
        .promote(request_execution, workspace)
    {
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

pub fn conditional_node(identity: &str) -> domain::WorthQueryPortableConditionalNodeDeclaration {
    conditional_node_result(
        identity,
        dependency(domain::WorthQuerySemanticLocality::SourceRecord),
        domain::WorthQueryConditionalEvaluationCondition::on_demand(),
        domain::WorthQueryConditionalTrigger::on_demand::<ManualRefresh>(),
        domain::WorthQueryMaintenancePosture::OnDemandOnly,
    )
    .unwrap()
}

pub fn stage_conditional_node(
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
