use worth_foundational::facade::{AspectValue, CanonicalF32};
use worth_query::facade::{domain, runtime};

use crate::{installed_domain::snapshot_measurement::MEASUREMENT_ROOT, WorthUiDomainEntry};

use super::{
    WorthUiMeasurementRecording, WorthUiMeasurementRecordingFamily, IDENTIFY_STAGE,
    LOWERING_FAMILY, RECORD_STAGE,
};

#[derive(Clone, Copy)]
pub(crate) struct WorthUiMeasurementRecordingExecutor;

impl
    domain::WorthQueryDomainWorkflowStageExecutor<
        WorthUiDomainEntry,
        WorthUiMeasurementRecording,
        WorthUiMeasurementRecordingFamily,
    > for WorthUiMeasurementRecordingExecutor
{
    const LOWERING_FAMILY: &'static str = LOWERING_FAMILY;
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        crate::installed_domain::execution_resources::operation_execution_resource_support()
    }

    fn apply(
        &self,
        application: domain::WorthQueryWorkflowStageApplication<'_, '_, '_>,
    ) -> Result<
        domain::WorthQueryWorkflowStageMaterial,
        domain::WorthQueryWorkflowStageExecutorFailure,
    > {
        let (input, _computed, context, workspace) = application.into_parts();
        match context.stage().identity() {
            IDENTIFY_STAGE => identify(input),
            RECORD_STAGE => record(input, context, workspace),
            _ => Err(invalid_input(
                "undeclared Worth UI measurement workflow stage",
            )),
        }
    }
}

fn identify(
    input: domain::WorthQueryWorkflowValue,
) -> Result<domain::WorthQueryWorkflowStageMaterial, domain::WorthQueryWorkflowStageExecutorFailure>
{
    let domain::WorthQueryWorkflowValue::Text(identity) = input else {
        return Err(invalid_input("measurement identity must be text"));
    };
    if identity.trim().is_empty() {
        return Err(invalid_input("measurement identity must not be empty"));
    }
    Ok(domain::WorthQueryWorkflowStageMaterial::new(
        domain::WorthQueryWorkflowValue::Text(identity),
    ))
}

fn record(
    input: domain::WorthQueryWorkflowValue,
    context: &domain::WorthQueryWorkflowStageExecutionContext<'_>,
    workspace: &mut domain::WorthQueryWorkflowStageWorkspace<'_>,
) -> Result<domain::WorthQueryWorkflowStageMaterial, domain::WorthQueryWorkflowStageExecutorFailure>
{
    let measurement_value = canonical_measurement_value(input)?;
    let measurement_identity = identified_measurement(context)?;
    commit_measurement(&measurement_identity, measurement_value, context, workspace)?;
    Ok(
        domain::WorthQueryWorkflowStageMaterial::new(domain::WorthQueryWorkflowValue::Text(
            "measurement-recorded".into(),
        ))
        .with_result_state(domain::WorthQueryOperationResultState::Ready),
    )
}

fn canonical_measurement_value(
    input: domain::WorthQueryWorkflowValue,
) -> Result<CanonicalF32, domain::WorthQueryWorkflowStageExecutorFailure> {
    let domain::WorthQueryWorkflowValue::U64(bits) = input else {
        return Err(invalid_input(
            "measurement value must carry canonical f32 bits",
        ));
    };
    let bits =
        u32::try_from(bits).map_err(|_| invalid_input("measurement value exceeds f32 width"))?;
    Ok(CanonicalF32::from_bits(bits))
}

fn identified_measurement(
    context: &domain::WorthQueryWorkflowStageExecutionContext<'_>,
) -> Result<String, domain::WorthQueryWorkflowStageExecutorFailure> {
    context
        .predecessor_receipts()
        .iter()
        .find(|receipt| receipt.stage_identity() == IDENTIFY_STAGE)
        .and_then(|receipt| match receipt.output() {
            domain::WorthQueryWorkflowValue::Text(identity) => Some(identity.clone()),
            _ => None,
        })
        .ok_or_else(|| dependency("measurement identity predecessor is unavailable"))
}

fn commit_measurement(
    identity: &str,
    value: CanonicalF32,
    context: &domain::WorthQueryWorkflowStageExecutionContext<'_>,
    workspace: &mut domain::WorthQueryWorkflowStageWorkspace<'_>,
) -> Result<(), domain::WorthQueryWorkflowStageExecutorFailure> {
    let command = runtime::WorthQueryAspectMutationBuilder::new()
        .aspect("identity.id", identity)
        .aspect(
            "measurement.value",
            runtime::WorthQueryAuthoredAspectValue::native(AspectValue::Float32(value)),
        )
        .build_insert(MEASUREMENT_ROOT)
        .map_err(|error| invalid_input(format!("measurement mutation is invalid: {error:?}")))?;
    context
        .execute_mutation(command, workspace)
        .map_err(|error| dependency(format!("measurement mutation was denied: {error:?}")))?;
    Ok(())
}

fn invalid_input(detail: impl Into<String>) -> domain::WorthQueryWorkflowStageExecutorFailure {
    domain::WorthQueryWorkflowStageExecutorFailure::new(
        domain::WorthQueryOperationFailureClass::InvalidInput,
        detail,
    )
}

fn dependency(detail: impl Into<String>) -> domain::WorthQueryWorkflowStageExecutorFailure {
    domain::WorthQueryWorkflowStageExecutorFailure::new(
        domain::WorthQueryOperationFailureClass::Dependency,
        detail,
    )
}
