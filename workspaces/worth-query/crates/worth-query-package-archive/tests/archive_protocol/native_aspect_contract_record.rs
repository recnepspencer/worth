use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
    StringApplicationValueBinding, U64ApplicationValueBinding,
};
use worth_query_installation::facade::{
    WorthQueryExpectedPortablePackageIdentity, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage, WorthQueryPortablePackageReconstruction,
    WorthQueryPortablePackageReconstructionLimits, WorthQueryPortablePackageRecordFamily as Family,
    WorthQueryValidatedPortableDomainPackage,
};
use worth_query_package_archive::facade::*;

const VERSION_ONE_NATIVE_ASPECT_HEX: &str = "0001000b00000002000000890000000c4e6174697665536368656d610000000c4e6174697665456e746974790000000750726f66696c650000000750726f66696c650000000091621100000000000000000300020000000200000003416765000a00010001000200000003546167001000010001000200000002000000034167650000000354616700010000000750726f66696c65";

struct NativeSchema;
worth_query_declaration::worth_query_entity!(NativeEntity for NativeSchema);
worth_query_declaration::worth_query_aspect!(
    Profile for NativeSchema, NativeEntity;
    identity = AspectIdentity(0x9162_1100),
    revision = AspectContractRevision(3),
);
worth_query_declaration::worth_query_field!(
    Age for NativeSchema, NativeEntity, Profile:
    u64 => U64ApplicationValueBinding, read_only, equality
);
worth_query_declaration::worth_query_field!(
    Tag for NativeSchema, NativeEntity, Profile:
    String => StringApplicationValueBinding, read_only, no_equality
);

impl ApplicationSchema for NativeSchema {
    const OWNER: &'static str = "archive.native";
    const NAME: &'static str = "NativeSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        ApplicationSchemaDeclarationBuilder::for_schema()
            .entity(NativeEntity::reference())
            .aspect(NativeEntity::reference(), Profile::reference())
            .field(NativeEntity::reference(), Age::reference())
            .field(NativeEntity::reference(), Tag::reference())
            .build()
    }
}

