//! Decoded selectors must match the exact fresh schema and derived read closure.
use super::ApplicationOperationBodyRequirementSource as Source;
use crate::application_schema::{
    ApplicationOperationBodyProgram, ApplicationSchemaDeclarationDenial as Denial,
    ApplicationSchemaMember as Member,
};
use std::collections::BTreeSet;

pub(in crate::application_schema::operation_definition::body) fn validate_requirements(
    operation: &str,
    body: &ApplicationOperationBodyProgram,
    members: &[Member],
) -> Result<(), Denial> {
    let mut names = BTreeSet::new();
    for requirement in body.requirements() {
        if !names.insert(requirement.identity()) {
            return Err(Denial::DuplicateMember);
        }
        for operand in requirement.operands() {
            if Some(operand.source().scope_entity()) != body.scope_entity() {
                return Err(Denial::MissingOperationDecisionReadDependency);
            }
            let bound = match operand.source() {
                Source::Field { field: locus, binding, scalar_family: family, presence: posture, presence_only, .. } => members.iter().any(|member| {
                    matches!(member, Member::Field { entity, aspect, field, scalar_family, value_type, presence, unit, .. }
                        if entity == locus.entity() && aspect == locus.aspect() && field == locus.field()
                        && value_type == binding.as_str() && scalar_family == family && presence == posture
                        && (*presence_only || unit.is_none()))
                }),
                Source::RelatedItemCount { relation: name, from: source, to: target, .. } => members.iter().any(|member| {
                    matches!(member, Member::Relation { relation, from, to, .. } if relation == name && from == source && to == target)
                }),
            };
            if !bound || !operand.source().decision_reads().iter().all(|read| members.iter().any(|member| {
                matches!(member, Member::OperationDecisionRead { operation: declared, target } if declared == operation && target == read)
            })) { return Err(Denial::MissingOperationDecisionReadDependency); }
        }
    }
    Ok(())
}
