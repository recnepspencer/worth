//! Portable syntax cannot weaken a required destination into optional input.
use super::*;
use worth_query_host::facade::declaration::application_schema::*;

#[test]
fn freshly_readmitted_optional_input_forms_require_optional_destinations() {
    reject_required_assignment();
    reject_required_initializer();
}

fn reject_required_assignment() {
    let schema = program::schema::<8>();
    let mut parts =
        WorthQueryPortableApplicationSchemaRecord::project(schema.erased()).into_parts();
    for member in &mut parts.members {
        if let ApplicationSchemaMember::OperationBody { operation, body } = member {
            if operation != "SetRetention" {
                continue;
            }
            let fields = body
                .scope_fields()
                .iter()
                .map(|step| match step {
                    ApplicationOperationBodyScopeField::Assign {
                        field,
                        binding,
                        value,
                    } => ApplicationOperationBodyScopeField::AssignOptionalInput {
                        field: field.clone(),
                        binding: binding.clone(),
                        scalar_family: value.value_family(),
                        member: "retention_days".to_owned(),
                    },
                    other => other.clone(),
                })
                .collect();
            *body = ApplicationOperationBodyProgram::from_untrusted_with_requirements(
                fields,
                body.creation().cloned(),
                body.requirements().to_vec(),
            );
        }
    }
    parts.members.sort();
    assert!(matches!(
        validate_portable_application_schema_freshly(
            WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts)
        )
        .unwrap_err(),
        ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency { .. }
    ));
}

fn reject_required_initializer() {
    let schema = program::schema::<30>();
    let mut parts =
        WorthQueryPortableApplicationSchemaRecord::project(schema.erased()).into_parts();
    for member in &mut parts.members {
        if let ApplicationSchemaMember::OperationBody { operation, body } = member {
            if operation != "CreateDocument" {
                continue;
            }
            let creation = body.creation().unwrap();
            let fields = creation
                .fields()
                .iter()
                .map(|step| match step {
                    ApplicationOperationBodyCreatedField::Input {
                        field,
                        binding,
                        scalar_family,
                        member,
                    } if field.field() == "DocumentRetentionField" => {
                        ApplicationOperationBodyCreatedField::OptionalInput {
                            field: field.clone(),
                            binding: binding.clone(),
                            scalar_family: *scalar_family,
                            member: member.clone(),
                        }
                    }
                    other => other.clone(),
                })
                .collect();
            let changed = ApplicationOperationBodyCreation::from_untrusted(
                creation.entity().to_owned(),
                creation.key().to_owned(),
                fields,
            );
            *body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
                body.scope_entity().unwrap().to_owned(),
                body.scope_fields().to_vec(),
                Some(changed),
                body.requirements().to_vec(),
            );
        }
    }
    parts.members.sort();
    assert!(matches!(
        validate_portable_application_schema_freshly(
            WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts)
        )
        .unwrap_err(),
        ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency { .. }
    ));
}