#[test]
fn native_aspect_frame_is_deterministic_exact_and_freshly_readmitted() {
    let source = native_package();
    let exported = source.export_typed_records().unwrap();
    let native = native_view(&exported);
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let first = encode_record_frame(native, limits).unwrap();
    assert_eq!(encode_record_frame(native, limits).unwrap(), first);
    assert_eq!(encode_hex(&first), VERSION_ONE_NATIVE_ASPECT_HEX);
    assert_eq!(u16::from_be_bytes(first[2..4].try_into().unwrap()), 11);
    let frozen = decode_hex(VERSION_ONE_NATIVE_ASPECT_HEX);
    let decoded = WorthQueryPackageArchiveRecordDecoder::new(limits)
        .decode_frame(&frozen)
        .unwrap();
    assert_eq!(decoded.canonical_index(), native.canonical_index());
    assert_eq!(decoded.record(), native.record());

    let mut reconstruction = WorthQueryPortablePackageReconstruction::begin(
        decode_manifest_frame(
            &encode_manifest_frame(exported.manifest(), limits).unwrap(),
            limits,
        )
        .unwrap(),
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
fn native_aspect_nested_work_is_symmetric_and_failed_attempts_are_atomic() {
    let exported = native_package().export_typed_records().unwrap();
    let native = native_view(&exported);
    let defaults = WorthQueryPackageArchiveLimits::DEFAULT;
    let bytes = encode_record_frame(native, defaults).unwrap();
    let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(defaults);
    decoder.decode_frame(&bytes).unwrap();
    let nested_entries = decoder.work().nested_entries();
    assert_eq!(nested_entries, 4);

    let exact = defaults.with_maximum_nested_entries(nested_entries);
    assert_eq!(encode_record_frame(native, exact).unwrap(), bytes);
    let mut exact_decoder = WorthQueryPackageArchiveRecordDecoder::new(exact);
    exact_decoder.decode_frame(&bytes).unwrap();
    assert_eq!(exact_decoder.work().nested_entries(), nested_entries);

    let narrow = defaults.with_maximum_nested_entries(nested_entries - 1);
    assert_eq!(
        encode_record_frame(native, narrow).unwrap_err().kind(),
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
fn noncanonical_retained_fields_and_unknown_binding_fail_closed() {
    let exported = native_package().export_typed_records().unwrap();
    let native = native_view(&exported);
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let bytes = encode_record_frame(native, limits).unwrap();
    let field_set = [
        0, 0, 0, 2, 0, 0, 0, 3, b'A', b'g', b'e', 0, 0, 0, 3, b'T', b'a', b'g',
    ];
    let offset = bytes
        .windows(field_set.len())
        .rposition(|window| window == field_set)
        .unwrap();

    let mut reordered = bytes.clone();
    reordered[offset + 8..offset + 11].copy_from_slice(b"Tag");
    reordered[offset + 15..offset + 18].copy_from_slice(b"Age");
    assert_decode_kind(
        &reordered,
        WorthQueryPackageArchiveDenialKind::NonCanonicalRecordSequence,
    );

    let mut duplicate = bytes.clone();
    duplicate[offset + 15..offset + 18].copy_from_slice(b"Age");
    assert_decode_kind(
        &duplicate,
        WorthQueryPackageArchiveDenialKind::NonCanonicalRecordSequence,
    );

    let mut unknown_binding = bytes;
    let binding_offset = unknown_binding.len() - (2 + 4 + "Profile".len());
    unknown_binding[binding_offset..binding_offset + 2].copy_from_slice(&9_u16.to_be_bytes());
    assert_decode_kind(
        &unknown_binding,
        WorthQueryPackageArchiveDenialKind::UnsupportedRecordVariant,
    );
}

#[test]
fn malformed_native_aspect_payloads_fail_without_committing_work() {
    let exported = native_package().export_typed_records().unwrap();
    let native = native_view(&exported);
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let bytes = encode_record_frame(native, limits).unwrap();
    for length in 0..bytes.len() {
        assert!(WorthQueryPackageArchiveRecordDecoder::new(limits)
            .decode_frame(&bytes[..length])
            .is_err());
    }

    let mut invalid_aspect = bytes.clone();
    let aspect_offset = 12 + 4 + NativeSchema::NAME.len() + 4 + "NativeEntity".len() + 4;
    invalid_aspect[aspect_offset] = b' ';
    assert_decode_kind(
        &invalid_aspect,
        WorthQueryPackageArchiveDenialKind::InvalidRecordShape,
    );

    let mut trailing = bytes;
    let payload_length = u32::from_be_bytes(trailing[8..12].try_into().unwrap()) + 1;
    trailing[8..12].copy_from_slice(&payload_length.to_be_bytes());
    trailing.push(0);
    assert_decode_kind(&trailing, WorthQueryPackageArchiveDenialKind::TrailingBytes);
}

#[test]
fn native_aspect_semantic_tamper_decodes_but_cannot_mint_package_identity() {
    let source = native_package();
    let exported = source.export_typed_records().unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let mut frames = exported
        .views()
        .map(|view| encode_record_frame(view, limits).unwrap())
        .collect::<Vec<_>>();
    let native_index = exported
        .views()
        .position(|view| view.family() == Family::NativeAspectContract)
        .unwrap();
    let schema_offset = frames[native_index]
        .windows(NativeSchema::NAME.len())
        .position(|window| window == NativeSchema::NAME.as_bytes())
        .unwrap();
    frames[native_index][schema_offset] = b'X';

    let mut reconstruction = WorthQueryPortablePackageReconstruction::begin(
        decode_manifest_frame(
            &encode_manifest_frame(exported.manifest(), limits).unwrap(),
            limits,
        )
        .unwrap(),
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();
    let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(limits);
    for frame in frames {
        let (index, record) = decoder.decode_frame(&frame).unwrap().into_parts();
        reconstruction = reconstruction.push_record(index, record).unwrap();
    }
    assert!(reconstruction
        .close()
        .unwrap()
        .materialize()
        .unwrap()
        .validate_freshly(
            WorthQueryExpectedPortablePackageIdentity::from_untrusted_identity(
                source.identity().clone(),
            ),
        )
        .is_err());
}

fn native_package() -> WorthQueryValidatedPortableDomainPackage {
    WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "archive.native",
        1,
        0,
    ))
    .application_schema(NativeSchema::declaration().unwrap())
    .validate()
    .unwrap()
}

fn native_view(
    exported: &worth_query_installation::facade::WorthQueryPortablePackageRecordSet,
) -> worth_query_installation::facade::WorthQueryPortablePackageRecordView<'_> {
    exported
        .views()
        .find(|view| view.family() == Family::NativeAspectContract)
        .unwrap()
}

fn assert_decode_kind(bytes: &[u8], expected: WorthQueryPackageArchiveDenialKind) {
    let mut decoder =
        WorthQueryPackageArchiveRecordDecoder::new(WorthQueryPackageArchiveLimits::DEFAULT);
    assert_eq!(decoder.decode_frame(bytes).unwrap_err().kind(), expected);
    assert_eq!(
        decoder.work(),
        WorthQueryPackageArchiveDecodeWork::default()
    );
}
