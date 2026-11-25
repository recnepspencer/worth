use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
pub(crate) use worth_query_testkit_certification_operation::facade::certification::native_operation_bodies::operation_body::conditional_requirement::portable::*;

#[test]
fn conditional_requirement_survives_checkpoint_and_fresh_schema_readmission() {
    let dir = super::super::unique_directory("cond-req");
    let old = super::super::open_directory::<25>(&dir);
    let projection = old.retain_invariant_projection_authority();
    assert_eq!(guidance(old.runtime(), &projection), Status::Met);
    let revision = *old.runtime().installed_program().revision();
    drop(projection);
    super::super::close_directory(old);
    let schema = program::schema::<25>();
    let portable = WorthQueryPortableApplicationSchemaRecord::project(schema.erased());
    assert_eq!(
        validate_portable_application_schema_freshly(portable).unwrap(),
        *schema.erased()
    );
    let restored = super::super::open_directory::<25>(&dir);
    assert_eq!(*restored.runtime().installed_program().revision(), revision);
    assert!(matches!(
        invoke(restored.runtime(), 507),
        Outcome::Committed { .. }
    ));
    assert_eq!(
        super::super::draft_query::read_audit_source(restored.runtime()),
        12
    );
    super::super::close_directory(restored);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn non_boolean_requirement_is_refused_at_authoring() {
    let operands = RequireDraftNote::reference::<DocumentRetentionSchema<25>>()
        .requirement_operands()
        .field("note", DocumentDraftNoteField::at("DocumentDraftNoteField"))
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
                    *binding = worth_query_host::facade::declaration::portable_identity::WorthQueryPortableTypeIdentity::from_untrusted(ARCHIVE_DECODE, "wrong.codec.v1".to_owned());
                }
                ApplicationOperationBodyRequirementOperand::from_untrusted(ARCHIVE_DECODE, operand.name().to_owned(), source)
            }).collect();
            let changed = ApplicationOperationBodyRequirementProgram::from_untrusted(
                ARCHIVE_DECODE,
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
                ARCHIVE_DECODE,
                body.scope_fields().to_vec(),
                body.creation().cloned(),
                vec![changed],
            );
        }
    }
    parts.members.sort();
    assert!(matches!(
        validate_portable_application_schema_freshly(
            WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(ARCHIVE_DECODE, parts)
        )
        .unwrap_err(),
        ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency { .. }
    ));
}
