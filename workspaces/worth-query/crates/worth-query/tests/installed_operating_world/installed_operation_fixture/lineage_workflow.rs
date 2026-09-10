use std::sync::{Arc, OnceLock};
use worth_query::facade::{domain, runtime};

mod executor;

use super::workflow::valid_stages;
use super::workflow_parallel_providers::WorkflowParallelProvider;
use super::{
    canonical_bundle, canonical_collection_bundle, configured_runtime_without_executors,
    semantic_closure, GeometryDomain, ReadFamily, WorkflowRead,
};
use executor::LineageWorkflowStageExecutor;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineageEvidenceScenario {
    PreservedIdentity,
    SingularSuccessor,
    SplitSuccessors,
    MergeSuccessor,
    GeneratedIdentity,
    RetiredIdentity,
    AdvisoryCorrespondence,
    AmbiguousCorrespondence,
    ContinuityBreak,
    MutationWithoutLineage,
}

pub fn lineage_workflow_workspace(
    name: &str,
    lineage: domain::WorthQueryOperationLineageContract,
    promotion_on_reference: bool,
    scenarios: Vec<LineageEvidenceScenario>,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    lineage_workflow_workspace_with_grouping(
        name,
        lineage,
        promotion_on_reference,
        scenarios,
        domain::WorthQueryOperationGroupingContract::Ungrouped,
    )
}

pub(crate) fn lineage_invalidation_workspace(
    name: &str,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    let target_identity = Arc::new(OnceLock::new());
    let operation = lineage_workflow_operation(
        domain::WorthQueryPortableWorkflowDefinition::new("start", lineage_stages()),
        domain::WorthQueryOperationLineageContract::Preserve,
        false,
        canonical_collection_bundle("Vertex"),
        lineage_collection_contract(domain::WorthQueryOperationGroupingContract::Ungrouped),
    );
    let package = super::package(false, false).operation(operation);
    let mut workspace = super::configured_base_runtime_for_package(package)
        .replayable_workflow_stage_executor(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            LineageWorkflowStageExecutor::new(
                vec![LineageEvidenceScenario::PreservedIdentity],
                Arc::clone(&target_identity),
            ),
        )
        .workflow_parallel_admission_provider(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            WorkflowParallelProvider,
        )
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Sharing,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Invalidation,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::DependencyImpact,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .workspace(name)?;
    let seed = workspace
        .insert("Vertex", |vertex| {
            vertex.aspect("identity.id", "lineage-authoritative-target")
        })
        .expect("lineage invalidation target inserts through Query mutation authority");
    target_identity
        .set(seed.deltas()[0].entity_identity().clone())
        .expect("lineage invalidation target identity is initialized once");
    Ok(workspace)
}

fn lineage_workflow_workspace_with_grouping(
    name: &str,
    lineage: domain::WorthQueryOperationLineageContract,
    promotion_on_reference: bool,
    scenarios: Vec<LineageEvidenceScenario>,
    grouping: domain::WorthQueryOperationGroupingContract,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    let target_identity = Arc::new(OnceLock::new());
    let mut workspace = configured_runtime_without_executors(lineage_workflow_package(
        domain::WorthQueryPortableWorkflowDefinition::new("start", lineage_stages()),
        lineage,
        promotion_on_reference,
        grouping,
    ))
    .replayable_workflow_stage_executor(
        GeometryDomain,
        WorkflowRead,
        ReadFamily,
        LineageWorkflowStageExecutor::new(scenarios, Arc::clone(&target_identity)),
    )
    .workflow_parallel_admission_provider(
        GeometryDomain,
        WorkflowRead,
        ReadFamily,
        WorkflowParallelProvider,
    )
    .workspace(name)?;
    let seed = workspace
        .insert("Vertex", |vertex| {
            vertex.aspect("identity.id", "lineage-authoritative-target")
        })
        .expect("lineage fixture target inserts through Query mutation authority");
    target_identity
        .set(seed.deltas()[0].entity_identity().clone())
        .expect("lineage fixture target identity is initialized once");
    Ok(workspace)
}

pub(super) fn lineage_workflow_package(
    workflow: domain::WorthQueryPortableWorkflowDefinition,
    lineage: domain::WorthQueryOperationLineageContract,
    promotion_on_reference: bool,
    grouping: domain::WorthQueryOperationGroupingContract,
) -> domain::WorthQueryDomainPackage<GeometryDomain> {
    configured_lineage_workflow_package(
        workflow,
        lineage,
        promotion_on_reference,
        canonical_collection_bundle("Vertex"),
        lineage_collection_contract(grouping),
    )
}

