//! Fresh readmission refuses partial content, missing read closure and created-slot promises.
use super::program;
use worth_query_decl::facade::application_schema::*;
#[test]
fn portable_reuse_requires_complete_installed_content_and_comparison_reads() {
    let schema = program::schema();
    let record = WorthQueryPortableApplicationSchemaRecord::project(schema.erased());
    assert!(validate_portable_application_schema_freshly(record.clone()).is_ok());
    for defect in [
        "uninitialized-optional",
        "missing-read",
        "ordinary-replacement",
        "computed-source",
    ] {
        let mut parts = record.clone().into_parts();
        let mut changed = false;
        for member in &mut parts.members {
            match member {
                ApplicationSchemaMember::OperationBody { operation, body }
                    if operation == "Reuse" && defect != "missing-read" =>
                {
                    let creation = body.creation().unwrap();
                    let mut fields = creation.fields().to_vec();
                    let mut links = creation.links().to_vec();
                    match defect {
                        "uninitialized-optional" => {
                            fields.retain(|field| field.field().field() != "SnapshotNote")
                        }
                        "ordinary-replacement" => {
                            links[0].posture =
                                ApplicationOperationBodyCreationLinkPosture::ReplaceSingle
                        }
                        "computed-source" => {
                            let field = fields[1].field().clone();
                            let binding = fields[1].binding().clone();
                            fields[1] = ApplicationOperationBodyCreatedField::Computed {
                                field,
                                binding,
                                value: "next".into(),
                            };
                        }
                        _ => unreachable!(),
                    }
                    *body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
                        "Row".into(),
                        body.scope_fields().to_vec(),
                        Some(
                            ApplicationOperationBodyCreation::from_untrusted(
                                creation.entity().into(),
                                creation.key().into(),
                                fields,
                            )
                            .with_untrusted_links(links)
                            .with_untrusted_reuse_current_if_unchanged(true),
                        ),
                        vec![],
                    )
                    .with_untrusted_values(body.values().to_vec())
                    .with_untrusted_path_value_assignments(body.path_value_assignments().to_vec());
                    changed = true;
                }
                ApplicationSchemaMember::OperationDecisionRead { operation, target }
                    if operation == "Reuse"
                        && defect == "missing-read"
                        && matches!(&*target,ApplicationOperationDecisionReadTarget::Field{field,..} if field == "SnapshotNote") =>
                {
                    *target = ApplicationOperationDecisionReadTarget::Entity {
                        entity: "Audit".into(),
                    };
                    changed = true;
                }
                _ => {}
            }
        }
        assert!(changed, "{defect}");
        parts.members.sort();
        let error = validate_portable_application_schema_freshly(
            WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts),
        )
        .unwrap_err();
        let expected = if defect == "missing-read" {
            ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency
        } else {
            ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency
        };
        assert_eq!(error, expected, "{defect}");
    }
}
