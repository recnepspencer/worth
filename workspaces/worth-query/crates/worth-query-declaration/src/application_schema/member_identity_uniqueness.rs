use std::collections::{BTreeMap, BTreeSet};

use super::{ApplicationSchemaDeclarationDenial, ApplicationSchemaMember};
use crate::portable_identity::WorthQueryPortableTypeIdentity;

pub(super) fn validate_member_identity_uniqueness(
    members: &[ApplicationSchemaMember],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    let mut mutations = BTreeSet::new();
    let mut operations = BTreeMap::<&str, &WorthQueryPortableTypeIdentity>::new();
    let mut effects = BTreeMap::<&str, &WorthQueryPortableTypeIdentity>::new();
    let mut query_types = BTreeSet::<&str>::new();
    let mut capability_names = BTreeSet::<&str>::new();
    let mut capability_types = BTreeSet::<WorthQueryPortableTypeIdentity>::new();
    let mut context_types = BTreeSet::<&WorthQueryPortableTypeIdentity>::new();
    let mut slot_types = BTreeSet::<&WorthQueryPortableTypeIdentity>::new();
    let mut provenance_types = BTreeSet::<&WorthQueryPortableTypeIdentity>::new();
    let mut invariants = BTreeSet::new();
    for member in members {
        let duplicate = match member {
            ApplicationSchemaMember::ApplicationMutation { description } => {
                !mutations.insert(description.binding_identity())
            }
            ApplicationSchemaMember::ApplicationInvariant {
                invariant,
                execution_point,
                ..
            } => !invariants.insert((invariant.as_str(), *execution_point)),
            ApplicationSchemaMember::Operation {
                operation,
                input_type,
            } => operations.insert(operation, input_type).is_some(),
            ApplicationSchemaMember::Effect {
                effect,
                payload_type,
            } => effects.insert(effect, payload_type).is_some(),
            ApplicationSchemaMember::ApplicationQuery { definition } => {
                !query_types.insert(definition.query_type())
            }
            ApplicationSchemaMember::ApplicationCapability { contract } => {
                !capability_names.insert(contract.name())
                    || !capability_types.insert(contract.capability_identity())
            }
            ApplicationSchemaMember::ApplicationCapabilityContext { context_type, .. } => {
                !context_types.insert(context_type)
            }
            ApplicationSchemaMember::ApplicationCapabilityContextEntitySlot {
                slot_type, ..
            } => !slot_types.insert(slot_type),
            ApplicationSchemaMember::ApplicationCapabilityProvenance {
                provenance_type, ..
            } => !provenance_types.insert(provenance_type),
            _ => false,
        };
        if duplicate {
            return Err(ApplicationSchemaDeclarationDenial::DuplicateMember);
        }
    }
    Ok(())
}
