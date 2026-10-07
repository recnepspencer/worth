use std::sync::OnceLock;
use worth_query::facade::{domain, read, runtime};

use super::{
    CountVertices, CountVerticesInput, FederatedRead, GeometryDomain, ReadExecutionInput,
    ReadFamily, ReadVertex,
};

mod workflow;

pub(super) use workflow::{
    materialize_owned_workflow_stage, MismatchedWorkflowDeterminismExecutor,
    MismatchedWorkflowStageExecutor, WorkflowStageExecutor,
};

#[derive(Clone, Copy)]
pub(super) struct ReadVertexExecutor;

impl domain::WorthQueryDomainOperationExecutor<GeometryDomain, ReadVertex, ReadFamily>
    for ReadVertexExecutor
{
    const LOWERING_FAMILY: &'static str = "read-vertex-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::execution_resource_support()
    }

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        Some(installed_read_declaration())
    }

    fn execute(
        &self,
        input: ReadExecutionInput,
        context: &domain::WorthQueryOperationExecutionContext<'_>,
        workspace: &mut domain::WorthQueryOperationWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryOperationExecutionMaterial<read::WorthQueryReadCompletion>,
        domain::WorthQueryOperationExecutorFailure,
    > {
        if let Some(class) = input.failure {
            return Err(domain::WorthQueryOperationExecutorFailure::new(
                class,
                "deliberate executor failure",
            ));
        }
        let completion = context.execute_installed_read(workspace)?;
        let material = domain::WorthQueryOperationExecutionMaterial::new(completion, input.state);
        Ok(match input.warning {
            Some(warning) => material.with_warning(warning),
            None => material,
        })
    }
}

pub(super) fn execute_reconstructed_read_for_sabotage(
    workspace: &mut runtime::WorthQueryWorkspace,
) -> Result<read::WorthQueryReadCompletion, domain::WorthQueryOperationExecutorFailure> {
    let declaration = read::declare(|builder| {
        builder.local_detail(
            "Vertex",
            schema(),
            |query| query.project(read::AspectFieldSelector::new("identity", "id").unwrap()),
            |shape| {
                shape.field(read::AuthoredResultShapeField::new("identity", "id", "id").unwrap())
            },
        )
    })
    .map_err(|stop| {
        domain::WorthQueryOperationExecutorFailure::new(
            domain::WorthQueryOperationFailureClass::Dependency,
            format!("{stop:?}"),
        )
    })?;
    declaration
        .using(read::current())
        .run(workspace)
        .into_result()
        .map_err(|stop| {
            domain::WorthQueryOperationExecutorFailure::new(
                domain::WorthQueryOperationFailureClass::Dependency,
                format!("{stop:?}"),
            )
        })
}

#[derive(Clone, Copy)]
pub(super) struct CountVerticesExecutor;

impl domain::WorthQueryDomainOperationExecutor<GeometryDomain, CountVertices, ReadFamily>
    for CountVerticesExecutor
{
    const LOWERING_FAMILY: &'static str = "read-vertex-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::execution_resource_support()
    }

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        Some(installed_read_declaration())
    }

    fn execute(
        &self,
        input: CountVerticesInput,
        context: &domain::WorthQueryOperationExecutionContext<'_>,
        workspace: &mut domain::WorthQueryOperationWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryOperationExecutionMaterial<u64>,
        domain::WorthQueryOperationExecutorFailure,
    > {
        let completion = context.execute_installed_read(workspace)?;
        Ok(domain::WorthQueryOperationExecutionMaterial::new(
            (completion.result().rows().len() as u64).max(input.minimum.unwrap_or_default()),
            domain::WorthQueryOperationResultState::Ready,
        ))
    }
}

#[derive(Clone, Copy)]
pub(super) struct FederatedReadExecutor;

impl domain::WorthQueryDomainOperationExecutor<GeometryDomain, FederatedRead, ReadFamily>
    for FederatedReadExecutor
{
    const LOWERING_FAMILY: &'static str = "read-vertex-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::ExternalBoundary;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::execution_resource_support()
    }

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        Some(installed_read_declaration())
    }

    fn execute(
        &self,
        _: (),
        context: &domain::WorthQueryOperationExecutionContext<'_>,
        workspace: &mut domain::WorthQueryOperationWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryOperationExecutionMaterial<read::WorthQueryReadCompletion>,
        domain::WorthQueryOperationExecutorFailure,
    > {
        execute_federated_read(context, workspace)
    }
}

