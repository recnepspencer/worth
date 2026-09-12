use worth_query::facade::{domain, read, runtime};
use worth_relational::facade::identity::KindId;

use super::{
    canonical_bundle, configured_runtime_without_executors, semantic_closure, GeometryDomain,
};

#[derive(Clone, Copy, Debug)]
pub struct WorkflowMutation;

#[derive(Clone, Copy, Debug)]
pub struct MutationFamily;

impl domain::WorthQueryExecutableDomainOperation<GeometryDomain, MutationFamily>
    for WorkflowMutation
{
    type Input = ();
    type Output = ();
    type Publication = domain::WorthQueryTerminalOperation;
    type Execution = domain::WorthQueryWorkflowOperation;
}

#[derive(Clone, Copy)]
struct MutationWorkflowExecutor;

#[derive(Clone, Copy)]
struct MixedMutationWorkflowExecutor;

impl domain::WorthQueryDomainWorkflowStageExecutor<GeometryDomain, WorkflowMutation, MutationFamily>
    for MixedMutationWorkflowExecutor
{
    const LOWERING_FAMILY: &'static str = "workflow-mutation-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::ExternalBoundary;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::execution_resource_support()
    }

    fn execute_stage(
        &self,
        input: domain::WorthQueryWorkflowValue,
        context: &domain::WorthQueryWorkflowStageExecutionContext<'_>,
        workspace: &mut domain::WorthQueryWorkflowStageWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryWorkflowStageMaterial,
        domain::WorthQueryWorkflowStageExecutorFailure,
    > {
        execute_mutation_stage(input, context, workspace, false)
    }
}

impl domain::WorthQueryDomainWorkflowStageExecutor<GeometryDomain, WorkflowMutation, MutationFamily>
    for MutationWorkflowExecutor
{
    const LOWERING_FAMILY: &'static str = "workflow-mutation-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::execution_resource_support()
    }

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        Some(super::executors::installed_read_declaration())
    }

    fn execute_stage(
        &self,
        input: domain::WorthQueryWorkflowValue,
        context: &domain::WorthQueryWorkflowStageExecutionContext<'_>,
        workspace: &mut domain::WorthQueryWorkflowStageWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryWorkflowStageMaterial,
        domain::WorthQueryWorkflowStageExecutorFailure,
    > {
        execute_mutation_stage(input, context, workspace, true)
    }
}

fn execute_mutation_stage(
    input: domain::WorthQueryWorkflowValue,
    context: &domain::WorthQueryWorkflowStageExecutionContext<'_>,
    workspace: &mut domain::WorthQueryWorkflowStageWorkspace<'_>,
    reads_primary_model: bool,
) -> Result<domain::WorthQueryWorkflowStageMaterial, domain::WorthQueryWorkflowStageExecutorFailure>
{
    let command = runtime::WorthQueryAspectMutationBuilder::new()
        .aspect("identity.id", "workflow-mutation-entity")
        .build_insert("Vertex")
        .map_err(|detail| {
            domain::WorthQueryWorkflowStageExecutorFailure::new(
                domain::WorthQueryOperationFailureClass::InvalidInput,
                format!("{detail:?}"),
            )
        })?;
    context
        .execute_mutation(command, workspace)
        .map_err(|denial| {
            domain::WorthQueryWorkflowStageExecutorFailure::new(
                domain::WorthQueryOperationFailureClass::Dependency,
                format!("{denial:?}"),
            )
        })?;
    if matches!(input, domain::WorthQueryWorkflowValue::Text(value) if value == "fail-after-effect")
    {
        return Err(domain::WorthQueryWorkflowStageExecutorFailure::new(
            domain::WorthQueryOperationFailureClass::Dependency,
            "declared failure after mutation",
        ));
    }
    let primary_read = reads_primary_model
        .then(|| context.execute_installed_read("model", workspace))
        .transpose()?;
    let material = domain::WorthQueryWorkflowStageMaterial::new(
        domain::WorthQueryWorkflowValue::Text("mutation-committed".into()),
    )
    .with_result_state(domain::WorthQueryOperationResultState::Ready);
    Ok(match primary_read {
        Some(read) => material.with_primary_graph_read("model", &read),
        None => material,
    })
}

pub fn mutation_workflow_workspace(
    name: &str,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    configured_runtime_without_executors(mutation_workflow_package())
        .workflow_stage_executor(
            GeometryDomain,
            WorkflowMutation,
            MutationFamily,
            MutationWorkflowExecutor,
        )
        .workspace(name)
}

