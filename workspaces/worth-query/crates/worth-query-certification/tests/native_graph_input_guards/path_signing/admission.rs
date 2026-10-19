//! Cold no-lifecycle signing preserves the same Subject-write law as typed finish.
use super::super::program;
use worth_query_decl::facade::application_schema::*;
#[test]
fn cold_signed_path_write_refuses_a_direct_subject_conflict_without_a_lifecycle() {
    let schema = program::schema();
    let record = WorthQueryPortableApplicationSchemaRecord::project(schema.erased());
    assert!(validate_portable_application_schema_freshly(record.clone()).is_ok());
    let mut parts = record.into_parts();
    let mut changed = false;
    for member in &mut parts.members {
        if let ApplicationSchemaMember::OperationBody { operation, body } = member
            && operation == "GraphSign"
        {
            let creation = body.creation().unwrap();
            assert!(creation.signing_lifecycle().is_none());
            let mut fields = creation.fields().to_vec();
            for field in &mut fields {
                if let ApplicationOperationBodyCreatedField::CapturedPathField {
                    source, role, ..
                } = field
                    && source.field.field() == "ParentRevision"
                {
                    assert_eq!(*role, ApplicationOperationBodyCaptureRole::AuditContext);
                    *role = ApplicationOperationBodyCaptureRole::Subject;
                    changed = true;
                }
            }
            let creation = ApplicationOperationBodyCreation::from_untrusted(
                creation.entity().into(),
                creation.key().into(),
                fields,
            )
            .with_untrusted_signing(creation.signing().cloned())
            .with_untrusted_links(creation.links().to_vec());
            *body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
                "Row".into(),
                body.scope_fields().to_vec(),
                Some(creation),
                vec![],
            )
            .with_untrusted_values(body.values().to_vec())
            .with_untrusted_path_value_assignments(body.path_value_assignments().to_vec());
        }
    }
    assert!(changed);
    parts.members.sort();
    assert_eq!(
        validate_portable_application_schema_freshly(
            WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts)
        )
        .unwrap_err(),
        ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency
    );
}
