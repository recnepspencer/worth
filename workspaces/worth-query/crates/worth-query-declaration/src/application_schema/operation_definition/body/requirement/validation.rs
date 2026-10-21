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
    for requirement in body.observation_requirements() {
        if !names.insert(requirement.identity()) {
            return Err(Denial::DuplicateMember);
        }
    }
    for operand in body.observation_operands() {
        if Some(operand.source().scope_entity()) != body.scope_entity() {
            return Err(Denial::decision_read_body_dependency(operation));
        }
        let bound = match operand.source() {
                Source::QueryContext { mutation_binding, query } => {
                    if !super::query_context::is_declared(operation, mutation_binding, query, members) {
                        return Err(Denial::program_body_dependency(operation));
                    }
                    true
                },
                Source::SignedEvidence(source) => source.is_declared(members),
                Source::RelatedSignedEvidence { relation: name, from: source, to: target, evidence } => {
                    target == &evidence.lifecycle.source_entity
                        && evidence.is_declared(members)
                        && members.iter().any(|member| matches!(member, Member::Relation { relation, from, to, .. }
                            if relation == name && from == source && to == target))
                },
                Source::Field { field: locus, binding, scalar_family: family, presence: posture, presence_only, .. } => members.iter().any(|member| {
                    matches!(member, Member::Field { entity, aspect, field, scalar_family, value_type, presence, unit, .. }
                        if entity == locus.entity() && aspect == locus.aspect() && field == locus.field()
                        && value_type == binding.as_str() && scalar_family == family && presence == posture
                        && (*presence_only || unit.is_none()))
                }),
                Source::RelatedItemCount { relation: name, from: source, to: target, .. } => members.iter().any(|member| {
                    matches!(member, Member::Relation { relation, from, to, .. } if relation == name && from == source && to == target)
                }),
                Source::RelatedFieldValues { relation: name, from: source, to: target, field: locus, binding, scalar_family: family, presence: posture } => {
                    locus.entity() == target && members.iter().any(|member| matches!(member, Member::Relation { relation, from, to, .. } if relation == name && from == source && to == target))
                    && members.iter().any(|member| matches!(member, Member::Field { entity, aspect, field, scalar_family, value_type, presence, unit, .. }
                        if entity == locus.entity() && aspect == locus.aspect() && field == locus.field() && value_type == binding.as_str() && scalar_family == family && presence == posture && unit.is_none()))
                },
                Source::PrincipalIdentity { scope, mutation_binding, principal_binding, field, binding, scalar_family } => {
                    super::super::principal_validation::PrincipalSourceContract { scope, mutation_binding, principal_binding, field, binding, scalar_family: *scalar_family }.is_declared_for(operation, members)
                },
            };
        if !bound || !operand.source().decision_reads().iter().all(|read| members.iter().any(|member| {
                matches!(member, Member::OperationDecisionRead { operation: declared, target } if declared == operation && target == read)
            })) { return Err(Denial::decision_read_body_dependency(operation)); }
    }
    Ok(())
}
