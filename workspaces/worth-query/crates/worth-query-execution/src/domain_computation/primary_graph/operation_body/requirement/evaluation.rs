//! One installed evaluator for descriptive guidance and handler admission.
use super::{
    WorthQueryOperationBodyRequirementFinding as Finding,
    WorthQueryOperationBodyRequirementStatus as Status,
};
use crate::domain_computation::primary_graph::{
    application_contribution::QueryRequestExecution,
    operation_body::{BodyFieldExpectation, WorthQueryCurrentBodyRequirementAssessment},
    HandlerExecutionDenial, WorthQueryAdvancementDenial,
    WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryInvariantEntityIdentity,
    WorthQueryManagedComputationResourceDenial,
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
        let status = evaluate_requirement(reader, identity, requirement, execution)?;
        findings.push(Finding::evaluated(requirement, status));
    }
    Ok(WorthQueryCurrentBodyRequirementAssessment::new(
        findings,
        findings_memory,
    ))
}

pub(in crate::domain_computation::primary_graph) fn evaluate_requirement<
    Schema: worth_query_installation::facade::ApplicationSchema,
    Operation,
    Entity,
>(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>,
    identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    requirement: &worth_query_declaration::facade::application_schema::ApplicationOperationBodyRequirementProgram,
    execution: &QueryRequestExecution<'_>,
) -> Result<Status, HandlerExecutionDenial> {
    let applicable = requirement
        .applicability()
        .map(|expression| evaluate(reader, identity, requirement, expression, execution))
        .transpose()?
        .unwrap_or(true);
    if !applicable {
        return Ok(Status::NotApplicable);
    }
    Ok(
        if evaluate(
            reader,
            identity,
            requirement,
            requirement.satisfaction(),
            execution,
        )? {
            Status::Met
        } else {
            Status::Unmet
        },
    )
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
            Source::QueryContext { query, .. } => Some(query.name().len()),
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
    super::super::expression::evaluate(
        reader,
        identity,
        requirement.operands(),
        expression.expression(),
        execution,
        super::super::expression::Purpose::Requirement,
    )
    .map(|value| value.as_bool().expect("requirement admitted as Bool"))
}
fn overflow() -> HandlerExecutionDenial {
    HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(
        WorthQueryManagedComputationResourceDenial::CapacityOverflow,
    ))
}
