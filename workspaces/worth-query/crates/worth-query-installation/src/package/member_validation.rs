use std::collections::BTreeMap;

use super::conditional_operation_validation::validate_conditional_application_operations;
use super::{
    application_schema_validation::validate_application_schemas,
    artifact_closure::{reject_artifact_contract_conflicts, validate_workflow_artifact_closure},
    WorthQueryPortableDefinition, WorthQueryPortableDefinitionKind,
    WorthQueryPortableDomainPackage, WorthQueryPortablePackageValidationDenial,
};
use crate::{
    domain_operation::WorthQueryPortableDomainOperationDefinition,
    package_requirements::WorthQueryInstallationContributionCategory,
};

pub(super) fn validate_package_members(
    package: &mut WorthQueryPortableDomainPackage,
) -> Result<(), WorthQueryPortablePackageValidationDenial> {
    validate_required_meaning(package)?;
    canonicalize(&mut package.capabilities);
    canonicalize(&mut package.configuration);
    canonicalize(&mut package.operating);
    package.contributions.sort();
    reject_duplicate_contributions(&package.contributions)?;
    package.definitions.sort();
    reject_definition_conflicts(&package.definitions)?;
    package
        .domain_operations
        .sort_by(|left, right| left.identity().cmp(right.identity()));
    reject_domain_operation_conflicts(&package.domain_operations)?;
    validate_artifact_contracts(package)?;
    package.application_schemas.sort_by(|left, right| {
        (left.name(), left.identity()).cmp(&(right.name(), right.identity()))
    });
    validate_application_schemas(&package.identity, &package.application_schemas)?;
    validate_conditional_application_operations(package)?;
    validate_workflow_artifact_closure(&package.domain_operations, &package.artifact_contracts)
}

fn validate_artifact_contracts(
    package: &mut WorthQueryPortableDomainPackage,
) -> Result<(), WorthQueryPortablePackageValidationDenial> {
    package.artifact_contracts.sort_by(|left, right| {
        (
            left.family(),
            left.schema_version(),
            left.protocol_version(),
        )
            .cmp(&(
                right.family(),
                right.schema_version(),
                right.protocol_version(),
            ))
    });
    reject_artifact_contract_conflicts(&package.artifact_contracts)
}

fn validate_required_meaning(
    package: &WorthQueryPortableDomainPackage,
) -> Result<(), WorthQueryPortablePackageValidationDenial> {
    if package.identity.owner().trim().is_empty() {
        return Err(WorthQueryPortablePackageValidationDenial::empty_domain_owner());
    }
    if let Some(definition) = package
        .definitions
        .iter()
        .find(|definition| definition.slot().trim().is_empty())
    {
        return Err(
            WorthQueryPortablePackageValidationDenial::empty_definition_slot(definition.kind()),
        );
    }
    if let Some(definition) = package
        .definitions
        .iter()
        .find(|definition| definition.semantics().trim().is_empty())
    {
        return Err(
            WorthQueryPortablePackageValidationDenial::empty_definition_semantics(
                definition.kind(),
                definition.slot(),
            ),
        );
    }
    if let Some(requirement) = empty_requirement(package) {
        return Err(WorthQueryPortablePackageValidationDenial::empty_requirement(requirement));
    }
    Ok(())
}

fn reject_duplicate_contributions(
    contributions: &[WorthQueryInstallationContributionCategory],
) -> Result<(), WorthQueryPortablePackageValidationDenial> {
    if contributions.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(WorthQueryPortablePackageValidationDenial::duplicate_contribution_category());
    }
    Ok(())
}

fn reject_definition_conflicts(
    definitions: &[WorthQueryPortableDefinition],
) -> Result<(), WorthQueryPortablePackageValidationDenial> {
    let mut slots = BTreeMap::new();
    for definition in definitions {
        let key = (definition.kind(), definition.slot().to_string());
        if let Some(existing) = slots.insert(key, definition.semantics().to_string()) {
            return Err(if existing == definition.semantics() {
                WorthQueryPortablePackageValidationDenial::duplicate_definition(
                    definition.kind(),
                    definition.slot(),
                )
            } else {
                WorthQueryPortablePackageValidationDenial::conflicting_definition(
                    definition.kind(),
                    definition.slot(),
                )
            });
        }
    }
    Ok(())
}

fn reject_domain_operation_conflicts(
    operations: &[WorthQueryPortableDomainOperationDefinition],
) -> Result<(), WorthQueryPortablePackageValidationDenial> {
    for pair in operations.windows(2) {
        if pair[0].identity() != pair[1].identity() {
            continue;
        }
        let slot = pair[1].identity().slot();
        return Err(if pair[0] == pair[1] {
            WorthQueryPortablePackageValidationDenial::duplicate_definition(
                WorthQueryPortableDefinitionKind::DomainOperation,
                slot,
            )
        } else {
            WorthQueryPortablePackageValidationDenial::conflicting_definition(
                WorthQueryPortableDefinitionKind::DomainOperation,
                slot,
            )
        });
    }
    Ok(())
}

fn empty_requirement(package: &WorthQueryPortableDomainPackage) -> Option<&'static str> {
    if package
        .capabilities
        .iter()
        .any(|value| value.as_str().trim().is_empty())
    {
        return Some("capability");
    }
    if package
        .configuration
        .iter()
        .any(|value| value.as_str().trim().is_empty())
    {
        return Some("configuration");
    }
    if package
        .operating
        .iter()
        .any(|value| value.as_str().trim().is_empty())
    {
        return Some("operating");
    }
    if package
        .contributions
        .iter()
        .any(|value| value.as_str().trim().is_empty())
    {
        return Some("contribution");
    }
    None
}

fn canonicalize<T: Ord>(values: &mut Vec<T>) {
    values.sort();
    values.dedup();
}
