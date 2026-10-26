//! Retained-body reconstruction and authority-free portable readmission.
use super::{host_limits, install, invoke, invoke_input, program, read_retention, Outcome};
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
use worth_query_host::facade::{
    application_installation::{
        in_memory_program_from_checkpoint, WorthQueryCheckpointCapturePolicy,
    },
    declaration::application_schema::{
        validate_portable_application_schema_freshly, ApplicationSchemaDeclarationDenial,
        ApplicationSchemaMember, WorthQueryPortableApplicationSchemaRecord,
    },
};

#[test]
fn authored_body_checkpoint_keeps_the_original_program_and_destination() {
    let old = install::<8>();
    let revision = *old.installed_program().revision();
    let checkpoint = old
        .with_host_advancement(|phase| {
            old.runtime()
                .capture_application_checkpoint(
                    &phase,
                    WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    drop(old);
    let successor = install::<9>();
    let schema = program::schema::<8>();
    let restored = in_memory_program_from_checkpoint(
        program::validated::<8>(&schema),
        schema,
        (program::Installation::Native,),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host_limits(),
        checkpoint,
    )
    .unwrap();
    assert_eq!(*restored.installed_program().revision(), revision);
    assert!(matches!(invoke(&restored, 86), Outcome::Committed { .. }));
    assert!(matches!(invoke(&successor, 96), Outcome::Committed { .. }));
    assert_eq!(read_retention(&restored), 8);
    assert_eq!(read_retention(&successor), 9);
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
    let old = install::<100>();
    let revision = *old.installed_program().revision();
    let checkpoint = old
        .with_host_advancement(|phase| {
            old.runtime()
                .capture_application_checkpoint(
                    &phase,
                    WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    drop(old);
    let schema = program::schema::<100>();
    let portable = WorthQueryPortableApplicationSchemaRecord::project(schema.erased());
    assert_eq!(
        validate_portable_application_schema_freshly(portable).unwrap(),
        *schema.erased()
    );
    let restored = in_memory_program_from_checkpoint(
        program::validated::<100>(&schema),
        schema,
        (program::Installation::Native,),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host_limits(),
        checkpoint,
    )
    .unwrap();
    assert_eq!(*restored.installed_program().revision(), revision);
    assert!(matches!(
        invoke_input(
            &restored,
            1005,
            super::input::SetRetentionInput {
                identity: super::DOCUMENT_IDENTITY.to_owned(),
                retention_days: 42,
                ..Default::default()
            }
        ),
        Outcome::Committed { .. }
    ));
    assert_eq!(read_retention(&restored), 42);
}
