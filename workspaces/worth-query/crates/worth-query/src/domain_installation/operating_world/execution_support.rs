use std::collections::BTreeSet;
use std::sync::Arc;

use worth_query_admission::facade::resource_admission::{
    WorthQueryExecutionResourceSupport, WorthQueryExecutionResourceSupportSnapshot,
};
use worth_query_execution::facade::domain_computation::WorthQueryInstalledOperationExecutionSupport;
use worth_query_installation::facade::{
    WorthQueryConditionalNodeLocation, WorthQueryOperationTouchContract,
    WorthQueryOperationWorkflowContract, WorthQueryPortableDomainOperationDefinition,
};

use super::{WorthQueryBoundCommitPosture, WorthQueryBoundGraphParticipation};

type ConditionalNode = crate::domain_installation::WorthQueryInstalledConditionalNode;
type DirectExecutor = crate::domain_installation::WorthQueryInstalledDomainOperationExecutor;
type WorkflowExecutor = crate::domain_installation::WorthQueryInstalledWorkflowStageExecutor;
type WorkflowGraph = crate::domain_installation::WorthQueryInstalledWorkflowGraph;
type ParallelProvider =
    crate::domain_installation::WorthQueryInstalledWorkflowParallelAdmissionProvider;

#[derive(Clone)]
pub(crate) enum WorthQueryBoundWorkflowParallelPosture {
    Sequential,
    Parallel(Arc<ParallelProvider>),
}

pub(crate) enum WorthQueryBoundExecutionProviders {
    Direct {
        executor: Arc<DirectExecutor>,
    },
    Workflow {
        graph: Arc<WorkflowGraph>,
        executor: Arc<WorkflowExecutor>,
        parallel: WorthQueryBoundWorkflowParallelPosture,
    },
}

pub(super) struct WorthQueryInstalledExecutionClosure {
    pub(super) support: WorthQueryInstalledOperationExecutionSupport,
    pub(super) providers: WorthQueryBoundExecutionProviders,
}

pub(super) struct WorthQueryInstalledRuntimeProviders<'a> {
    pub(super) direct: Option<&'a Arc<DirectExecutor>>,
    pub(super) workflow_graph: Option<Arc<WorkflowGraph>>,
    pub(super) workflow: Option<&'a Arc<WorkflowExecutor>>,
    pub(super) parallel: Option<&'a Arc<ParallelProvider>>,
}

pub(super) fn lower_installed_execution_support(
    definition: &WorthQueryPortableDomainOperationDefinition,
    graphs: &[WorthQueryBoundGraphParticipation],
    commit_posture: WorthQueryBoundCommitPosture,
    conditional_nodes: &[Arc<ConditionalNode>],
    providers: WorthQueryInstalledRuntimeProviders<'_>,
) -> Result<WorthQueryInstalledExecutionClosure, &'static str> {
    match &definition.semantics().workflow {
        WorthQueryOperationWorkflowContract::NotRequired => lower_direct_execution_closure(
            definition,
            graphs,
            commit_posture,
            conditional_nodes,
            providers,
        ),
        WorthQueryOperationWorkflowContract::Declared(workflow) => {
            lower_workflow_execution_closure(
                workflow,
                graphs,
                commit_posture,
                conditional_nodes,
                providers,
            )
        }
    }
}

fn lower_direct_execution_closure(
    definition: &WorthQueryPortableDomainOperationDefinition,
    graphs: &[WorthQueryBoundGraphParticipation],
    commit_posture: WorthQueryBoundCommitPosture,
    conditional_nodes: &[Arc<ConditionalNode>],
    providers: WorthQueryInstalledRuntimeProviders<'_>,
) -> Result<WorthQueryInstalledExecutionClosure, &'static str> {
    let executor = providers
        .direct
        .ok_or("installed direct operation has no exact executor")?;
    if providers.workflow_graph.is_some()
        || providers.workflow.is_some()
        || providers.parallel.is_some()
    {
        return Err("direct operation retained workflow-only providers");
    }
    Ok(WorthQueryInstalledExecutionClosure {
        support: WorthQueryInstalledOperationExecutionSupport::direct(direct_support_snapshot(
            definition,
            graphs,
            commit_posture,
            conditional_nodes,
            &executor.resource_support,
        )),
        providers: WorthQueryBoundExecutionProviders::Direct {
            executor: Arc::clone(executor),
        },
    })
}

fn lower_workflow_execution_closure(
    workflow: &worth_query_installation::facade::WorthQueryPortableWorkflowDefinition,
    graphs: &[WorthQueryBoundGraphParticipation],
    commit_posture: WorthQueryBoundCommitPosture,
    conditional_nodes: &[Arc<ConditionalNode>],
    providers: WorthQueryInstalledRuntimeProviders<'_>,
) -> Result<WorthQueryInstalledExecutionClosure, &'static str> {
    let graph = providers
        .workflow_graph
        .ok_or("installed workflow operation has no exact workflow graph")?;
    let executor = providers
        .workflow
        .ok_or("installed workflow operation has no exact stage executor")?;
    if providers.direct.is_some()
        || workflow.has_parallel_frontier() != providers.parallel.is_some()
    {
        return Err("workflow provider closure disagrees with installed semantics");
    }
    let operation = workflow_operation_support(executor, conditional_nodes, providers.parallel);
    let stages = workflow.stages().iter().map(|stage| {
        (
            stage.identity().to_owned(),
            workflow_stage_support_snapshot(
                stage,
                graphs,
                commit_posture,
                conditional_nodes,
                &executor.resource_support,
            ),
        )
    });
    let parallel = match providers.parallel {
        Some(provider) => WorthQueryBoundWorkflowParallelPosture::Parallel(Arc::clone(provider)),
        None => WorthQueryBoundWorkflowParallelPosture::Sequential,
    };
    Ok(WorthQueryInstalledExecutionClosure {
        support: WorthQueryInstalledOperationExecutionSupport::workflow(operation, stages),
        providers: WorthQueryBoundExecutionProviders::Workflow {
            graph,
            executor: Arc::clone(executor),
            parallel,
        },
    })
}