fn lineage_collection_contract(
    grouping: domain::WorthQueryOperationGroupingContract,
) -> domain::WorthQueryOperationCollectionContract {
    let identity_field = domain::WorthQueryOperationCollectionField::from_dotted("identity.id")
        .expect("valid collection field");
    domain::WorthQueryOperationCollectionContract::Collection {
        row_identity_field: identity_field.clone(),
        ordering_fields: vec![identity_field],
        grouping,
        window: domain::WorthQueryOperationWindowPolicy::CompleteCollection,
        continuation: domain::WorthQueryOperationContinuationPosture::NotRequired,
    }
}

pub(super) fn deferred_lineage_workflow_package(
    workflow: domain::WorthQueryPortableWorkflowDefinition,
) -> domain::WorthQueryDomainPackage<GeometryDomain> {
    configured_lineage_workflow_package(
        workflow,
        domain::WorthQueryOperationLineageContract::Evolve,
        false,
        canonical_bundle("Vertex"),
        domain::WorthQueryOperationCollectionContract::NotCollection,
    )
}

fn configured_lineage_workflow_package(
    workflow: domain::WorthQueryPortableWorkflowDefinition,
    lineage: domain::WorthQueryOperationLineageContract,
    promotion_on_reference: bool,
    canonical_query: worth_query_declaration::facade::canonicalization::CanonicalQueryBundle,
    collection: domain::WorthQueryOperationCollectionContract,
) -> domain::WorthQueryDomainPackage<GeometryDomain> {
    let operation = lineage_workflow_operation(
        workflow,
        lineage,
        promotion_on_reference,
        canonical_query,
        collection,
    );
    domain::WorthQueryDomainPackage::declare(
        GeometryDomain,
        domain::WorthQueryDomainIdentityDeclaration::new(
            domain::WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
            domain::WorthQueryDomainIdentityName::new("geometry").unwrap(),
            domain::WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
    .operation(operation)
}

fn lineage_workflow_operation(
    workflow: domain::WorthQueryPortableWorkflowDefinition,
    lineage: domain::WorthQueryOperationLineageContract,
    promotion_on_reference: bool,
    canonical_query: worth_query_declaration::facade::canonicalization::CanonicalQueryBundle,
    collection: domain::WorthQueryOperationCollectionContract,
) -> domain::WorthQueryDomainOperationDefinition<GeometryDomain, WorkflowRead, ReadFamily> {
    let mut semantics = semantic_closure(
        canonical_query,
        domain::WorthQuerySupportRequirement::Required,
        true,
    );
    semantics.lineage = lineage;
    semantics.collection = collection;
    semantics.effects = domain::WorthQueryOperationEffectContract::Declared {
        effect_families: vec![domain::WorthQueryOperationEffectFamily::Mutation],
    };
    semantics.aftermath = None;
    semantics.promotion = if promotion_on_reference {
        domain::WorthQueryOperationPromotionContract::OnDurableReference
    } else {
        domain::WorthQueryOperationPromotionContract::NotRequired
    };
    semantics.replay = domain::WorthQueryOperationReplayContract::CertReplayable {
        comparator: domain::WorthQueryOperationReplayComparatorContract::new(
            "installed-workflow-exact-v1",
        )
        .expect("static replay comparator family is portable"),
    };
    semantics.workflow = domain::WorthQueryOperationWorkflowContract::Declared(workflow);
    domain::WorthQueryDomainOperationDefinition::<GeometryDomain, WorkflowRead, ReadFamily>::new(
        domain::WorthQueryDomainOperationIdentity::new("workflow-read", 1),
        semantics,
    )
}

pub(super) fn lineage_stages() -> Vec<domain::WorthQueryPortableWorkflowStage> {
    valid_stages()
        .into_iter()
        .map(|stage| {
            let mut semantics = stage.semantics().clone();
            if stage.identity() == "publish" {
                semantics.effect_roles = vec![domain::WorthQueryOperationEffectFamily::Mutation];
                semantics
                    .cost_roles
                    .push(domain::WorthQueryWorkflowCostRole::Effect);
                semantics.cost_roles.sort();
                semantics.cost_roles.dedup();
            }
            domain::WorthQueryPortableWorkflowStage::new(
                stage.identity(),
                stage.predecessors().iter().map(String::as_str),
                stage.is_terminal(),
                stage.is_publishable(),
                stage.required_capabilities().iter().cloned(),
            )
            .with_semantics(semantics)
        })
        .collect()
}
