//! Scope, real creation and replay survive fresh native reconstruction.
use super::*;
use worth_query_host::facade::{
    application_installation::{
        in_memory_program_from_checkpoint, WorthQueryCheckpointCapturePolicy,
    },
    declaration::application_schema::*,
};

#[test]
fn creation_only_body_survives_checkpoint_and_fresh_readmission() {
    let old = install::<30>();
    assert!(matches!(
        create(&old, 304, input(1)).unwrap(),
        Outcome::Committed { .. }
    ));
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
    let revision = *old.installed_program().revision();
    drop(old);
    let schema = program::schema::<30>();
    let portable = WorthQueryPortableApplicationSchemaRecord::project(schema.erased());
    assert_eq!(
        validate_portable_application_schema_freshly(portable).unwrap(),
        *schema.erased()
    );
    let restored = in_memory_program_from_checkpoint(
        program::validated::<30>(&schema),
        schema,
        (program::Installation::Native,),
        host_limits(),
        checkpoint,
    )
    .unwrap();
    assert_eq!(*restored.installed_program().revision(), revision);
    let replay = create(&restored, 304, input(1)).unwrap();
    // Restart retains the committed idempotency fact, not the old live receipt.
    assert!(
        matches!(replay, Outcome::PreviouslyCommitted(_)),
        "{replay:?}"
    );
    assert_eq!(read_retention(&restored), 7);
    let mut second = input(1);
    second.identity = "restored-new-document".to_owned();
    second.retention_days = 10;
    assert!(matches!(
        create(&restored, 306, second).unwrap(),
        Outcome::Committed { .. }
    ));
    let scope = request_scope();
    let principal = authenticate_operator(restored.runtime().installed_schema(), &scope);
    let read = restored
        .request(&principal, &scope)
        .query(query::DocumentRetentionRead {
            identity: "restored-new-document".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(read.rows().len(), 1);
    assert_eq!(read.rows()[0].retention_days, 10);
    assert!(matches!(invoke(&restored, 305), Outcome::Committed { .. }));
    assert_eq!(read_retention(&restored), 30);
}

#[test]
fn portable_genesis_cannot_replace_its_scope_with_a_foreign_entity() {
    let schema = program::schema::<30>();
    let mut parts =
        WorthQueryPortableApplicationSchemaRecord::project(schema.erased()).into_parts();
    for member in &mut parts.members {
        if let ApplicationSchemaMember::OperationBody { operation, body } = member {
            if operation == "CreateDocument" {
                *body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
                    "Principal".to_owned(),
                    body.scope_fields().to_vec(),
                    body.creation().cloned(),
                    body.requirements().to_vec(),
                );
            }
        }
    }
    parts.members.sort();
    assert_eq!(
        validate_portable_application_schema_freshly(
            WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts),
        )
        .unwrap_err(),
        ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency
    );
}

#[test]
fn explicit_scope_preserves_legacy_field_body_identity_and_cannot_be_effectless() {
    let schema = program::schema::<30>();
    let mut parts =
        WorthQueryPortableApplicationSchemaRecord::project(schema.erased()).into_parts();
    for member in &mut parts.members {
        if let ApplicationSchemaMember::OperationBody { operation, body } = member {
            if operation == "SetRetention" {
                // The typed author used in_scope(Document); its redundant scope
                // must normalize to the same value as the old field-derived form.
                assert_eq!(body.explicit_scope_entity(), None);
                let old = ApplicationOperationBodyProgram::from_untrusted_with_requirements(
                    body.scope_fields().to_vec(),
                    body.creation().cloned(),
                    body.requirements().to_vec(),
                );
                assert_eq!(*body, old);
                *body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
                    "Document".to_owned(),
                    old.scope_fields().to_vec(),
                    old.creation().cloned(),
                    old.requirements().to_vec(),
                );
            }
        }
    }
    assert_eq!(
        validate_portable_application_schema_freshly(
            WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts),
        )
        .unwrap(),
        *schema.erased()
    );
    assert!(matches!(
        declaration::CreateDocument::reference::<DocumentRetentionSchema<30>>()
            .body()
            .in_scope(declaration::Tenant::reference())
            .unwrap()
            .finish(),
        Err(ApplicationOperationBodyAuthoringDenial::Empty)
    ));
}
