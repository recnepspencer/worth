//! One installed evaluator for descriptive guidance and handler admission.
use super::{
    PreparedRequirementOperand, WorthQueryOperationBodyRequirementFinding as Finding,
    WorthQueryOperationBodyRequirementStatus as Status,
};
use crate::domain_computation::primary_graph::{
    application_contribution::{advancement_denial, QueryRequestExecution},
    operation_body::{BodyFieldExpectation, WorthQueryCurrentBodyRequirementAssessment},
    HandlerExecutionDenial, WorthQueryAdvancementDenial,
    WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryInvariantEntityIdentity,
    WorthQueryManagedComputationResourceDenial,
};
use worth_foundational::expression_api::{
    ExpressionDenialDetail, ExpressionInputs, ExpressionResource, ExpressionStep, MAX_SLICE_QUANTUM,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyProgram, ApplicationOperationBodyRequirementExpression as Expression,
    ApplicationOperationBodyScopeField,
};

pub(in crate::domain_computation::primary_graph) fn evaluate_requirements<
    Schema: worth_query_installation::facade::ApplicationSchema,
    Operation,
    Entity,
>(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>,
    identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    body: &ApplicationOperationBodyProgram,
    execution: &QueryRequestExecution<'_>,
) -> Result<WorthQueryCurrentBodyRequirementAssessment, HandlerExecutionDenial> {
    let (count, bytes) = finding_quote(body)?;
    if count == 0 {
        return Ok(WorthQueryCurrentBodyRequirementAssessment::empty());
    }
    let findings_memory = execution.reserve(bytes).map_err(|cause| {
        HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(cause))
    })?;
    let mut findings = Vec::with_capacity(count);
    for step in body.scope_fields() {
        if let ApplicationOperationBodyScopeField::RequirePresent { field, .. } = step {
            let met = reader.body_field(identity, field, BodyFieldExpectation::Present)?;
            findings.push(Finding::new(field.clone(), met));
        }
    }
    for requirement in body.requirements() {
        let applicable = requirement
            .applicability()
            .map(|expression| evaluate(reader, identity, requirement, expression, execution))
            .transpose()?
            .unwrap_or(true);
        let status = if !applicable {
            Status::NotApplicable
        } else if evaluate(
            reader,
            identity,
            requirement,
            requirement.satisfaction(),
            execution,
        )? {
            Status::Met
        } else {
            Status::Unmet
        };
        findings.push(Finding::evaluated(requirement, status));
    }
    Ok(WorthQueryCurrentBodyRequirementAssessment::new(
        findings,
        findings_memory,
    ))
}

fn finding_quote(
    body: &ApplicationOperationBodyProgram,
) -> Result<(usize, u64), HandlerExecutionDenial> {
    use worth_query_declaration::facade::application_schema::ApplicationOperationBodyRequirementSource as Source;
    let mut count = body.requirements().len();
    let mut bytes = 0_usize;
    let field_bytes = |field: &worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus| field.entity().len().checked_add(field.aspect().len()).and_then(|bytes| bytes.checked_add(field.field().len()));
    for step in body.scope_fields() {
        if let ApplicationOperationBodyScopeField::RequirePresent { field, .. } = step {
            count = count.checked_add(1).ok_or_else(overflow)?;
            bytes = bytes
                .checked_add(field_bytes(field).ok_or_else(overflow)?)
                .ok_or_else(overflow)?;
        }
    }
    for requirement in body.requirements() {
        let target = match requirement.target() {
            Source::RelatedFieldValues {
                relation,
                from,
                to,
                field,
                ..
            } => field_bytes(field)
                .and_then(|bytes| bytes.checked_add(relation.len()))
                .and_then(|bytes| bytes.checked_add(from.len()))
                .and_then(|bytes| bytes.checked_add(to.len())),
            Source::SignedEvidence(source) => source.retained_string_bytes(),
            Source::RelatedSignedEvidence {
                relation,
                from,
                to,
                evidence,
            } => evidence
                .retained_string_bytes()
                .and_then(|bytes| bytes.checked_add(relation.len()))
                .and_then(|bytes| bytes.checked_add(from.len()))
                .and_then(|bytes| bytes.checked_add(to.len())),
            Source::Field { field, .. } | Source::PrincipalIdentity { field, .. } => {
                field_bytes(field)
            }
            Source::RelatedItemCount {
                relation, from, to, ..
            } => relation
                .len()
                .checked_add(from.len())
                .and_then(|bytes| bytes.checked_add(to.len())),
        }
        .ok_or_else(overflow)?;
        bytes = bytes
            .checked_add(target)
            .and_then(|bytes| bytes.checked_add(requirement.identity().len()))
            .ok_or_else(overflow)?;
    }
    let bytes = count
        .checked_mul(size_of::<Finding>())
        .and_then(|slots| slots.checked_add(bytes))
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(overflow)?;
    Ok((count, bytes))
}

fn evaluate<Schema: worth_query_installation::facade::ApplicationSchema, Operation, Entity>(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>,
    identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    requirement: &worth_query_declaration::facade::application_schema::ApplicationOperationBodyRequirementProgram,
    expression: &Expression,
    execution: &QueryRequestExecution<'_>,
) -> Result<bool, HandlerExecutionDenial> {
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
                let source = requirement
                    .operand(name)
                    .expect("admitted operand slot")
                    .source();
                let observed =
                    reader.body_requirement_operand(identity, source, execution, context)?;
                inputs = inputs
                    .bind(name, observed.value.clone())
                    .map_err(super::expression_denial)?;
                held.push(observed);
            }
            let remaining = context.remaining_work();
            if remaining == 0 {
                return Err(exhausted());
            }
            let limit = remaining.min(Expression::PROFILE.limit(ExpressionResource::SemanticWork));
            let profile = Expression::PROFILE
                .narrowed(ExpressionResource::SemanticWork, limit)
                .map_err(super::expression_denial)?;
            let mut continuation = expression
                .compiled()
                .start(&inputs.build(), &profile)
                .map_err(super::expression_denial)?;
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
                            if limit < Expression::PROFILE.limit(ExpressionResource::SemanticWork)
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
                                super::expression_denial(cause)
                            }
                        })?;
                        return Ok(result.as_bool().expect("requirement admitted as Bool"));
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