fn mutation_workflow_package() -> domain::WorthQueryDomainPackage<GeometryDomain> {
    let operation = domain::WorthQueryDomainOperationDefinition::<
        GeometryDomain,
        WorkflowMutation,
        MutationFamily,
    >::new(
        domain::WorthQueryDomainOperationIdentity::new("workflow-mutation", 1),
        mutation_workflow_semantics(),
    );
    domain::WorthQueryDomainPackage::declare(GeometryDomain, geometry_identity())
        .invariant(mutation_invariant())
        .operation(operation)
}

fn mutation_workflow_semantics() -> domain::WorthQueryDomainOperationSemanticClosure {
    let mut semantics = semantic_closure(
        canonical_bundle("Vertex"),
        domain::WorthQuerySupportRequirement::NotRequired,
        false,
    );
    semantics.graph_reads = domain::WorthQueryOperationGraphReadContract::DeclaredDomain {
        roles: vec![domain::WorthQueryDomainOperationGraphReadRole {
            role: "model".into(),
            participation: domain::WorthQueryOperationGraphParticipation::PrimaryLogicalGraph,
            access: domain::WorthQueryOperationGraphAccess::Observe,
            semantic_reads: Vec::new(),
        }],
    };
    semantics.touches = domain::WorthQueryOperationTouchContract::Declared {
        graph_roles: vec!["model".into()],
        scopes: vec![domain::WorthQueryOperationTouchScope::DeclaredDomain(
            domain::WorthQueryDeclaredDomainTouchScopeIdentity::new("vertex").unwrap(),
        )],
    };
    semantics.effects = domain::WorthQueryOperationEffectContract::Declared {
        effect_families: vec![domain::WorthQueryOperationEffectFamily::Mutation],
    };
    semantics.aftermath = None;
    semantics.invariants = domain::WorthQueryOperationInvariantContract::Declared {
        invariant_slots: vec!["workflow-invariant:1".into()],
    };
    semantics.invariant_execution = mutation_invariant_execution();
    semantics.lowering.family = "workflow-mutation-v1".into();
    semantics.terminal.failure_classes = mutation_failure_classes();
    semantics.workflow =
        domain::WorthQueryOperationWorkflowContract::Declared(mutation_workflow_definition());
    semantics
}

fn mutation_invariant_execution() -> domain::WorthQueryInvariantExecutionContract {
    domain::WorthQueryInvariantExecutionContract::declared([
        domain::WorthQueryInstalledInvariantExecutionRequirement::new(
            "workflow-invariant:1",
            "workflow-invariant",
            std::num::NonZeroU32::new(1).unwrap(),
            domain::WorthQueryInvariantEnforcement::Blocking,
            "model",
            ["vertex"],
            4,
            8,
        )
        .unwrap(),
    ])
    .unwrap()
}

fn mutation_workflow_definition() -> domain::WorthQueryPortableWorkflowDefinition {
    let semantics = domain::WorthQueryWorkflowStageSemantics {
        input: domain::WorthQueryWorkflowValueContract::Text,
        output: domain::WorthQueryWorkflowValueContract::Text,
        graph_read_roles: vec!["model".into()],
        touch_roles: vec!["model".into()],
        effect_roles: vec![domain::WorthQueryOperationEffectFamily::Mutation],
        invariant_roles: vec!["workflow-invariant:1".into()],
        cost_roles: vec![
            domain::WorthQueryWorkflowCostRole::Admission,
            domain::WorthQueryWorkflowCostRole::GraphRead,
            domain::WorthQueryWorkflowCostRole::Effect,
            domain::WorthQueryWorkflowCostRole::Invariant,
            domain::WorthQueryWorkflowCostRole::Execution,
            domain::WorthQueryWorkflowCostRole::ResultValidation,
        ],
        resources: super::execution_resource_contract(),
        terminal_result_states: vec![domain::WorthQueryOperationResultState::Ready],
        failure_classes: mutation_failure_classes(),
        ..Default::default()
    };
    domain::WorthQueryPortableWorkflowDefinition::new(
        "mutate",
        [domain::WorthQueryPortableWorkflowStage::new(
            "mutate",
            std::iter::empty::<&str>(),
            true,
            false,
            std::iter::empty::<domain::WorthQueryOperationCapabilityRequirement>(),
        )
        .with_semantics(semantics)],
    )
}

fn mutation_invariant() -> domain::WorthQueryDomainInvariantDefinition {
    domain::WorthQueryDomainInvariantDefinition::new(
        domain::WorthQueryDomainIdentityName::new("workflow-invariant").unwrap(),
        domain::WorthQueryDomainSemanticVersion::new(1, 0),
        domain::WorthQueryDomainInvariantPredicate::requires_outgoing_relations(
            vec![KindId::new(0xff00_0001)],
            vec![KindId::new(0xff00_0002)],
            1,
        ),
        std::num::NonZeroU64::new(4096).unwrap(),
    )
}

