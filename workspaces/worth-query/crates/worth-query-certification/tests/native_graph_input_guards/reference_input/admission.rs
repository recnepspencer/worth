//! Cold authority closes relation cardinality, identity codec and derived contracts.
use super::super::program;
use worth_query_decl::facade::application_schema::*;
#[test]
fn cold_reference_input_rejects_mandatory_relation_missing_contract_and_forged_identity() {
    let schema = program::schema();
    let record = WorthQueryPortableApplicationSchemaRecord::project(schema.erased());
    assert!(validate_portable_application_schema_freshly(record.clone()).is_ok());
    for defect in [
        "mandatory",
        "unbounded",
        "unique",
        "read",
        "link",
        "codec",
        "duplicate",
        "member",
    ] {
        let mut parts = record.clone().into_parts();
        let mut changed = false;
        for member in &mut parts.members {
            match member {
                ApplicationSchemaMember::Field {
                    entity,
                    field,
                    unique,
                    ..
                } if defect == "unique" && entity == "Parent" && field == "ParentIdentity" => {
                    *unique = false;
                    changed = true;
                }
                ApplicationSchemaMember::Relation {
                    relation,
                    integrity,
                    ..
                } if relation == "SelectedParent"
                    && matches!(defect, "mandatory" | "unbounded") =>
                {
                    integrity.cardinality = ApplicationRelationCardinality::new(
                        if defect == "mandatory" { Some(1) } else { None },
                        if defect == "unbounded" { None } else { Some(1) },
                        None,
                        None,
                        None,
                        None,
                    );
                    changed = true;
                }
                ApplicationSchemaMember::OperationDecisionRead { operation, target }
                    if operation == "ReplaceReference"
                        && defect == "read"
                        && matches!(&*target,ApplicationOperationDecisionReadTarget::Field{field,..} if field=="ParentIdentity") =>
                {
                    *target = ApplicationOperationDecisionReadTarget::Entity {
                        entity: "Audit".into(),
                    };
                    changed = true;
                }
                ApplicationSchemaMember::OperationProgram { operation, target }
                    if operation == "ReplaceReference"
                        && defect == "link"
                        && matches!(&*target, ApplicationOperationProgramTarget::Link { .. }) =>
                {
                    *target = ApplicationOperationProgramTarget::Create {
                        entity: "Audit".into(),
                    };
                    changed = true;
                }
                ApplicationSchemaMember::OperationBody { operation, body }
                    if operation == "ReplaceReference"
                        && matches!(defect, "codec" | "duplicate" | "member") =>
                {
                    let mut references = body.reference_inputs().to_vec();
                    match defect {
                        "codec" => references[0].binding = U64ApplicationValueBinding::IDENTITY,
                        "duplicate" => references.push(references[0].clone()),
                        "member" => references[0].member = " selected".into(),
                        _ => unreachable!(),
                    }
                    *body = body.clone().with_untrusted_reference_inputs(references);
                    changed = true;
                }
                _ => {}
            }
        }
        assert!(changed, "{defect}");
        parts.members.sort();
        let denied = validate_portable_application_schema_freshly(
            WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts),
        )
        .unwrap_err();
        let expected = if defect == "read" {
            ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency
        } else {
            ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency
        };
        assert_eq!(denied, expected, "{defect}");
    }
}

#[test]
fn cold_signing_cannot_retarget_its_own_certified_subject_path() {
    let schema = program::schema();
    let record = WorthQueryPortableApplicationSchemaRecord::project(schema.erased());
    assert!(validate_portable_application_schema_freshly(record.clone()).is_ok());
    let mut parts = record.into_parts();
    let reference = parts
        .members
        .iter()
        .find_map(|member| match member {
            ApplicationSchemaMember::OperationBody { operation, body }
                if operation == "ReplaceReference" =>
            {
                Some(body.reference_inputs()[0].clone())
            }
            _ => None,
        })
        .unwrap();
    // Supply the new contracts too, so missing permission cannot counterfeit the Subject conflict.
    for target in reference.program_targets() {
        parts
            .members
            .push(ApplicationSchemaMember::OperationProgram {
                operation: "SignReference".into(),
                target,
            });
    }
    parts.members.sort();
    // This fixture has one real contribution. Keep its exact member inventory
    // complete after adding the two inert program contracts.
    assert_eq!(parts.contributions.len(), 1);
    parts.contributions[0] = ApplicationSchemaContributionProvenance::from_untrusted_parts(
        parts.contributions[0].identity().as_str().into(),
        (0..parts.members.len())
            .map(|index| u32::try_from(index).unwrap())
            .collect(),
    );
    let supplied = WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts);
    assert!(
        validate_portable_application_schema_freshly(supplied.clone()).is_ok(),
        "extra contracts alone must be valid"
    );
    let mut parts = supplied.into_parts();
    for member in &mut parts.members {
        if let ApplicationSchemaMember::OperationBody { operation, body } = member
            && operation == "SignReference"
        {
            *body = body
                .clone()
                .with_untrusted_reference_inputs(vec![reference.clone()]);
        }
    }
    parts.members.sort();
    assert_eq!(
        validate_portable_application_schema_freshly(
            WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts)
        )
        .unwrap_err(),
        ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency
    );
}
