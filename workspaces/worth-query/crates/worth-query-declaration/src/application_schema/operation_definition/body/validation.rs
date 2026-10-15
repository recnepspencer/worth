//! Validate body closure before canonical identity or installation exists.

use std::collections::BTreeSet;

use crate::application_schema::{
    ApplicationFieldPresence, ApplicationOperationBodyScopeField,
    ApplicationOperationProgramTarget, ApplicationSchemaDeclarationDenial, ApplicationSchemaMember,
};

pub(crate) fn validate_operation_bodies(
    members: &[ApplicationSchemaMember],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    let mut operations = BTreeSet::new();
    for member in members {
        let ApplicationSchemaMember::OperationBody { operation, body } = member else {
            continue;
        };
        if !operations.insert(operation) {
            return Err(ApplicationSchemaDeclarationDenial::DuplicateMember);
        }
        let scope = body
            .scope_entity()
            .ok_or(ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency)?;
        if !members.iter().any(|member| matches!(member,
            ApplicationSchemaMember::Entity { entity } if entity == scope
        )) || (body.scope_fields().is_empty() && !members.iter().any(|member| matches!(member,
            ApplicationSchemaMember::OperationDecisionRead { operation: candidate, target:
                crate::application_schema::ApplicationOperationDecisionReadTarget::Entity { entity } }
                if candidate == operation && entity == scope
        ))) {
            return Err(ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency);
        }
        super::requirement::validate_requirements(operation, body, members)?;
        super::output_validation::validate_outputs(operation, body, members)?;
        let mut writes = BTreeSet::new();
        for step in body.scope_fields() {
            if step.field().entity() != scope {
                return Err(ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency);
            }
            if step.write_target().is_some() && !writes.insert(step.field()) {
                return Err(ApplicationSchemaDeclarationDenial::DuplicateMember);
            }
        }
        if let Some(creation) = body.creation() {
            if creation.key().is_empty() || creation.key().trim() != creation.key() {
                return Err(ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency);
            }
            let creates = ApplicationOperationProgramTarget::Create {
                entity: creation.entity().to_owned(),
            };
            if !members.iter().any(|member| {
                matches!(member, ApplicationSchemaMember::Entity { entity } if entity == creation.entity())
            }) || !members.iter().any(|member| {
                matches!(member, ApplicationSchemaMember::OperationProgram { operation: candidate, target }
                    if candidate == operation && target == &creates)
            }) {
                return Err(ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency);
            }
            let mut initialized = BTreeSet::new();
            for step in creation.fields() {
                let locus = step.field();
                if step.input_is_optional() && !members.iter().any(|member| matches!(member,
                    ApplicationSchemaMember::Field { entity, aspect, field, presence: ApplicationFieldPresence::Optional, .. }
                    if entity == locus.entity() && aspect == locus.aspect() && field == locus.field()
                )) {
                    return Err(ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency);
                }
                if locus.entity() != creation.entity() || !initialized.insert(locus.clone()) {
                    return Err(ApplicationSchemaDeclarationDenial::DuplicateMember);
                }
                if step
                    .input_member()
                    .is_some_and(|member| member.is_empty() || member.trim() != member)
                {
                    return Err(
                        ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency,
                    );
                }
                if let Some(source) = step.source_field() {
                    let target_unit = members.iter().find_map(|member| match member {
                        ApplicationSchemaMember::Field {
                            entity,
                            aspect,
                            field,
                            unit,
                            ..
                        } if entity == locus.entity()
                            && aspect == locus.aspect()
                            && field == locus.field() =>
                        {
                            Some(unit)
                        }
                        _ => None,
                    });
                    if source.entity() != scope
                        || target_unit.is_none()
                        || !members.iter().any(|member| {
                            matches!(member, ApplicationSchemaMember::Field {
                                entity, aspect, field, scalar_family, value_type,
                                unit, presence: ApplicationFieldPresence::Required, ..
                            } if entity == source.entity() && aspect == source.aspect()
                                && field == source.field()
                                && value_type == step.binding().as_str()
                                && *scalar_family == step.scalar_family()
                                && target_unit == Some(unit))
                        })
                        || !members.iter().any(|member| {
                            matches!(member, ApplicationSchemaMember::OperationDecisionRead {
                                operation: candidate, target,
                            } if candidate == operation && Some(target.clone()) == step.source_read())
                        })
                    {
                        return Err(ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency);
                    }
                }
                if !members.iter().any(|member| {
                    matches!(member, ApplicationSchemaMember::Field {
                        entity, aspect, field, scalar_family, value_type, ..
                    } if entity == locus.entity() && aspect == locus.aspect() && field == locus.field()
                        && value_type == step.binding().as_str()
                        && *scalar_family == step.scalar_family())
                }) || !members.iter().any(|member| {
                    matches!(member, ApplicationSchemaMember::OperationProgram { operation: candidate, target }
                        if candidate == operation && target == &step.write_target())
                }) {
                    return Err(ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency);
                }
            }
            for member in members {
                if let ApplicationSchemaMember::Field {
                    entity,
                    aspect,
                    field,
                    presence: ApplicationFieldPresence::Required,
                    ..
                } = member
                {
                    if entity == creation.entity()
                        && !initialized.iter().any(|locus| {
                            locus.entity() == entity
                                && locus.aspect() == aspect
                                && locus.field() == field
                        })
                    {
                        return Err(
                            ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency,
                        );
                    }
                }
            }
        }
        if writes.is_empty() && body.creation().is_none() && body.outputs().is_empty() {
            return Err(ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency);
        }
        if !members.iter().any(|member| {
            matches!(member, ApplicationSchemaMember::Operation { operation: candidate, .. }
                if candidate == operation)
        }) {
            return Err(ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency);
        }
        for step in body.scope_fields() {
            let locus = step.field();
            let expected_presence = if matches!(
                step,
                ApplicationOperationBodyScopeField::SetOptionalInput { .. }
                    | ApplicationOperationBodyScopeField::AssignOptionalInput { .. }
                    | ApplicationOperationBodyScopeField::ClearOptional { .. }
                    | ApplicationOperationBodyScopeField::RequirePresent { .. }
            ) {
                ApplicationFieldPresence::Optional
            } else {
                ApplicationFieldPresence::Required
            };
            if step
                .input_member()
                .is_some_and(|member| member.is_empty() || member.trim() != member)
            {
                return Err(ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency);
            }
            let found = members.iter().any(|member| {
                matches!(member, ApplicationSchemaMember::Field {
                    entity, aspect, field, scalar_family, value_type, presence, writable, ..
                } if entity == locus.entity() && aspect == locus.aspect() && field == locus.field()
                    && value_type == step.binding().as_str()
                    && *scalar_family == step.scalar_family()
                    && *presence == expected_presence
                    && (step.write_target().is_none() || *writable))
            });
            if !found {
                return Err(ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency);
            }
            if !members.iter().any(|member| {
                matches!(member,
                ApplicationSchemaMember::OperationDecisionRead { operation: candidate, target }
                    if candidate == operation && target == &step.decision_read())
            }) {
                return Err(
                    ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency,
                );
            }
            if let Some(write) = step.write_target() {
                if !members.iter().any(|member| {
                    matches!(member,
                    ApplicationSchemaMember::OperationProgram { operation: candidate, target }
                        if candidate == operation && target == &write)
                }) {
                    return Err(
                        ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency,
                    );
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application_schema::{
        ApplicationFieldBindingLocus, ApplicationOperationBodyCreatedField,
        ApplicationOperationBodyCreation, ApplicationOperationBodyProgram,
        ApplicationOperationBodyScopeField,
    };
    use crate::portable_identity::WorthQueryPortableTypeIdentity;
    use worth_foundational::facade::{AspectValue, ScalarAspectType};

    #[test]
    fn portable_scope_initializer_requires_matching_declared_units() {
        let scope = ApplicationFieldBindingLocus::new("Item", "Facts", "Current");
        let source = ApplicationFieldBindingLocus::new("Item", "Facts", "Prior");
        let target = ApplicationFieldBindingLocus::new("Audit", "Facts", "Prior");
        let codec = WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.u64".to_owned());
        let scope_step = ApplicationOperationBodyScopeField::Assign {
            field: scope.clone(),
            binding: codec.clone(),
            value: AspectValue::UInt64(20),
        };
        let created_step = ApplicationOperationBodyCreatedField::ScopeField {
            field: target.clone(),
            binding: codec,
            scalar_family: ScalarAspectType::UInt64,
            source: source.clone(),
        };
        let field =
            |locus: &ApplicationFieldBindingLocus, unit: &str| ApplicationSchemaMember::Field {
                entity: locus.entity().to_owned(),
                aspect: locus.aspect().to_owned(),
                field: locus.field().to_owned(),
                presence: ApplicationFieldPresence::Required,
                scalar_family: ScalarAspectType::UInt64,
                value_type: "worth.rust.u64".to_owned(),
                unit: Some(unit.to_owned()),
                frame: None,
                writable: true,
                equality_queryable: true,
                unique: false,
            };
        let mut members = vec![
            ApplicationSchemaMember::Entity {
                entity: "Audit".to_owned(),
            },
            ApplicationSchemaMember::Operation {
                operation: "Set".to_owned(),
                input_type: WorthQueryPortableTypeIdentity::from_untrusted(
                    "worth.rust.unit".to_owned(),
                ),
            },
            field(&scope, "days"),
            field(&source, "days"),
            field(&target, "days"),
            ApplicationSchemaMember::OperationDecisionRead {
                operation: "Set".to_owned(),
                target: scope_step.decision_read(),
            },
            ApplicationSchemaMember::OperationDecisionRead {
                operation: "Set".to_owned(),
                target: created_step.source_read().unwrap(),
            },
            ApplicationSchemaMember::OperationProgram {
                operation: "Set".to_owned(),
                target: scope_step.write_target().unwrap(),
            },
            ApplicationSchemaMember::OperationProgram {
                operation: "Set".to_owned(),
                target: ApplicationOperationProgramTarget::Create {
                    entity: "Audit".to_owned(),
                },
            },
            ApplicationSchemaMember::OperationProgram {
                operation: "Set".to_owned(),
                target: created_step.write_target(),
            },
            ApplicationSchemaMember::OperationBody {
                operation: "Set".to_owned(),
                body: ApplicationOperationBodyProgram::from_untrusted_with_creation(
                    vec![scope_step],
                    Some(ApplicationOperationBodyCreation::from_untrusted(
                        "Audit".to_owned(),
                        "audit".to_owned(),
                        vec![created_step],
                    )),
                ),
            },
        ];
        assert_eq!(validate_operation_bodies(&members), Ok(()));
        let ApplicationSchemaMember::Field { unit, .. } = &mut members[4] else {
            unreachable!()
        };
        *unit = Some("hours".to_owned());
        assert_eq!(
            validate_operation_bodies(&members),
            Err(ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency),
        );
    }

    #[test]
    fn portable_optional_clear_refuses_a_required_field() {
        let field = ApplicationFieldBindingLocus::new("Item", "Facts", "Note");
        let step = ApplicationOperationBodyScopeField::ClearOptional {
            field: field.clone(),
            binding: WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.string".to_owned()),
            scalar_family: ScalarAspectType::String,
        };
        let mut members = vec![
            ApplicationSchemaMember::Operation {
                operation: "Clear".to_owned(),
                input_type: WorthQueryPortableTypeIdentity::from_untrusted(
                    "worth.rust.unit".to_owned(),
                ),
            },
            ApplicationSchemaMember::Field {
                entity: "Item".to_owned(),
                aspect: "Facts".to_owned(),
                field: "Note".to_owned(),
                presence: ApplicationFieldPresence::Required,
                scalar_family: ScalarAspectType::String,
                value_type: "worth.rust.string".to_owned(),
                unit: None,
                frame: None,
                writable: true,
                equality_queryable: false,
                unique: false,
            },
            ApplicationSchemaMember::OperationDecisionRead {
                operation: "Clear".to_owned(),
                target: step.decision_read(),
            },
            ApplicationSchemaMember::OperationProgram {
                operation: "Clear".to_owned(),
                target: step.write_target().unwrap(),
            },
            ApplicationSchemaMember::OperationBody {
                operation: "Clear".to_owned(),
                body: ApplicationOperationBodyProgram::from_untrusted_scope_fields(vec![step]),
            },
        ];
        assert_eq!(
            validate_operation_bodies(&members),
            Err(ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency),
        );
        let ApplicationSchemaMember::Field { presence, .. } = &mut members[1] else {
            unreachable!()
        };
        *presence = ApplicationFieldPresence::Optional;
        assert_eq!(validate_operation_bodies(&members), Ok(()));
    }
}
