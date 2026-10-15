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
        let Member::OperationBody { body, .. } = member else {
            continue;
        };
        let Some(creation) = body.creation() else {
            continue;
        };
        super::classified_capture::validate_creation(creation).map_err(|_| denied())?;
        let provenance_count = creation
            .fields()
            .iter()
            .filter(|field| matches!(field, Step::SigningProvenance { .. }))
            .count();
        let Some(signing) = creation.signing() else {
            if provenance_count != 0 {
                return Err(denied());
            }
            continue;
        };
        if crate::authentication::WorthQueryAuthenticationPurpose::validate(signing.purpose()).is_err()
            || signing.actor().entity() != creation.entity() || signing.provenance().entity() != creation.entity()
            || provenance_count != 1 || actor(creation).is_none()
            || !creation.fields().iter().any(|field| matches!(field, Step::SigningProvenance { field, binding }
                if field == signing.provenance() && ApplicationSigningProvenanceEncoding::for_binding(binding).is_some())) {
            return Err(denied());
        }
        for target in [signing.actor(), signing.provenance()] {
            if !members.iter().any(|member| matches!(member, Member::Field { entity, aspect, field, presence: Presence::Required, writable: false, unit: None, .. }
                if entity == target.entity() && aspect == target.aspect() && field == target.field())) {
                return Err(denied());
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
                    return Err(denied())
                }
                Member::Field {
                    entity,
                    writable: true,
                    ..
                } if entity == creation.entity() => return Err(denied()),
                Member::OperationProgram {
                    target: Target::Delete { entity },
                    ..
                } if entity == creation.entity() => return Err(denied()),
                Member::OperationProgram {
                    operation,
                    target: Target::Create { entity },
                } if entity == creation.entity() => {
                    let Some(other) = members
                        .iter()
                        .find_map(|member| match member {
                            Member::OperationBody {
                                operation: candidate,
                                body,
                            } if candidate == operation => body.creation(),
                            _ => None,
                        })
                        .filter(|other| other.entity() == entity)
                    else {
                        return Err(denied());
                    };
                    let Some(other_signing) = other.signing() else {
                        return Err(denied());
                    };
                    if other_signing.actor() != signing.actor()
                        || other_signing.provenance() != signing.provenance()
                        || actor(other) != actor(creation)
                    {
                        return Err(denied());
                    }
                }
                _ => {}
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
fn denied() -> Denial {
    Denial::MissingOperationProgramDependency
}
