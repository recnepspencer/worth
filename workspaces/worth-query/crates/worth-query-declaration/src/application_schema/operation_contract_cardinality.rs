//! Per-operation cardinality for contracts whose absence has meaning.

use std::collections::BTreeSet;

use super::{ApplicationSchemaDeclarationDenial, ApplicationSchemaMember};

/// Proves that each operation has at most one external-effect contract and at
/// most one aftermath contract before the schema receives canonical identity.
pub(super) fn validate_operation_contract_cardinality(
    members: &[ApplicationSchemaMember],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    validate_external_effect_cardinality(members)?;
    validate_inbound_cardinality(members)?;
    validate_aftermath_cardinality(members)
}

fn validate_inbound_cardinality(
    members: &[ApplicationSchemaMember],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    let mut operations = BTreeSet::new();
    for member in members {
        if let ApplicationSchemaMember::OperationInboundOccurrence { operation, .. } = member {
            if !operations.insert(operation.as_str()) {
                return Err(ApplicationSchemaDeclarationDenial::DuplicateMember);
            }
        }
    }
    Ok(())
}

fn validate_external_effect_cardinality(
    members: &[ApplicationSchemaMember],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    let mut operations = BTreeSet::new();
    for member in members {
        if let ApplicationSchemaMember::OperationExternalEffect { operation, .. } = member {
            if !operations.insert(operation.as_str()) {
                return Err(ApplicationSchemaDeclarationDenial::DuplicateOperationExternalEffect);
            }
        }
    }
    Ok(())
}

fn validate_aftermath_cardinality(
    members: &[ApplicationSchemaMember],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    let mut operations = BTreeSet::new();
    for member in members {
        if let ApplicationSchemaMember::OperationAftermath { operation, .. } = member {
            if !operations.insert(operation.as_str()) {
                return Err(ApplicationSchemaDeclarationDenial::DuplicateOperationAftermath);
            }
        }
    }
    Ok(())
}
