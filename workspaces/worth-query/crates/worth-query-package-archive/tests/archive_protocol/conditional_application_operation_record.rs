use worth_query_declaration::facade::application_schema::{
    ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationSchema,
    ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
};
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;
use worth_query_installation::facade::{
    WorthQueryExpectedPortablePackageIdentity,
    WorthQueryPortableApplicationConditionalOperationBinding,
    WorthQueryPortableApplicationConditionalOperationBindingParts,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
    WorthQueryPortablePackageReconstruction, WorthQueryPortablePackageReconstructionLimits,
    WorthQueryPortablePackageRecordFamily as Family, WorthQueryValidatedPortableDomainPackage,
};
use worth_query_package_archive::facade::*;

const VERSION_ONE_CONDITIONAL_BINDING_HEX: &str = "0001000900000003000000cc00000013617263686976652e636f6e646974696f6e616c00000011436f6e646974696f6e616c536368656d610000001b436f6e646974696f6e616c417263686976654f7065726174696f6e00000025776f7274682e71756572792e617263686976652e636f6e646974696f6e616c2d696e707574000000106c65646765722d62616c616e63653a310000004035616331333563613538336631343538643632353030326633316366383866633565633036346339343533356237623530626633356462386337613133373037";

struct Schema;
struct Operation;
struct Input;

worth_query_declaration::worth_query_portable_type!(
    Input => "worth.query.archive.conditional-input"
);
worth_query_declaration::worth_query_structured_value_binding!(
    InputBinding for Input {
        identity: "worth.query.archive.conditional-input"
    }
);

impl ApplicationOperationMarkerIdentity<Schema> for Operation {
    type InputBinding = InputBinding;
    const IDENTIFIER: &'static str = "ConditionalArchiveOperation";
}

impl ApplicationSchema for Schema {
    const OWNER: &'static str = "archive.conditional";
    const NAME: &'static str = "ConditionalSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        let operation = ApplicationOperationRef::<Self, Operation, Input>::from_declaration();
        ApplicationSchemaDeclarationBuilder::for_schema()
            .operation(
                operation
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .build()
    }
}

#[test]
fn conditional_application_operation_frame_is_deterministic_and_exact() {
    let source = conditional_package();
    let exported = source.export_typed_records().unwrap();
    let view = conditional_view(&exported);
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;

    let first = encode_record_frame(view, limits).unwrap();
    assert_eq!(encode_record_frame(view, limits).unwrap(), first);
    assert_eq!(encode_hex(&first), VERSION_ONE_CONDITIONAL_BINDING_HEX);
    assert_eq!(u16::from_be_bytes(first[2..4].try_into().unwrap()), 9);

    let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(limits);
    let decoded = decoder
        .decode_frame(&decode_hex(VERSION_ONE_CONDITIONAL_BINDING_HEX))
        .unwrap();
    assert_eq!(decoded.canonical_index(), view.canonical_index());
    assert_eq!(decoded.record(), view.record());
    assert_eq!(decoder.work().record_frames(), 1);
    assert_eq!(decoder.work().logical_bytes(), (first.len() - 12) as u64);
    assert_eq!(decoder.work().nested_entries(), 0);
}

fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;

    bytes.iter().fold(
        String::with_capacity(bytes.len() * 2),
        |mut encoded, byte| {
            write!(&mut encoded, "{byte:02x}").unwrap();
            encoded
        },
    )
}

fn decode_hex(encoded: &str) -> Vec<u8> {
    encoded
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let digits = std::str::from_utf8(pair).unwrap();
            u8::from_str_radix(digits, 16).unwrap()
        })
        .collect()
}

