use std::collections::{BTreeMap, BTreeSet};

use crate::application_capability::{
    ApplicationCapabilityElevationRule, ErasedApplicationCapabilityContract,
};

use super::member_closure::ClosureIndex;
use super::{ApplicationSchemaDeclarationDenial, ApplicationSchemaMember};

mod canonical_structure;
mod capability_revocation_program;
mod composition;
mod declared_dimensions;
mod delegation_activation_program;
mod dependencies;
mod elevation_lifecycle_program;
mod topology;

use canonical_structure::validate_canonical_structure;
use capability_revocation_program::revocation_programs_are_framework_owned;
use composition::composition_is_closed;
use declared_dimensions::DeclaredCapabilityDimensions;
use delegation_activation_program::{
    activation_identities_are_unique, activation_programs_are_framework_owned,
};
use dependencies::dependencies_are_closed;
use elevation_lifecycle_program::lifecycle_program_targets_are_framework_owned;
use topology::topology_is_valid;

const MAXIMUM_CAPABILITY_CONTRACTS: usize = 1_024;

pub(super) fn validate_application_capability_members(
    members: &[ApplicationSchemaMember],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    let dimensions = DeclaredCapabilityDimensions::validate(members)?;
    let closure = ClosureIndex::new(members);
    let contracts = capability_contracts(members);
    if contracts.len() > MAXIMUM_CAPABILITY_CONTRACTS {
        return Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability);
    }
    if !lifecycle_operations_have_one_owner(&contracts) {
        return Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability);
    }
    if !specialized_operations_have_one_posture(&contracts) {
        return Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability);
    }
    let mut identities = BTreeSet::new();
    for contract in &contracts {
        if !identities.insert((contract.name(), contract.capability_type())) {
            return Err(ApplicationSchemaDeclarationDenial::DuplicateApplicationCapability);
        }
        validate_canonical_structure(contract)?;
        validate_contract(members, &closure, &dimensions, contract)?;
    }
    if !activation_programs_are_framework_owned(members, &contracts) {
        return Err(
            ApplicationSchemaDeclarationDenial::InvalidApplicationCapabilityDelegationActivationProgram,
        );
    }
    if !activation_identities_are_unique(members, &contracts) {
        return Err(ApplicationSchemaDeclarationDenial::DelegationActivationIdentityNotUnique);
    }
    if !revocation_programs_are_framework_owned(members, &contracts) {
        return Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability);
    }
    if !lifecycle_program_targets_are_framework_owned(members, &contracts) {
        return Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability);
    }
    Ok(())
}

fn lifecycle_operations_have_one_owner(contracts: &[&ErasedApplicationCapabilityContract]) -> bool {
    let mut operations = BTreeSet::new();
    contracts.iter().all(|contract| {
        let ApplicationCapabilityElevationRule::Governed(elevation) = contract.elevation() else {
            return true;
        };
        elevation
            .lifecycle()
            .transitions()
            .into_iter()
            .all(|transition| {
                let operation = transition.operation();
                operations.insert((operation.operation(), operation.input_type()))
            })
    })
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum SpecializedExecutionPosture {
    DelegationActivation,
    CapabilityRevocation,
    ElevationTransition,
}

fn specialized_operations_have_one_posture(
    contracts: &[&ErasedApplicationCapabilityContract],
) -> bool {
    let mut operations = BTreeMap::new();
    for contract in contracts {
        let delegation = contract.delegation();
        if let Some(activation) = delegation.activation() {
            if !record_posture(
                &mut operations,
                activation.operation(),
                SpecializedExecutionPosture::DelegationActivation,
            ) {
                return false;
            }
        }
        if let Some(revocation) = delegation.revocation() {
            if !record_posture(
                &mut operations,
                revocation.operation(),
                SpecializedExecutionPosture::CapabilityRevocation,
            ) {
                return false;
            }
        }
        if let ApplicationCapabilityElevationRule::Governed(elevation) = contract.elevation() {
            for transition in elevation.lifecycle().transitions() {
                if !record_posture(
                    &mut operations,
                    transition.operation(),
                    SpecializedExecutionPosture::ElevationTransition,
                ) {
                    return false;
                }
            }
        }
    }
    true
}

fn record_posture<'a>(
    operations: &mut BTreeMap<(&'a str, &'a str), SpecializedExecutionPosture>,
    operation: &'a crate::application_capability::ApplicationCapabilityOperationBinding,
    posture: SpecializedExecutionPosture,
) -> bool {
    operations
        .entry((operation.operation(), operation.input_type()))
        .or_insert(posture)
        == &posture
}

fn capability_contracts(
    members: &[ApplicationSchemaMember],
) -> Vec<&ErasedApplicationCapabilityContract> {
    members
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::ApplicationCapability { contract } => Some(contract),
            _ => None,
        })
        .collect()
}

fn validate_contract(
    members: &[ApplicationSchemaMember],
    closure: &ClosureIndex<'_>,
    dimensions: &DeclaredCapabilityDimensions<'_>,
    contract: &ErasedApplicationCapabilityContract,
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    dependencies_are_closed(members, dimensions, contract)?;
    if !topology_is_valid(contract) || !composition_is_closed(closure, dimensions, contract) {
        return Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability);
    }
    Ok(())
}
