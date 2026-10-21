//! Fresh native readmission must retain expression meaning and reject false bindings.
use super::*;
use worth_query_host::facade::{
    application_installation::{
        in_memory_program_from_checkpoint, WorthQueryCheckpointCapturePolicy,
    },
    declaration::application_schema::*,
};

#[test]
fn conditional_requirement_survives_checkpoint_and_fresh_schema_readmission() {
    let (old, projection) = super::super::install_with_projection::<25>();
    assert_eq!(guidance(&old, &projection), Status::Met);
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
    drop(projection);
    drop(old);
    let schema = program::schema::<25>();
    let portable = WorthQueryPortableApplicationSchemaRecord::project(schema.erased());
    assert_eq!(
        validate_portable_application_schema_freshly(portable).unwrap(),
        *schema.erased()
    );
    let restored = in_memory_program_from_checkpoint(
        program::validated::<25>(&schema),
        schema,
        (program::Installation::Native,),
        super::super::host_limits(),
        checkpoint,
    )
    .unwrap();
    assert_eq!(*restored.installed_program().revision(), revision);
    assert!(matches!(invoke(&restored, 507), Outcome::Committed { .. }));
    assert_eq!(super::super::draft_query::read_audit_source(&restored), 12);
}

#[test]
fn non_boolean_requirement_is_refused_at_authoring() {
    let operands = RequireDraftNote::reference::<DocumentRetentionSchema<25>>()
        .requirement_operands()
        .field("note", DocumentDraftNoteField::reference())
        .unwrap();
    let reader = worth_foundational::expression_api::expressions()
        .reading_within(ApplicationOperationBodyRequirementExpression::PROFILE);
    assert!(matches!(
        ApplicationOperationBodyRequirement::admit(
            "note",
            operands,
            None,
            reader.parse("note").unwrap(),
            "note"
        ),
        Err(ApplicationOperationBodyRequirementDenial::Expression(_))
    ));
}

#[test]
fn portable_requirement_cannot_claim_a_different_installed_field_codec() {
    let schema = program::schema::<25>();
    let mut parts =
        WorthQueryPortableApplicationSchemaRecord::project(schema.erased()).into_parts();
    for member in &mut parts.members {
        if let ApplicationSchemaMember::OperationBody { operation, body } = member {
            if operation != "RequireDraftNote" {
                continue;
            }
            let required = &body.requirements()[0];
            let operands = required.operands().iter().map(|operand| {
                let mut source = operand.source().clone();
                if let ApplicationOperationBodyRequirementSource::Field { binding, .. } = &mut source {
                    *binding = worth_query_host::facade::declaration::portable_identity::WorthQueryPortableTypeIdentity::from_untrusted("wrong.codec.v1".to_owned());
                }
                ApplicationOperationBodyRequirementOperand::from_untrusted(operand.name().to_owned(), source)
            }).collect();
            let changed = ApplicationOperationBodyRequirementProgram::from_untrusted(
                required.identity().to_owned(),
                operands,
                required
                    .applicability()
                    .map(|expression| expression.draft().to_vec()),
                required.satisfaction().draft().to_vec(),
                required.target_name().to_owned(),
            )
            .unwrap();
            *body = ApplicationOperationBodyProgram::from_untrusted_with_requirements(
                body.scope_fields().to_vec(),
                body.creation().cloned(),
                vec![changed],
            );
        }
    }
    parts.members.sort();
    assert!(matches!(
        validate_portable_application_schema_freshly(
            WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts)
        )
        .unwrap_err(),
        ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency { .. }
    ));
}
