use worth_query_declaration::facade::application_schema::{
    ApplicationEntityRef, ApplicationSchema, ApplicationSchemaDeclaration,
    ApplicationSchemaDeclarationBuilder,
};
use worth_query_installation::facade::{
    WorthQueryExpectedPortablePackageIdentity, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage, WorthQueryPortablePackageReconstruction,
    WorthQueryPortablePackageReconstructionLimits, WorthQueryPortablePackageRecordFamily as Family,
};
use worth_query_package_archive::facade::*;

const VERSION_FOUR_MINIMAL_SCHEMA_HEX: &str = "0004000800000001000000380000000d617263686976652e74657374730000000d4d696e696d616c536368656d610000000100000000000000010001000000044974656d";

struct Schema;
struct Entity;

impl ApplicationSchema for Schema {
    const OWNER: &'static str = "archive.tests";
    const NAME: &'static str = "MinimalSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        ApplicationSchemaDeclarationBuilder::for_schema()
            .entity(ApplicationEntityRef::<Self, Entity>::from_schema_identifier("Item"))
            .build()
    }
}

#[test]
fn application_schema_frame_is_deterministic_exact_and_freshly_readmitted() {
    let source = schema_package();
    let exported = source.export_typed_records().unwrap();
    let schema = schema_view(&exported);
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let first = encode_record_frame(schema, limits).unwrap();
    let second = encode_record_frame(schema, limits).unwrap();
    assert_eq!(first, second);

    let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(limits);
    let decoded = decoder.decode_frame(&first).unwrap();
    assert_eq!(decoded.canonical_index(), schema.canonical_index());
    assert_eq!(decoded.record(), schema.record());

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
    let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(limits);
    for view in exported.views() {
        let decoded = decoder
            .decode_frame(&encode_record_frame(view, limits).unwrap())
            .unwrap();
        let (index, record) = decoded.into_parts();
        reconstruction = reconstruction.push_record(index, record).unwrap();
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
fn version_four_minimal_application_schema_matches_the_frozen_vector() {
    let exported = schema_package().export_typed_records().unwrap();
    let schema = schema_view(&exported);
    let bytes = encode_record_frame(schema, WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    assert_eq!(encode_hex(&bytes), VERSION_FOUR_MINIMAL_SCHEMA_HEX);
    assert_eq!(u16::from_be_bytes(bytes[0..2].try_into().unwrap()), 4);
    assert_eq!(u16::from_be_bytes(bytes[2..4].try_into().unwrap()), 8);

    let frozen = decode_hex(VERSION_FOUR_MINIMAL_SCHEMA_HEX);
    let mut decoder =
        WorthQueryPackageArchiveRecordDecoder::new(WorthQueryPackageArchiveLimits::DEFAULT);
    let decoded = decoder.decode_frame(&frozen).unwrap();
    assert_eq!(decoded.canonical_index(), schema.canonical_index());
    assert_eq!(decoded.record(), schema.record());
}

#[test]
fn application_schema_semantic_tamper_cannot_mint_the_expected_query_identity() {
    let source = schema_package();
    let exported = source.export_typed_records().unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let manifest = decode_manifest_frame(
        &encode_manifest_frame(exported.manifest(), limits).unwrap(),
        limits,
    )
    .unwrap();
    let mut frames = exported
        .views()
        .map(|view| encode_record_frame(view, limits).unwrap())
        .collect::<Vec<_>>();
    let schema_index = exported
        .views()
        .position(|view| view.family() == Family::ApplicationSchema)
        .unwrap();
    let entity = b"Item";
    let entity_offset = frames[schema_index]
        .windows(entity.len())
        .position(|window| window == entity)
        .expect("schema frame carries the declared entity identifier");
    frames[schema_index][entity_offset] = b'X';

    let mut reconstruction = WorthQueryPortablePackageReconstruction::begin(
        manifest,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();
    let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(limits);
    for frame in frames {
        let decoded = decoder.decode_frame(&frame).unwrap();
        let (index, record) = decoded.into_parts();
        reconstruction = reconstruction.push_record(index, record).unwrap();
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

#[test]
fn application_schema_nested_work_is_symmetric_and_failed_attempts_do_not_commit() {
    let exported = schema_package().export_typed_records().unwrap();
    let schema = schema_view(&exported);
    let defaults = WorthQueryPackageArchiveLimits::DEFAULT;
    let bytes = encode_record_frame(schema, defaults).unwrap();
    let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(defaults);
    decoder.decode_frame(&bytes).unwrap();
    let nested_entries = decoder.work().nested_entries();
    assert_eq!(nested_entries, 1);

    let exact = defaults.with_maximum_nested_entries(nested_entries);
    assert_eq!(encode_record_frame(schema, exact).unwrap(), bytes);
    let mut exact_decoder = WorthQueryPackageArchiveRecordDecoder::new(exact);
    exact_decoder.decode_frame(&bytes).unwrap();
    assert_eq!(exact_decoder.work().nested_entries(), nested_entries);

    let narrow = defaults.with_maximum_nested_entries(nested_entries - 1);
    assert_eq!(
        encode_record_frame(schema, narrow).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::NestedEntryBudgetExceeded
    );
    let mut narrow_decoder = WorthQueryPackageArchiveRecordDecoder::new(narrow);
    assert_eq!(
        narrow_decoder.decode_frame(&bytes).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::NestedEntryBudgetExceeded
    );
    assert_eq!(
        narrow_decoder.work(),
        WorthQueryPackageArchiveDecodeWork::default()
    );
}

#[test]
fn application_schema_unknown_member_and_trailing_payload_fail_closed() {
    let exported = schema_package().export_typed_records().unwrap();
    let schema = schema_view(&exported);
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let bytes = encode_record_frame(schema, limits).unwrap();
    let mut unknown = bytes.clone();
    let member_tag_offset = 12 + 4 + Schema::OWNER.len() + 4 + Schema::NAME.len() + 4 + 4 + 4;
    unknown[member_tag_offset..member_tag_offset + 2].copy_from_slice(&u16::MAX.to_be_bytes());
    let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(limits);
    assert_eq!(
        decoder.decode_frame(&unknown).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::UnsupportedRecordVariant
    );
    assert_eq!(
        decoder.work(),
        WorthQueryPackageArchiveDecodeWork::default()
    );

    let mut trailing = bytes;
    let payload_length = u32::from_be_bytes(trailing[8..12].try_into().unwrap());
    trailing[8..12].copy_from_slice(&(payload_length + 1).to_be_bytes());
    trailing.push(0);
    assert_eq!(
        decoder.decode_frame(&trailing).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::TrailingBytes
    );
    assert_eq!(
        decoder.work(),
        WorthQueryPackageArchiveDecodeWork::default()
    );
}

fn schema_package() -> worth_query_installation::facade::WorthQueryValidatedPortableDomainPackage {
    WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "archive.tests",
        1,
        0,
    ))
    .application_schema(Schema::declaration().unwrap())
    .validate()
    .unwrap()
}

fn schema_view<'a>(
    exported: &'a worth_query_installation::facade::WorthQueryPortablePackageRecordSet,
) -> worth_query_installation::facade::WorthQueryPortablePackageRecordView<'a> {
    exported
        .views()
        .find(|view| view.family() == Family::ApplicationSchema)
        .unwrap()
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn decode_hex(encoded: &str) -> Vec<u8> {
    encoded
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let pair = std::str::from_utf8(pair).unwrap();
            u8::from_str_radix(pair, 16).unwrap()
        })
        .collect()
}