#[derive(Clone, Copy)]
pub(super) struct PartialEffectFederatedReadExecutor;

impl domain::WorthQueryDomainOperationExecutor<GeometryDomain, FederatedRead, ReadFamily>
    for PartialEffectFederatedReadExecutor
{
    const LOWERING_FAMILY: &'static str = "read-vertex-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::ExternalBoundary;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::partial_effect_execution_resource_support()
    }

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        Some(installed_read_declaration())
    }

    fn execute(
        &self,
        _: (),
        context: &domain::WorthQueryOperationExecutionContext<'_>,
        workspace: &mut domain::WorthQueryOperationWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryOperationExecutionMaterial<read::WorthQueryReadCompletion>,
        domain::WorthQueryOperationExecutorFailure,
    > {
        execute_federated_read(context, workspace)
    }
}

fn execute_federated_read(
    context: &domain::WorthQueryOperationExecutionContext<'_>,
    workspace: &mut domain::WorthQueryOperationWorkspace<'_>,
) -> Result<
    domain::WorthQueryOperationExecutionMaterial<read::WorthQueryReadCompletion>,
    domain::WorthQueryOperationExecutorFailure,
> {
    let projected = context.graph_projection("remote-a").ok_or_else(|| {
        domain::WorthQueryOperationExecutorFailure::new(
            domain::WorthQueryOperationFailureClass::Dependency,
            "the installed remote-a adapter supplied no projection",
        )
    })?;
    Ok(domain::WorthQueryOperationExecutionMaterial::new(
        context.execute_installed_read(workspace)?,
        domain::WorthQueryOperationResultState::Ready,
    )
    .with_warning(domain::WorthQueryOperationExecutionWarning::Advisory(
        format!("remote-a-projected-rows={}", projected.rows().len()),
    )))
}

#[derive(Clone, Copy)]
pub(super) struct UnderstatedFederatedReadExecutor;

impl domain::WorthQueryDomainOperationExecutor<GeometryDomain, FederatedRead, ReadFamily>
    for UnderstatedFederatedReadExecutor
{
    const LOWERING_FAMILY: &'static str = "read-vertex-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::execution_resource_support()
    }

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        Some(installed_read_declaration())
    }

    fn execute(
        &self,
        input: (),
        context: &domain::WorthQueryOperationExecutionContext<'_>,
        workspace: &mut domain::WorthQueryOperationWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryOperationExecutionMaterial<read::WorthQueryReadCompletion>,
        domain::WorthQueryOperationExecutorFailure,
    > {
        domain::WorthQueryDomainOperationExecutor::execute(
            &FederatedReadExecutor,
            input,
            context,
            workspace,
        )
    }
}

fn schema() -> read::QuerySchemaView {
    read::QuerySchemaView::new(
        "installed-operation",
        [read::SchemaFieldView::new(
            read::AspectName::new("identity").unwrap(),
            read::FieldName::new("id").unwrap(),
            read::ScalarAspectType::String,
        )],
        [],
    )
}

pub(super) fn installed_read_declaration() -> &'static read::WorthQueryReadDeclaration {
    static DECLARATION: OnceLock<read::WorthQueryReadDeclaration> = OnceLock::new();
    DECLARATION.get_or_init(|| {
        read::declare(|builder| {
            builder.local_detail(
                "Vertex",
                schema(),
                |query| query.project(read::AspectFieldSelector::new("identity", "id").unwrap()),
                |shape| {
                    shape
                        .field(read::AuthoredResultShapeField::new("identity", "id", "id").unwrap())
                },
            )
        })
        .expect("installed read declaration fixture is canonical")
    })
}

pub(super) fn mismatched_read_declaration() -> &'static read::WorthQueryReadDeclaration {
    static DECLARATION: OnceLock<read::WorthQueryReadDeclaration> = OnceLock::new();
    DECLARATION.get_or_init(|| {
        read::declare(|builder| {
            builder.local_detail(
                "Vertex",
                schema(),
                |query| query.project(read::AspectFieldSelector::new("identity", "id").unwrap()),
                |shape| {
                    shape.field(
                        read::AuthoredResultShapeField::new("identity", "id", "vertex_id").unwrap(),
                    )
                },
            )
        })
        .expect("mismatched read declaration fixture is independently canonical")
    })
}
