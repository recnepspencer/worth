//! A signed artifact has no unsigned alternative creator or mutable record contract.
use super::{
    ApplicationOperationBodyCreatedField as Step, ApplicationOperationBodyCreation as Creation,
};
use crate::application_schema::{
    ApplicationFieldPresence as Presence, ApplicationOperationProgramTarget as Target,
    ApplicationSchemaDeclarationDenial as Denial, ApplicationSchemaMember as Member,
    ApplicationSigningProvenanceEncoding,
};
#[cfg(test)]
mod tests;

pub(super) fn validate_signed_creations(members: &[Member]) -> Result<(), Denial> {
    for member in members {
        let Member::OperationBody { operation, body } = member else {
            continue;
        };
        for creation in body
            .creation()
            .into_iter()
            .flat_map(|c| c.initialization_sites())
        {
            super::classified_capture::validate_creation(creation)
                .map_err(|_| denied(operation))?;
            let provenance_count = creation
                .fields()
                .iter()
                .filter(|field| matches!(field, Step::SigningProvenance { .. }))
                .count();
            let Some(signing) = creation.signing() else {
                if provenance_count != 0 {
                    return Err(denied(operation));
                }
                continue;
            };
            if crate::authentication::WorthQueryAuthenticationPurpose::validate(signing.purpose()).is_err()
            || signing.actor().entity() != creation.entity() || signing.provenance().entity() != creation.entity()
            || provenance_count != 1 || actor(creation).is_none()
            || !creation.fields().iter().any(|field| matches!(field, Step::SigningProvenance { field, binding }
                if field == signing.provenance() && ApplicationSigningProvenanceEncoding::for_binding(binding).is_some())) {
            return Err(denied(operation));
        }
            for target in [signing.actor(), signing.provenance()] {
                if !members.iter().any(|member| matches!(member, Member::Field { entity, aspect, field, presence: Presence::Required, writable: false, unit: None, .. }
                if entity == target.entity() && aspect == target.aspect() && field == target.field())) {
                return Err(denied(operation));
            }
            }
            for member in members {
                match member {
                    Member::Field {
                        entity,
                        aspect,
                        field,
                        presence: Presence::Required,
                        ..
                    } if entity == creation.entity()
                        && !creation.fields().iter().any(|step| {
                            step.field().entity() == entity
                                && step.field().aspect() == aspect
                                && step.field().field() == field
                        }) =>
                    {
                        return Err(denied(operation))
                    }
                    Member::Field {
                        entity,
                        writable: true,
                        ..
                    } if entity == creation.entity() => return Err(denied(operation)),
                    Member::OperationProgram {
                        target: Target::Delete { entity },
                        ..
                    } if entity == creation.entity() => return Err(denied(operation)),
                    Member::OperationProgram {
                        operation,
                        target: Target::Create { entity },
                    } if entity == creation.entity() => {
                        let others = members
                            .iter()
                            .filter_map(|member| match member {
                                Member::OperationBody {
                                    operation: candidate,
                                    body,
                                } if candidate == operation => body.creation(),
                                _ => None,
                            })
                            .flat_map(|root| root.initialization_sites())
                            .filter(|other| other.entity() == entity)
                            .collect::<Vec<_>>();
                        if others.is_empty()
                            || others.into_iter().any(|other| {
                                other.signing().is_none_or(|other_signing| {
                                    other_signing.actor() != signing.actor()
                                        || other_signing.provenance() != signing.provenance()
                                        || actor(other) != actor(creation)
                                })
                            })
                        {
                            return Err(denied(operation));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

/// Different actions may use different purposes/bindings; their actor source cannot drift.
fn actor(
    creation: &Creation,
) -> Option<(
    &str,
    &crate::application_schema::ApplicationFieldBindingLocus,
    &crate::portable_identity::WorthQueryPortableTypeIdentity,
    worth_foundational::facade::ScalarAspectType,
)> {
    let expected = creation.signing()?.actor();
    creation.fields().iter().find_map(|step| match step {
        Step::PrincipalIdentity {
            field,
            principal_binding,
            source,
            binding,
            scalar_family,
            ..
        } if field == expected => {
            Some((principal_binding.as_str(), source, binding, *scalar_family))
        }
        _ => None,
    })
}
fn denied(operation: &str) -> Denial {
    Denial::program_body_dependency(operation)
}
