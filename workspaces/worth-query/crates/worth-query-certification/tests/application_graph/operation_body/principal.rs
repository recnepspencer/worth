//! Native actor capture is binding-affine, replay-safe, and independent of caller values.
use super::*;
use worth_query_host::facade::{
    application_installation::{
        in_memory_program_from_checkpoint, WorthQueryCheckpointCapturePolicy,
    },
    declaration::{application_schema::*, portable_identity::WorthQueryPortableTypeIdentity},
};

#[test]
fn admitted_actor_survives_reconstruction_and_cannot_be_chosen_by_input() {
    let old = install::<36>();
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
    let schema = program::schema::<36>();
    let restored = in_memory_program_from_checkpoint(
        program::validated::<36>(&schema),
        schema,
        (program::Installation::Native,),
        host_limits(),
        checkpoint,
    )
    .unwrap();
    // invoke supplies retention_days=99. The admitted native principal is 1.
    assert!(matches!(invoke(&restored, 360), Outcome::Committed { .. }));
    assert!(matches!(
        invoke(&restored, 360),
        Outcome::AlreadyCommitted(_)
    ));
    let scope = request_scope();
    let principal = authenticate_operator(restored.runtime().installed_schema(), &scope);
    let result = restored
        .request(&principal, &scope)
        .query(audit_query::AuditRead {
            identity: DOCUMENT_IDENTITY.to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(result.rows().len(), 1);
    let row = &result.rows()[0];
    assert_eq!(
        (row.actor, row.retention_days, row.prior_retention_days),
        (Some(1), 36, 7)
    );
    assert_eq!(read_retention(&restored), 36);
}

#[test]
fn portable_actor_contract_refuses_missing_read_unit_codec_and_binding() {
    let schema = program::schema::<36>();
    let portable = WorthQueryPortableApplicationSchemaRecord::project(schema.erased());
    assert_eq!(
        validate_portable_application_schema_freshly(portable.clone()).unwrap(),
        *schema.erased()
    );
    for defect in ["read", "unit", "codec", "binding", "principal", "source"] {
        let mut parts = portable.clone().into_parts();
        for member in &mut parts.members {
            match member {
                ApplicationSchemaMember::OperationDecisionRead { operation, target }
                    if defect == "read"
                        && operation == "SetRetention"
                        && matches!(&*target, ApplicationOperationDecisionReadTarget::Field {field, ..} if field == "PrincipalIdentityField") =>
                {
                    *target = ApplicationOperationDecisionReadTarget::Entity {
                        entity: "Principal".to_owned(),
                    };
                }
                ApplicationSchemaMember::Field { field, unit, .. }
                    if defect == "unit" && field == "AuditActorField" =>
                {
                    *unit = Some("days".to_owned())
                }
                ApplicationSchemaMember::OperationBody { operation, body }
                    if operation == "SetRetention" =>
                {
                    let creation = body.creation().unwrap();
                    let mut fields = creation.fields().to_vec();
                    for field in &mut fields {
                        if let ApplicationOperationBodyCreatedField::PrincipalIdentity {
                            binding,
                            mutation_binding,
                            principal_binding,
                            source,
                            ..
                        } = field
                        {
                            match defect {
                                "codec" => {
                                    *binding = WorthQueryPortableTypeIdentity::from_untrusted(
                                        "wrong.codec".to_owned(),
                                    )
                                }
                                "binding" => {
                                    *mutation_binding =
                                        "worth.query.body-court.source-binding.v1".to_owned()
                                }
                                "principal" => *principal_binding = "missing-principal".to_owned(),
                                "source" => {
                                    *source = ApplicationFieldBindingLocus::new(
                                        "Document",
                                        "DocumentFacts",
                                        "DocumentRetentionField",
                                    )
                                }
                                _ => {}
                            }
                        }
                    }
                    let creation = ApplicationOperationBodyCreation::from_untrusted(
                        creation.entity().to_owned(),
                        creation.key().to_owned(),
                        fields,
                    )
                    .with_untrusted_links(creation.links().to_vec());
                    *body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
                        body.scope_entity().unwrap().to_owned(),
                        body.scope_fields().to_vec(),
                        Some(creation),
                        body.requirements().to_vec(),
                    );
                }
                _ => {}
            }
        }
        parts.members.sort();
        let denial = validate_portable_application_schema_freshly(
            WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts),
        )
        .unwrap_err();
        if defect == "read" {
            assert_eq!(
                denial,
                ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency
            );
        }
        if defect == "unit" {
            assert_eq!(
                denial,
                ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency
            );
        }
    }
}