#[test]
fn decoded_conditional_binding_reenters_only_through_fresh_package_validation() {
    let source = conditional_package();
    let exported = source.export_typed_records().unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let conditional = conditional_view(&exported);
    let decoded = WorthQueryPackageArchiveRecordDecoder::new(limits)
        .decode_frame(&encode_record_frame(conditional, limits).unwrap())
        .unwrap();
    let decoded_record = decoded.into_record();
    let manifest = decode_manifest_frame(
        &encode_manifest_frame(exported.manifest(), limits).unwrap(),
        limits,
    )
    .unwrap();
    let mut reconstruction = WorthQueryPortablePackageReconstruction::begin(
        manifest,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();
    for view in exported.views() {
        let record = if view.family() == Family::ConditionalApplicationOperation {
            decoded_record.clone()
        } else {
            view.record().clone()
        };
        reconstruction = reconstruction
            .push_record(view.canonical_index(), record)
            .unwrap();
    }
    let reconstructed = reconstruction
        .close()
        .unwrap()
        .materialize()
        .unwrap()
        .validate_freshly(
            WorthQueryExpectedPortablePackageIdentity::from_untrusted_identity(
                source.identity().clone(),
            ),
        )
        .unwrap();
    assert_eq!(reconstructed.identity(), source.identity());
}

#[test]
fn conditional_binding_byte_limit_is_symmetric_and_failures_do_not_commit() {
    let exported = conditional_package().export_typed_records().unwrap();
    let view = conditional_view(&exported);
    let defaults = WorthQueryPackageArchiveLimits::DEFAULT;
    let bytes = encode_record_frame(view, defaults).unwrap();
    let payload_bytes = (bytes.len() - 12) as u64;
    let exact = WorthQueryPackageArchiveLimits::new(
        defaults.maximum_manifest_frame_bytes(),
        defaults.maximum_records(),
        payload_bytes,
        defaults.maximum_canonical_work_bytes(),
    );
    assert_eq!(encode_record_frame(view, exact).unwrap(), bytes);
    assert!(WorthQueryPackageArchiveRecordDecoder::new(exact)
        .decode_frame(&bytes)
        .is_ok());

    let narrow = WorthQueryPackageArchiveLimits::new(
        defaults.maximum_manifest_frame_bytes(),
        defaults.maximum_records(),
        payload_bytes - 1,
        defaults.maximum_canonical_work_bytes(),
    );
    assert_eq!(
        encode_record_frame(view, narrow).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::RecordFrameByteBudgetExceeded
    );
    let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(narrow);
    assert_eq!(
        decoder.decode_frame(&bytes).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::RecordFrameByteBudgetExceeded
    );
    assert_eq!(
        decoder.work(),
        WorthQueryPackageArchiveDecodeWork::default()
    );
}

#[test]
fn malformed_conditional_binding_payloads_fail_closed_without_committing_work() {
    let exported = conditional_package().export_typed_records().unwrap();
    let view = conditional_view(&exported);
    let defaults = WorthQueryPackageArchiveLimits::DEFAULT;
    let bytes = encode_record_frame(view, defaults).unwrap();
    for length in 0..bytes.len() {
        assert!(WorthQueryPackageArchiveRecordDecoder::new(defaults)
            .decode_frame(&bytes[..length])
            .is_err());
    }
    let mut invalid_utf8 = bytes.clone();
    let owner_offset = 12 + 4;
    invalid_utf8[owner_offset] = 0xff;
    assert_eq!(
        WorthQueryPackageArchiveRecordDecoder::new(defaults)
            .decode_frame(&invalid_utf8)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::InvalidUtf8
    );
    let mut trailing = bytes;
    let payload_length = u32::from_be_bytes(trailing[8..12].try_into().unwrap()) + 1;
    trailing[8..12].copy_from_slice(&payload_length.to_be_bytes());
    trailing.push(0);
    let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(defaults);
    assert_eq!(
        decoder.decode_frame(&trailing).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::TrailingBytes
    );
    assert_eq!(
        decoder.work(),
        WorthQueryPackageArchiveDecodeWork::default()
    );
}

#[test]
fn semantic_tamper_decodes_but_cannot_mint_the_expected_package_identity() {
    let source = conditional_package();
    let exported = source.export_typed_records().unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let view = conditional_view(&exported);
    let mut bytes = encode_record_frame(view, limits).unwrap();
    let operation = Operation::IDENTIFIER.as_bytes();
    let offset = bytes
        .windows(operation.len())
        .position(|window| window == operation)
        .unwrap();
    bytes[offset] = b'X';
    let tampered = WorthQueryPackageArchiveRecordDecoder::new(limits)
        .decode_frame(&bytes)
        .unwrap()
        .into_record();
    let manifest = decode_manifest_frame(
        &encode_manifest_frame(exported.manifest(), limits).unwrap(),
        limits,
    )
    .unwrap();
    let mut reconstruction = WorthQueryPortablePackageReconstruction::begin(
        manifest,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();
    for view in exported.views() {
        let record = if view.family() == Family::ConditionalApplicationOperation {
            tampered.clone()
        } else {
            view.record().clone()
        };
        reconstruction = reconstruction
            .push_record(view.canonical_index(), record)
            .unwrap();
    }
    let candidate = reconstruction.close().unwrap().materialize().unwrap();
    assert!(candidate
        .validate_freshly(
            WorthQueryExpectedPortablePackageIdentity::from_untrusted_identity(
                source.identity().clone(),
            ),
        )
        .is_err());
}

pub(super) fn conditional_package() -> WorthQueryValidatedPortableDomainPackage {
    let operation = super::domain_operation_record::fixture::operation();
    let binding = WorthQueryPortableApplicationConditionalOperationBinding::from_untrusted_parts(
        WorthQueryPortableApplicationConditionalOperationBindingParts {
            schema_owner: Schema::OWNER.to_owned(),
            schema_name: Schema::NAME.to_owned(),
            application_operation: Operation::IDENTIFIER.to_owned(),
            input_type: WorthQueryPortableTypeIdentity::from_untrusted(
                "worth.query.archive.conditional-input".to_owned(),
            ),
            domain_operation_slot: operation.identity().slot(),
            domain_operation_canonical_identity: operation.canonical_identity().to_owned(),
        },
    );
    WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "archive.conditional",
        1,
        0,
    ))
    .domain_operation(operation)
    .application_schema(Schema::declaration().unwrap())
    .conditional_application_operation_erased(binding)
    .validate()
    .unwrap()
}

fn conditional_view(
    exported: &worth_query_installation::facade::WorthQueryPortablePackageRecordSet,
) -> worth_query_installation::facade::WorthQueryPortablePackageRecordView<'_> {
    exported
        .views()
        .find(|view| view.family() == Family::ConditionalApplicationOperation)
        .unwrap()
}
