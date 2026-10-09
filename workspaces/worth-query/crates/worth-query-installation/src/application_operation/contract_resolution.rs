use worth_query_declaration::facade::application_schema::{
    ApplicationMutationPreconditionTarget, ApplicationOperationDecisionReadTarget,
    ApplicationOperationProgramTarget, ApplicationSchema, ApplicationSchemaMember,
};

use crate::application_schema::WorthQueryInstalledApplicationSchema;

use super::installed_contract_support::operation_denial;
use super::{
    WorthQueryApplicationOperationInstallationDenial,
    WorthQueryApplicationOperationInstallationDenialKind, WorthQueryInstalledAbilityRequirement,
    WorthQueryInstalledApplicationOperationExecutionPosture,
};
use worth_query_declaration::facade::application_capability::{
    application_capability_delegation_activation_program_targets,
    application_capability_revocation_decision_reads,
    application_capability_revocation_program_target,
};

mod aftermath;
mod cardinality_denial;
mod elevation_lifecycle_program;
#[cfg(test)]
mod external_effect;
pub(super) use aftermath::operation_aftermath;
pub(super) use cardinality_denial::WorthQueryOperationContractCardinalityDenial;
use elevation_lifecycle_program::{lifecycle_program_targets, lifecycle_resource_decision_read};
#[cfg(test)]
pub(super) use external_effect::operation_external_effect;

#[cfg(test)]
mod tests;

pub(super) fn operation_execution_posture(
    members: &[ApplicationSchemaMember],
    operation: &str,
    input_type: &str,
) -> WorthQueryInstalledApplicationOperationExecutionPosture {
    if members.iter().any(|member| {
        let ApplicationSchemaMember::ApplicationCapability { contract } = member else {
            return false;
        };
        contract
            .delegation()
            .activation()
            .is_some_and(|activation| {
                activation.operation().operation() == operation
                    && activation.operation().input_type() == input_type
            })
    }) {
        WorthQueryInstalledApplicationOperationExecutionPosture::DelegationActivation
    } else if members.iter().any(|member| {
        let ApplicationSchemaMember::ApplicationCapability { contract } = member else {
            return false;
        };
        contract
            .delegation()
            .revocation()
            .is_some_and(|revocation| {
                revocation.operation().operation() == operation
                    && revocation.operation().input_type() == input_type
            })
    }) {
        WorthQueryInstalledApplicationOperationExecutionPosture::CapabilityRevocation
    } else {
        WorthQueryInstalledApplicationOperationExecutionPosture::Ordinary
    }
}

pub(super) fn ability_requirement_meaning_matches(
    members: &[ApplicationSchemaMember],
    operation: &str,
    installed: &[WorthQueryInstalledAbilityRequirement],
) -> bool {
    let mut declared = members
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::OperationAbility {
                operation: candidate,
                ability,
                scope_entity,
            } if candidate == operation => Some((ability.as_str(), scope_entity.as_str())),
            _ => None,
        })
        .collect::<Vec<_>>();
    declared.sort_unstable();
    declared.dedup();
    declared.len() == installed.len()
        && declared.into_iter().all(|(ability, scope_entity)| {
            installed.iter().any(|requirement| {
                requirement.ability() == ability
                    && requirement.scope_entity() == scope_entity
                    && members.iter().any(|member| {
                        matches!(
                            member,
                            ApplicationSchemaMember::AbilityPolicy {
                                ability: candidate_ability,
                                scope_entity: candidate_scope,
                                policy,
                                paths,
                            } if candidate_ability == ability
                                && candidate_scope == scope_entity
                                && policy == requirement.policy()
                                && paths.len() == requirement.policy_paths().len()
                                && paths.iter().zip(requirement.policy_paths()).all(
                                    |(path, installed_path)| path == installed_path.path()
                                )
                        )
                    })
            })
        })
}

pub(super) fn operation_projection_work_budget(
    members: &[ApplicationSchemaMember],
    operation: &str,
) -> Option<usize> {
    members.iter().find_map(|member| match member {
        ApplicationSchemaMember::OperationProjectionWorkBudget {
            operation: installed,
            maximum_work_units,
        } if installed == operation => Some(*maximum_work_units),
        _ => None,
    })
}