fn workflow_operation_support(
    executor: &Arc<WorkflowExecutor>,
    conditional_nodes: &[Arc<ConditionalNode>],
    parallel: Option<&Arc<ParallelProvider>>,
) -> WorthQueryExecutionResourceSupportSnapshot {
    WorthQueryExecutionResourceSupportSnapshot::new(
        executor.resource_support.clone(),
        operation_conditional_supports(conditional_nodes),
        Vec::new(),
        Vec::new(),
        parallel.map(|provider| provider.resource_support().clone()),
    )
}

fn direct_support_snapshot(
    definition: &WorthQueryPortableDomainOperationDefinition,
    graphs: &[WorthQueryBoundGraphParticipation],
    commit_posture: WorthQueryBoundCommitPosture,
    conditional_nodes: &[Arc<ConditionalNode>],
    executor: &WorthQueryExecutionResourceSupport,
) -> WorthQueryExecutionResourceSupportSnapshot {
    let semantics = definition.semantics();
    let mut graph_roles = semantics
        .graph_reads
        .domain_roles()
        .iter()
        .map(|read| read.role.as_str())
        .collect::<BTreeSet<_>>();
    if let WorthQueryOperationTouchContract::Declared {
        graph_roles: touch_roles,
        ..
    } = &semantics.touches
    {
        graph_roles.extend(touch_roles.iter().map(String::as_str));
    }
    WorthQueryExecutionResourceSupportSnapshot::new(
        executor.clone(),
        operation_conditional_supports(conditional_nodes),
        graph_supports_for_roles(graphs, &graph_roles),
        commit_supports_for_roles(graphs, commit_posture, &graph_roles),
        None,
    )
}

fn workflow_stage_support_snapshot(
    stage: &worth_query_installation::facade::WorthQueryPortableWorkflowStage,
    graphs: &[WorthQueryBoundGraphParticipation],
    commit_posture: WorthQueryBoundCommitPosture,
    conditional_nodes: &[Arc<ConditionalNode>],
    executor: &WorthQueryExecutionResourceSupport,
) -> WorthQueryExecutionResourceSupportSnapshot {
    let roles = stage
        .semantics()
        .graph_read_roles
        .iter()
        .chain(&stage.semantics().touch_roles)
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let touch_roles = stage
        .semantics()
        .touch_roles
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    WorthQueryExecutionResourceSupportSnapshot::new(
        executor.clone(),
        stage_conditional_supports(conditional_nodes, stage.identity()),
        graph_supports_for_roles(graphs, &roles),
        commit_supports_for_roles(graphs, commit_posture, &touch_roles),
        None,
    )
}

fn operation_conditional_supports(
    nodes: &[Arc<ConditionalNode>],
) -> Vec<(String, WorthQueryExecutionResourceSupport)> {
    nodes
        .iter()
        .filter_map(|node| match &node.location {
            WorthQueryConditionalNodeLocation::Operation { node_identity } => Some((
                format!("operation:{node_identity}"),
                node.resource_support.clone(),
            )),
            WorthQueryConditionalNodeLocation::WorkflowStage { .. } => None,
        })
        .collect()
}

fn stage_conditional_supports(
    nodes: &[Arc<ConditionalNode>],
    expected_stage: &str,
) -> Vec<(String, WorthQueryExecutionResourceSupport)> {
    nodes
        .iter()
        .filter_map(|node| match &node.location {
            WorthQueryConditionalNodeLocation::WorkflowStage {
                stage_identity,
                node_identity,
            } if stage_identity == expected_stage => Some((
                format!("stage:{stage_identity}:{node_identity}"),
                node.resource_support.clone(),
            )),
            _ => None,
        })
        .collect()
}

fn graph_supports_for_roles(
    graphs: &[WorthQueryBoundGraphParticipation],
    roles: &BTreeSet<&str>,
) -> Vec<(String, WorthQueryExecutionResourceSupport)> {
    graphs
        .iter()
        .filter(|graph| roles.contains(graph.role.as_str()))
        .map(|graph| (graph.role.clone(), graph.record.resource_support.clone()))
        .collect()
}

fn commit_supports_for_roles(
    graphs: &[WorthQueryBoundGraphParticipation],
    commit_posture: WorthQueryBoundCommitPosture,
    roles: &BTreeSet<&str>,
) -> Vec<(String, WorthQueryExecutionResourceSupport)> {
    if commit_posture != WorthQueryBoundCommitPosture::Atomic {
        return Vec::new();
    }
    let mut groups = Vec::<(
        Arc<crate::domain_installation::graph_participation::WorthQueryInstalledGraphCommitAuthority>,
        Vec<String>,
    )>::new();
    for graph in graphs
        .iter()
        .filter(|graph| roles.contains(graph.role.as_str()))
    {
        let Some(authority) = &graph.record.commit_authority else {
            continue;
        };
        match groups
            .iter_mut()
            .find(|(candidate, _)| Arc::ptr_eq(candidate, authority))
        {
            Some((_, group_roles)) => group_roles.push(graph.role.clone()),
            None => groups.push((Arc::clone(authority), vec![graph.role.clone()])),
        }
    }
    groups
        .into_iter()
        .map(|(authority, mut roles)| {
            roles.sort();
            (roles.join(","), authority.resource_support.clone())
        })
        .collect()
}
