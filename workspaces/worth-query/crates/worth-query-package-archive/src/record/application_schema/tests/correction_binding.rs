use super::{decode_member, encode_member, try_decode_member};
use worth_query_declaration::facade::{
    application_aftermath::{
        DeclaredCorrectionAuthority, DeclaredCorrectionBinding, DeclaredCorrectionBindingParts,
        DeclaredCorrectionDisclosure, DeclaredCorrectionDisclosureParts,
        DeclaredCorrectionOutputField, DeclaredCorrectionOutputFieldParts,
        PortableApplicationAftermathContract,
    },
    application_operation::{
        ApplicationMutationDescription, ApplicationMutationDescriptionParts,
        ApplicationMutationScopeDescription, ApplicationMutationScopeResolutionMode,
        ApplicationWorkflowExecutionRequirements,
    },
    application_schema::{ApplicationFieldPresence, ApplicationSchemaMember},
    portable_identity::WorthQueryPortableTypeIdentity,
};

#[test]
fn correction_archive_retains_both_bindings_handlers_queries_and_codecs() {
    let binding = DeclaredCorrectionBinding::from_untrusted_parts(DeclaredCorrectionBindingParts {
        identity: identity("restore-amount.v1"),
        source: description("set-amount.v1", "SetAmount", "amount-input.v1"),
        inverse: description("restore-amount.v1", "RestoreAmount", "restore-input.v2"),
        source_handler: "set-handler.v1".to_owned(),
        inverse_handler: "restore-handler.v2".to_owned(),
        source_query: Some("amount-before.v1".to_owned()),
        inverse_query: Some("amount-current.v2".to_owned()),
        source_output_fields: vec![DeclaredCorrectionOutputField::from_untrusted_parts(DeclaredCorrectionOutputFieldParts {
            source_binding: identity("set-amount.v1"), role: "created-account".to_owned(),
            entity: "Account".to_owned(), aspect: "State".to_owned(), field: "id".to_owned(),
            presence: ApplicationFieldPresence::Required,
            scalar_family: worth_foundational::facade::ScalarAspectType::UInt64,
            value_binding: identity("worth.u64"), unit: Some("journal-number".to_owned()),
            binding_unit: Some("journal-number".to_owned()), frame: Some("ledger".to_owned()), canonical: None,
        })],
        disclosure: DeclaredCorrectionDisclosure::from_untrusted_parts(DeclaredCorrectionDisclosureParts {
            binding: identity("amount-history.v1"), input: identity("history-input.v1"),
            query_name: "amount-history".to_owned(), query: identity("history-query.v1"),
            parameters: identity("history-parameters.v1"), result: identity("history-result.v1"),
            scope: identity("Account"), scope_entity: "Account".to_owned(),
            scope_aspect: "State".to_owned(), scope_field: "id".to_owned(),
            scope_value: identity("worth.u64"),
            scope_resolution: worth_query_declaration::facade::application_query::ApplicationQueryScopeResolutionMode::InputField,
            scope_writable: false, principal_binding: "HistoryPrincipal".to_owned(),
        }),
    });
    let contract = PortableApplicationAftermathContract::from_untrusted_fields(
        DeclaredCorrectionAuthority::RuntimeAlone,
        None,
        None,
    )
    .with_untrusted_correction_binding(binding);
    let member = ApplicationSchemaMember::OperationAftermath {
        operation: "SetAmount".to_owned(),
        contract,
    };
    let bytes = encode_member(&member);
    assert_eq!(decode_member(&bytes), member);
    assert_eq!(encode_member(&decode_member(&bytes)), bytes);

    // Losing any suffix of the association cannot silently degrade to descriptive aftermath.
    for end in [bytes.len() - 1, bytes.len() - "amount-current.v2".len()] {
        assert_eq!(
            try_decode_member(&bytes[..end]).unwrap_err().kind(),
            crate::facade::WorthQueryPackageArchiveDenialKind::Truncated
        );
    }
}

fn description(binding: &str, operation: &str, input: &str) -> ApplicationMutationDescription {
    ApplicationMutationDescription::from_untrusted_parts(ApplicationMutationDescriptionParts {
        binding_identity: identity(binding),
        operation: operation.to_owned(),
        input_identity: identity(input),
        result_identity: identity("amount-result.v1"),
        denial_identity: identity("amount-denial.v1"),
        scope: ApplicationMutationScopeDescription {
            entity: "Account".to_owned(),
            aspect: "State".to_owned(),
            field: "id".to_owned(),
            resolution: ApplicationMutationScopeResolutionMode::InputField,
        },
        output_roles: Vec::new(),
        output_role_families: Vec::new(),
        workflow_execution_requirements: ApplicationWorkflowExecutionRequirements::occurrence_only(
        ),
    })
}

fn identity(value: &str) -> WorthQueryPortableTypeIdentity {
    WorthQueryPortableTypeIdentity::from_untrusted(value.to_owned())
}
