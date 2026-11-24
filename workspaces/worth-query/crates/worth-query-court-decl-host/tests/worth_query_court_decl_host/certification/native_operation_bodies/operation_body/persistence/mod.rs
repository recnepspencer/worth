pub(crate) use worth_query_testkit_certification_operation::facade::certification::native_operation_bodies::operation_body::persistence::*;

#[test]
fn authored_body_checkpoint_keeps_the_original_program_and_destination() {
    let dir = unique_directory("persistence-dest");
    let old = open_directory::<8>(&dir);
    let revision = *old.runtime().installed_program().revision();
    close_directory(old);
    let successor = install::<9>();
    let restored = open_directory::<8>(&dir);
    assert_eq!(*restored.runtime().installed_program().revision(), revision);
    assert!(matches!(
        invoke(restored.runtime(), 86),
        Outcome::Committed { .. }
    ));
    assert!(matches!(invoke(&successor, 96), Outcome::Committed { .. }));
    assert_eq!(read_retention(restored.runtime()), 8);
    assert_eq!(read_retention(&successor), 9);
    close_directory(restored);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn authored_body_portable_readmission_preserves_values_and_rejects_incomplete_reads() {
    let schema = program::schema::<8>();
    let portable = WorthQueryPortableApplicationSchemaRecord::project(schema.erased());
    let restored = validate_portable_application_schema_freshly(portable.clone()).unwrap();
    assert_eq!(restored, *schema.erased());
    let mut parts = portable.into_parts();
    use worth_query_host::facade::declaration::application_schema::ApplicationOperationDecisionReadTarget;
    for member in &mut parts.members {
        if let ApplicationSchemaMember::OperationDecisionRead { target, .. } = member
            && matches!(target, ApplicationOperationDecisionReadTarget::Field { field, .. }
                if field == "DocumentRetentionField")
        {
            *target = ApplicationOperationDecisionReadTarget::Entity {
                entity: "Document".to_owned(),
            };
        }
    }
    // The same member count and complete single-contribution coverage remain
    // valid. The body must refuse its missing exact field read, not an earlier
    // malformed provenance ordinal or an unrelated dangling member.
    parts.members.sort();
    assert!(matches!(
        validate_portable_application_schema_freshly(
            WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts)
        )
        .unwrap_err(),
        ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency { .. }
    ));
}

#[test]
fn authored_body_input_survives_checkpoint_and_fresh_portable_readmission() {
    let dir = unique_directory("persistence-input");
    let old = open_directory::<100>(&dir);
    let revision = *old.runtime().installed_program().revision();
    close_directory(old);
    let schema = program::schema::<100>();
    let portable = WorthQueryPortableApplicationSchemaRecord::project(schema.erased());
    assert_eq!(
        validate_portable_application_schema_freshly(portable).unwrap(),
        *schema.erased()
    );
    let restored = open_directory::<100>(&dir);
    assert_eq!(*restored.runtime().installed_program().revision(), revision);
    assert!(matches!(
        invoke_input(
            restored.runtime(),
            1005,
            super::input::SetRetentionInput {
                identity: super::DOCUMENT_IDENTITY.to_owned(),
                retention_days: 42,
                ..Default::default()
            }
        ),
        Outcome::Committed { .. }
    ));
    assert_eq!(read_retention(restored.runtime()), 42);
    close_directory(restored);
    let _ = std::fs::remove_dir_all(&dir);
}