pub(super) fn operation_decision_reads_from_members(
    members: &[ApplicationSchemaMember],
    operation: &str,
    input_type: &str,
) -> Vec<ApplicationOperationDecisionReadTarget> {
    let mut reads = if operation_has_revocation(members, operation) {
        members
            .iter()
            .filter_map(|member| match member {
                ApplicationSchemaMember::ApplicationCapability { contract }
                    if contract
                        .delegation()
                        .revocation()
                        .is_some_and(|revocation| {
                            revocation.operation().operation() == operation
                        }) =>
                {
                    application_capability_revocation_decision_reads(contract)
                }
                _ => None,
            })
            .flatten()
            .collect::<Vec<_>>()
    } else {
        members
            .iter()
            .filter_map(|member| match member {
                ApplicationSchemaMember::OperationDecisionRead {
                    operation: installed,
                    target,
                } if installed == operation => Some(target.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    reads.sort();
    reads.dedup();
    reads.extend(lifecycle_resource_decision_read(
        members, operation, input_type,
    ));
    reads.sort();
    reads.dedup();
    reads
}

pub(super) fn operation_mutation_preconditions(
    members: &[ApplicationSchemaMember],
    operation: &str,
) -> Vec<ApplicationMutationPreconditionTarget> {
    members
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::OperationMutationPrecondition {
                operation: installed,
                target,
            } if installed == operation => Some(target.clone()),
            _ => None,
        })
        .collect()
}

pub(super) fn ability_requirements<Schema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    operation: &str,
) -> Result<
    Vec<WorthQueryInstalledAbilityRequirement>,
    WorthQueryApplicationOperationInstallationDenial,
>
where
    Schema: ApplicationSchema,
{
    ability_requirements_from_schema(schema, operation)
}

fn ability_requirements_from_schema<Schema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    operation: &str,
) -> Result<
    Vec<WorthQueryInstalledAbilityRequirement>,
    WorthQueryApplicationOperationInstallationDenial,
>
where
    Schema: ApplicationSchema,
{
    let mut requirements = Vec::new();
    for member in schema.installed_declaration().members() {
        let requirement = match member {
            ApplicationSchemaMember::OperationAbility {
                operation: installed,
                ability,
                scope_entity,
            } if installed == operation => {
                let requirement = schema
                    .installed_ability_requirement(ability, scope_entity)
                    .cloned()
                    .ok_or_else(|| {
                        operation_denial(
                            WorthQueryApplicationOperationInstallationDenialKind::MissingAbilityPolicy,
                            operation,
                        )
                    })?;
                Some(requirement)
            }
            _ => None,
        };
        requirements.extend(requirement);
    }
    requirements.sort();
    requirements.dedup();
    Ok(requirements)
}

pub(super) fn operation_program_from_members(
    members: &[ApplicationSchemaMember],
    operation: &str,
    input_type: &str,
) -> Vec<ApplicationOperationProgramTarget> {
    let posture = operation_execution_posture(members, operation, input_type);
    let mut program = if posture.requires_delegation_activation() {
        members
            .iter()
            .filter_map(|member| match member {
                ApplicationSchemaMember::ApplicationCapability { contract }
                    if contract
                        .delegation()
                        .activation()
                        .is_some_and(|activation| {
                            activation.operation().operation() == operation
                                && activation.operation().input_type() == input_type
                        }) =>
                {
                    application_capability_delegation_activation_program_targets(contract)
                }
                _ => None,
            })
            .flatten()
            .collect::<Vec<_>>()
    } else if posture.requires_capability_revocation() {
        members
            .iter()
            .filter_map(|member| match member {
                ApplicationSchemaMember::ApplicationCapability { contract }
                    if contract
                        .delegation()
                        .revocation()
                        .is_some_and(|revocation| {
                            revocation.operation().operation() == operation
                                && revocation.operation().input_type() == input_type
                        }) =>
                {
                    application_capability_revocation_program_target(contract)
                }
                _ => None,
            })
            .collect::<Vec<_>>()
    } else {
        members
            .iter()
            .filter_map(|member| match member {
                ApplicationSchemaMember::OperationProgram {
                    operation: installed,
                    target,
                } if installed == operation => Some(target.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    program.extend(lifecycle_program_targets(members, operation, input_type));
    program.sort();
    program.dedup();
    program
}

fn operation_has_revocation(members: &[ApplicationSchemaMember], operation: &str) -> bool {
    members.iter().any(|member| {
        matches!(
            member,
            ApplicationSchemaMember::ApplicationCapability { contract }
                if contract.delegation().revocation().is_some_and(|revocation| {
                    revocation.operation().operation() == operation
                })
        )
    })
}