fn mutation_failure_classes() -> Vec<domain::WorthQueryOperationFailureClass> {
    vec![
        domain::WorthQueryOperationFailureClass::InvalidInput,
        domain::WorthQueryOperationFailureClass::Dependency,
    ]
}

fn geometry_identity() -> domain::WorthQueryDomainIdentityDeclaration<GeometryDomain> {
    domain::WorthQueryDomainIdentityDeclaration::new(
        domain::WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
        domain::WorthQueryDomainIdentityName::new("geometry").unwrap(),
        domain::WorthQueryDomainSemanticVersion::new(1, 0),
    )
}

pub fn mixed_mutation_workflow_runtime<G: 'static>(
) -> worth_query::facade::consumer_kit::WorthQueryInMemoryTestRuntimeBuilder {
    let operation = domain::WorthQueryDomainOperationDefinition::<
        GeometryDomain,
        WorkflowMutation,
        MutationFamily,
    >::new(
        domain::WorthQueryDomainOperationIdentity::new("mixed-workflow-mutation", 1),
        mixed_mutation_semantics(),
    );
    let package = domain::WorthQueryDomainPackage::declare(GeometryDomain, geometry_identity())
        .operation(operation)
        .operation_graph_participation::<WorkflowMutation, MutationFamily, G>("remote-a");
    configured_runtime_without_executors(package).workflow_stage_executor(
        GeometryDomain,
        WorkflowMutation,
        MutationFamily,
        MixedMutationWorkflowExecutor,
    )
}

fn mixed_mutation_semantics() -> domain::WorthQueryDomainOperationSemanticClosure {
    let mut semantics = semantic_closure(
        canonical_bundle("Vertex"),
        domain::WorthQuerySupportRequirement::NotRequired,
        false,
    );
    semantics.graph_reads = domain::WorthQueryOperationGraphReadContract::DeclaredDomain {
        roles: vec![domain::WorthQueryDomainOperationGraphReadRole {
            role: "remote-a".into(),
            participation: domain::WorthQueryOperationGraphParticipation::SeparateAuthority {
                role: "remote-a".into(),
            },
            access: domain::WorthQueryOperationGraphAccess::Observe,
            semantic_reads: Vec::new(),
        }],
    };
    semantics.touches = domain::WorthQueryOperationTouchContract::Declared {
        graph_roles: vec!["remote-a".into()],
        scopes: vec![domain::WorthQueryOperationTouchScope::DeclaredDomain(
            domain::WorthQueryDeclaredDomainTouchScopeIdentity::new("vertex").unwrap(),
        )],
    };
    semantics.effects = domain::WorthQueryOperationEffectContract::Declared {
        effect_families: vec![domain::WorthQueryOperationEffectFamily::Mutation],
    };
    semantics.invariants = domain::WorthQueryOperationInvariantContract::NotRequired;
    semantics.aftermath = None;
    semantics.lowering.family = "workflow-mutation-v1".into();
    semantics.cost.execution = domain::WorthQueryOperationCostClass::ExternalBoundary;
    semantics.workflow =
        domain::WorthQueryOperationWorkflowContract::Declared(mixed_mutation_workflow_definition());
    semantics
}

fn mixed_mutation_workflow_definition() -> domain::WorthQueryPortableWorkflowDefinition {
    let semantics = domain::WorthQueryWorkflowStageSemantics {
        input: domain::WorthQueryWorkflowValueContract::Text,
        output: domain::WorthQueryWorkflowValueContract::Text,
        graph_read_roles: vec!["remote-a".into()],
        touch_roles: vec!["remote-a".into()],
        effect_roles: vec![domain::WorthQueryOperationEffectFamily::Mutation],
        cost_roles: vec![
            domain::WorthQueryWorkflowCostRole::Admission,
            domain::WorthQueryWorkflowCostRole::GraphRead,
            domain::WorthQueryWorkflowCostRole::TouchEffect,
            domain::WorthQueryWorkflowCostRole::Effect,
            domain::WorthQueryWorkflowCostRole::Execution,
            domain::WorthQueryWorkflowCostRole::ResultValidation,
        ],
        resources: super::execution_resource_contract(),
        terminal_result_states: vec![domain::WorthQueryOperationResultState::Ready],
        failure_classes: vec![domain::WorthQueryOperationFailureClass::Dependency],
        ..Default::default()
    };
    domain::WorthQueryPortableWorkflowDefinition::new(
        "mutate",
        [domain::WorthQueryPortableWorkflowStage::new(
            "mutate",
            std::iter::empty::<&str>(),
            true,
            false,
            std::iter::empty::<domain::WorthQueryOperationCapabilityRequirement>(),
        )
        .with_semantics(semantics)],
    )
}
