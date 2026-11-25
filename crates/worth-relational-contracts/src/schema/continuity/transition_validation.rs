use crate::schema::data::{
    ProposedSchemaTransition, SchemaReconciliationPolicy, ValidatedSchemaTransition,
};

use super::errors::SchemaTransitionValidationError;
use crate::schema::continuity::transition_classification::{
    classify_schema_transition, is_narrowing,
};

pub fn validate_schema_transition(
    proposed: ProposedSchemaTransition,
    policy: Option<SchemaReconciliationPolicy>,
) -> Result<ValidatedSchemaTransition, SchemaTransitionValidationError> {
    reject_empty_transition(&proposed)?;
    reject_unstratified_transition_atoms(&proposed)?;
    reject_unpolicyed_narrowing_transition(&proposed, policy)?;
    Ok(classify_schema_transition(proposed, policy))
}

fn reject_empty_transition(
    proposed: &ProposedSchemaTransition,
) -> Result<(), SchemaTransitionValidationError> {
    if proposed.diff_atoms.is_empty() {
        return Err(SchemaTransitionValidationError::EmptyDiff);
    }
    Ok(())
}

fn reject_unstratified_transition_atoms(
    proposed: &ProposedSchemaTransition,
) -> Result<(), SchemaTransitionValidationError> {
    for atom in &proposed.diff_atoms {
        if atom.strata.is_empty() {
            return Err(SchemaTransitionValidationError::UnstratifiedChange {
                element_name: atom.element.element_name.clone(),
            });
        }
    }
    Ok(())
}

fn reject_unpolicyed_narrowing_transition(
    proposed: &ProposedSchemaTransition,
    policy: Option<SchemaReconciliationPolicy>,
) -> Result<(), SchemaTransitionValidationError> {
    if policy.is_some() {
        return Ok(());
    }
    for atom in &proposed.diff_atoms {
        if is_narrowing(atom) {
            return Err(SchemaTransitionValidationError::NarrowingWithoutPolicy {
                element_name: atom.element.element_name.clone(),
            });
        }
    }
    Ok(())
}
