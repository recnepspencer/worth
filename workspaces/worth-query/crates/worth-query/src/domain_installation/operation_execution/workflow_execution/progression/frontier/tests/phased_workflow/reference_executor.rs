//! Member-by-member reference executor: owner reads, computation and effects
//! finish together in apply. Its orchestration reference uses singleton batches.
use worth_query::facade::{domain, read, runtime};

use super::{world, PhaseDomain, PhaseFamily, PhaseOperation};

pub(crate) struct ReferenceExecutor;

impl domain::WorthQueryDomainWorkflowStageExecutor<PhaseDomain, PhaseOperation, PhaseFamily>
    for ReferenceExecutor
{
    const LOWERING_FAMILY: &'static str = "workflow-fold-v1";
    const DETERMINISTIC: bool = true;
    const IDEMPOTENT_STAGE_RETRY: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        world::execution_resource_support()
    }

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        Some(world::installed_read_declaration())
    }

    fn apply(
        &self,
        application: domain::WorthQueryWorkflowStageApplication<'_, '_, '_>,
    ) -> Result<
        domain::WorthQueryWorkflowStageMaterial,
        domain::WorthQueryWorkflowStageExecutorFailure,
    > {
        let (input, _computed, context, workspace) = application.into_parts();
        reference_step(input, context, workspace)
    }
}

pub(crate) fn reference_step(
    input: domain::WorthQueryWorkflowValue,
    context: &domain::WorthQueryWorkflowStageExecutionContext<'_>,
    workspace: &mut domain::WorthQueryWorkflowStageWorkspace<'_>,
) -> Result<domain::WorthQueryWorkflowStageMaterial, domain::WorthQueryWorkflowStageExecutorFailure>
{
    let stage = context.stage().identity();
    if stage == "start" {
        return Ok(domain::WorthQueryWorkflowStageMaterial::new(
            domain::WorthQueryWorkflowValue::Text("start".into()),
        ));
    }
    if stage == "publish" {
        return Ok(domain::WorthQueryWorkflowStageMaterial::projection(
            "model",
            context.execute_installed_read("model", workspace)?,
        )
        .with_result_state(domain::WorthQueryOperationResultState::Ready));
    }
    let domain::WorthQueryWorkflowValue::Text(input) = input else {
        panic!("fixture member input must be text")
    };
    let (seed, mode) = parse(&input);
    if mode == 1 {
        return Err(failure("prepare failure"));
    }
    // Declared algorithm: positional weighted sum of 17 consecutive seed values.
    let mut folded = 0_u64;
    for position in 0..17_u64 {
        folded = folded.wrapping_add(seed.wrapping_add(position).wrapping_mul(position + 1));
    }
    if mode == 2 {
        return Err(failure("compute failure"));
    }
    if mode == 3 {
        return Err(failure("apply failure"));
    }
    let command = runtime::WorthQueryAspectMutationBuilder::new()
        .aspect("identity.id", format!("{stage}:{folded}"))
        .build_insert("Vertex")
        .unwrap();
    context
        .execute_mutation(command, workspace)
        .map_err(|denial| failure(format!("{denial:?}")))?;
    if mode == 4 {
        return Err(failure("failure after effect"));
    }
    let read = context.execute_installed_read("model", workspace)?;
    let output = if mode == 5 {
        domain::WorthQueryWorkflowValue::Bool(false)
    } else {
        domain::WorthQueryWorkflowValue::Text(format!(
            "{stage}:{folded}:{}",
            read.result().rows().len()
        ))
    };
    Ok(
        domain::WorthQueryWorkflowStageMaterial::new(output)
            .with_primary_graph_read("model", &read),
    )
}

pub(crate) fn parse(input: &str) -> (u64, u64) {
    let mut parts = input.split(':');
    let (seed, mode) = (parts.next().unwrap(), parts.next().unwrap());
    (seed.parse().unwrap(), mode.parse().unwrap())
}

pub(crate) fn failure(detail: impl Into<String>) -> domain::WorthQueryWorkflowStageExecutorFailure {
    domain::WorthQueryWorkflowStageExecutorFailure::new(
        domain::WorthQueryOperationFailureClass::Dependency,
        detail,
    )
}
