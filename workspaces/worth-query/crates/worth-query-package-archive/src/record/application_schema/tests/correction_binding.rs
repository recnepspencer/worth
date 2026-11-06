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

fn member(content_format: Option<[u8; 32]>, frame: Option<String>) -> ApplicationSchemaMember {
    let binding = DeclaredCorrectionBinding::from_untrusted_parts(DeclaredCorrectionBindingParts {
        identity: identity("restore-amount.v1"),
        source: description("set-amount.v1", "SetAmount", "amount-input.v1"),
        inverse: description("restore-amount.v1", "RestoreAmount", "restore-input.v2"),
        source_handler: "set-handler.v1".to_owned(),
        inverse_handler: "restore-handler.v2".to_owned(),
        source_query: Some("amount-before.v1".to_owned()),
        inverse_query: Some("amount-current.v2".to_owned()),
        reapplication_source: Some(identity("set-amount.v1")),
        completed_external_reconciliation: Some(worth_query_declaration::facade::application_aftermath::DeclaredReconciliationProcedure::new("settled-source-local-compensation").unwrap()),
        authored_identity: Some(worth_query_declaration::facade::application_aftermath::DeclaredAuthoredCorrection::from_untrusted_parts(
            worth_query_declaration::facade::application_aftermath::DeclaredAuthoredCorrectionParts {
                family: identity("authored-account.v1"), source_role: "retired".to_owned(), inverse_role: "restored".to_owned(),
                source_posture: worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture::Retire,
                binding_unit: None,
                logical_key: Box::new(ApplicationSchemaMember::Field {
                    content_format: None,
                    entity: "Account".to_owned(), aspect: "State".to_owned(), field: "name".to_owned(),
                    presence: ApplicationFieldPresence::Required, scalar_family: worth_foundational::facade::ScalarAspectType::String,
                    value_type: "worth.string".to_owned(), canonical: None, unit: None, frame: None,
                    writable: false, equality_queryable: false, unique: false,
                }),
            },
        )),
        source_output_fields: vec![DeclaredCorrectionOutputField::from_untrusted_parts(DeclaredCorrectionOutputFieldParts {
            source_binding: identity("set-amount.v1"), role: "created-account".to_owned(),
            entity: "Account".to_owned(), aspect: "State".to_owned(), field: "id".to_owned(),
            presence: ApplicationFieldPresence::Required,
            scalar_family: if content_format.is_some() { worth_foundational::facade::ScalarAspectType::ContentRef } else { worth_foundational::facade::ScalarAspectType::UInt64 },
            value_binding: identity(if content_format.is_some() { "worth.content_ref" } else { "worth.u64" }), unit: Some("journal-number".to_owned()),
            binding_unit: Some("journal-number".to_owned()), frame, canonical: None, content_format,
        })],
        disclosure: DeclaredCorrectionDisclosure::from_untrusted_parts(DeclaredCorrectionDisclosureParts {
            binding: identity("amount-history.v1"), input: identity("history-input.v1"),
            query_name: "amount-history".to_owned(), query: identity("history-query.v1"),
            parameters: identity("history-parameters.v1"), result: identity("history-result.v1"),
            scope: identity("Account"), scope_entity: "Account".to_owned(),
            scope_aspect: "State".to_owned(), scope_field: "id".to_owned(),
            scope_value: identity("worth.u64"),
            scope_resolution: worth_query_declaration::facade::application_query::ApplicationQueryScopeResolutionMode::InputField, principal_binding: "HistoryPrincipal".to_owned(),
        }),
    });
    let contract = PortableApplicationAftermathContract::from_untrusted_fields(
        DeclaredCorrectionAuthority::RuntimeAlone,
        None,
        None,
    )
    .with_untrusted_correction_binding(binding);
    ApplicationSchemaMember::OperationAftermath {
        operation: "SetAmount".to_owned(),
        contract,
    }
}

#[test]
fn correction_archive_retains_both_bindings_handlers_queries_and_codecs() {
    let member = member(None, Some("ledger".to_owned()));
    let bytes = encode_member(&member);
    assert_eq!(decode_member(&bytes), member);
    assert_eq!(encode_member(&decode_member(&bytes)), bytes);

    // Independent wire expectation: present option, u32 text width, exact procedure.
    // An encoder/decoder pair that drops the explicit external opt-in cannot pass.
    let external_suffix =
        b"\0\x01\0\0\0\x21settled-source-local-compensation\0\x01\0\0\0\x0dset-amount.v1";
    assert!(bytes.ends_with(external_suffix));
    let mut invalid_option = bytes.clone();
    let option = bytes.len() - external_suffix.len();
    invalid_option[option + 1] = 2;
    assert_eq!(
        try_decode_member(&invalid_option).unwrap_err().kind(),
        crate::facade::WorthQueryPackageArchiveDenialKind::UnsupportedRecordVariant,
    );
    let mut invalid_reapplication = bytes.clone();
    let reapplication_option = bytes.len() - b"\0\x01\0\0\0\x0dset-amount.v1".len();
    invalid_reapplication[reapplication_option + 1] = 2;
    assert_eq!(
        try_decode_member(&invalid_reapplication)
            .unwrap_err()
            .kind(),
        crate::facade::WorthQueryPackageArchiveDenialKind::UnsupportedRecordVariant
    );

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
        inspection_disclosure: None,
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

#[test]
fn correction_content_output_archive_retains_exact_format_with_and_without_frame() {
    for (frame, tag) in [(None, 53u16), (Some("model".to_owned()), 54u16)] {
        let member = member(Some([0x5B; 32]), frame);
        let bytes = encode_member(&member);
        let decoded = decode_member(&bytes);
        assert_eq!(decoded, member);
        assert_eq!(encode_member(&decoded), bytes);
        // Independent local wire framing: the output role is followed by its binding-unit option,
        // a big-endian u16 field tag and the standard field record. The literal format ID must survive.
        let marker = b"created-account";
        let role_end = bytes
            .windows(marker.len())
            .position(|w| w == marker)
            .unwrap()
            + marker.len();
        let tag_start = role_end + 2 + 4 + b"journal-number".len();
        assert_eq!(&bytes[tag_start..tag_start + 2], &tag.to_be_bytes());
        assert!(bytes.windows(32).any(|w| w == [0x5B; 32]));
        let mut damaged = bytes.clone();
        damaged[tag_start..tag_start + 2].copy_from_slice(&55u16.to_be_bytes());
        assert_eq!(
            try_decode_member(&damaged).unwrap_err().kind(),
            crate::facade::WorthQueryPackageArchiveDenialKind::UnsupportedRecordVariant
        );
    }
}
