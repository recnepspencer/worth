//! One native driver for admitted Boolean requirements and typed body values.
use crate::domain_computation::primary_graph::{
    application_contribution::{advancement_denial, QueryRequestExecution},
    operation_body::PreparedRequirementOperand,
    HandlerExecutionDenial, WorthQueryAdvancementDenial,
    WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryInvariantEntityIdentity,
    WorthQueryManagedComputationResourceDenial,
};
use worth_foundational::expression_api::{
    ExpressionDenialDetail, ExpressionInputs, ExpressionResource, ExpressionStep, MAX_SLICE_QUANTUM,
};
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyExpression as Expression;
#[derive(Clone, Copy)]
pub(super) enum Purpose<'a> {
    Requirement,
    Value(&'a str),
}
impl Purpose<'_> {
    fn denial(
        self,
        cause: worth_foundational::expression_api::ExpressionDenial,
    ) -> HandlerExecutionDenial {
        use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial as Denial;
        HandlerExecutionDenial::new(match self {
            Self::Requirement => {
                Denial::body_requirement_expression("operation body requirement", cause)
            }
            Self::Value(name) => Denial::body_value_expression(name, cause),
        })
    }
}
pub(super) fn evaluate<
    Schema: worth_query_installation::facade::ApplicationSchema,
    Operation,
    Entity,
>(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>,
    identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    operands: &[worth_query_declaration::facade::application_schema::ApplicationOperationBodyRequirementOperand],
    expression: &Expression,
    execution: &QueryRequestExecution<'_>,
    purpose: Purpose<'_>,
) -> Result<worth_foundational::expression_api::ExpressionValue, HandlerExecutionDenial> {
    let fixed_memory = [
        ExpressionResource::ScratchBytes,
        ExpressionResource::RetainedConsumptionBytes,
        ExpressionResource::OutputBytes,
    ]
    .into_iter()
    .try_fold(0_u64, |total, resource| {
        total.checked_add(Expression::PROFILE.limit(resource))
    })
    .ok_or_else(overflow)?;
    let slots = expression.compiled().slots().len();
    let slot_memory = slots
        .checked_mul(
            size_of::<PreparedRequirementOperand>()
                + size_of::<worth_foundational::expression_api::ExpressionValue>() * 3,
        )
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(overflow)?;
    let _memory = execution
        .reserve(fixed_memory.checked_add(slot_memory).ok_or_else(overflow)?)
        .map_err(|cause| {
            HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(cause))
        })?;
    execution
        .decision_stage(|context| {
            context.checkpoint(0).map_err(stopped)?;
            let mut held = Vec::with_capacity(slots);
            let mut inputs = ExpressionInputs::builder(expression.schema());
            for (name, _) in expression.compiled().slots() {
                let source = operands
                    .iter()
                    .find(|operand| operand.name() == name)
                    .expect("admitted operand slot")
                    .source();
                let observed =
                    reader.body_requirement_operand(identity, source, execution, context)?;
                inputs = inputs
                    .bind(name, observed.value.clone())
                    .map_err(|cause| purpose.denial(cause))?;
                held.push(observed);
            }
            let remaining = context.remaining_work();
            let semantic_work = Expression::PROFILE.limit(ExpressionResource::SemanticWork);
            let profile = match remaining {
                Some(0) => return Err(exhausted()),
                Some(remaining) => Expression::PROFILE
                    .narrowed(
                        ExpressionResource::SemanticWork,
                        remaining.min(semantic_work),
                    )
                    .map_err(|cause| purpose.denial(cause))?,
                None => Expression::PROFILE,
            };
            let mut continuation = expression
                .compiled()
                .start(&inputs.build(), &profile)
                .map_err(|cause| purpose.denial(cause))?;
            let mut charged = 0;
            loop {
                context.checkpoint(0).map_err(stopped)?;
                let step = continuation.step(MAX_SLICE_QUANTUM);
                let used = match &step {
                    ExpressionStep::Complete(result) => {
                        result.cost().used(ExpressionResource::SemanticWork)
                    }
                    ExpressionStep::Suspended(next) => {
                        next.cost().used(ExpressionResource::SemanticWork)
                    }
                };
                context
                    .account_completed_work(used.checked_sub(charged).ok_or_else(overflow)?)
                    .map_err(stopped)?;
                charged = used;
                context.checkpoint(0).map_err(stopped)?;
                match step {
                    ExpressionStep::Suspended(next) => continuation = next,
                    ExpressionStep::Complete(result) => {
                        let result = result.into_result().map_err(|cause| {
                            if remaining.is_some_and(|remaining| remaining < semantic_work)
                                && matches!(
                                    cause.detail(),
                                    ExpressionDenialDetail::ResourceExceeded {
                                        resource: ExpressionResource::SemanticWork,
                                        ..
                                    }
                                )
                            {
                                exhausted()
                            } else {
                                purpose.denial(cause)
                            }
                        })?;
                        return Ok(result);
                    }
                }
            }
        })
        .map_err(HandlerExecutionDenial::new)?
}

fn overflow() -> HandlerExecutionDenial {
    HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(
        WorthQueryManagedComputationResourceDenial::CapacityOverflow,
    ))
}
fn exhausted() -> HandlerExecutionDenial {
    HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(
        WorthQueryManagedComputationResourceDenial::WorkExhausted,
    ))
}
fn stopped(cause: worth_execution::MapKernelStop) -> HandlerExecutionDenial {
    HandlerExecutionDenial::new(advancement_denial(
        worth_execution::WorkCeilingDenial::Stopped(cause),
    ))
}
