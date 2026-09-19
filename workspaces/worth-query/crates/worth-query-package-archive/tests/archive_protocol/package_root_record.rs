use worth_query_installation::facade::{
    WorthQueryExpectedPortablePackageIdentity, WorthQueryPortableDefinition,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
    WorthQueryPortablePackageReconstruction, WorthQueryPortablePackageReconstructionDenial,
    WorthQueryPortablePackageReconstructionLimits, WorthQueryPortablePackageRecordFamily as Family,
    WorthQueryValidatedPortableDomainPackage,
};
use worth_query_package_archive::facade::*;

const VERSION_FOUR_PACKAGE_ROOT_FRAMES_HEX: &str = "0004000100000000000000190000000d61636d652e776f726b666c6f770000000300000007|00040002000000010000000e0000000a67726170682e72656164|00040003000000020000001400000010776f726b666c6f772e72756e74696d65|00040004000000030000000b0000000764757261626c65|00040005000000040000002600010000000862616c616e636564000000146465626974732d657175616c2d63726564697473|00040005000000050000002800020000000d6163636f756e742d62792d69640000001165786163742d656e746974792d72656164|00040005000000060000002e00030000000e7061796d656e742d6e6f746963650000001665787465726e616c2d6566666563742d66616d696c79|0004000a0000000700000009000000056175646974";

fn root_package() -> WorthQueryValidatedPortableDomainPackage {
    WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "acme.workflow",
        3,
        7,
    ))
    .requires_capability("graph.read")
    .requires_configuration("workflow.runtime")
    .requires_operating_posture("durable")
    .definition(WorthQueryPortableDefinition::invariant(
        "balanced",
        "debits-equal-credits",
    ))
    .definition(WorthQueryPortableDefinition::graph_read_operation(
        "account-by-id",
        "exact-entity-read",
    ))
    .definition(WorthQueryPortableDefinition::declaration_family(
        "payment-notice",
        "external-effect-family",
    ))
    .permits_contribution("audit")
    .validate()
    .unwrap()
}

#[test]
fn package_root_frames_are_deterministic_and_reenter_fresh_validation() {
    let source = root_package();
    let exported = source.export_typed_records().unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let manifest_bytes = encode_manifest_frame(exported.manifest(), limits).unwrap();
    let manifest = decode_manifest_frame(&manifest_bytes, limits).unwrap();
    let mut reconstruction = WorthQueryPortablePackageReconstruction::begin(
        manifest,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();

    for view in exported.views() {
        let first = encode_package_root_record_frame(view, limits).unwrap();
        let second = encode_package_root_record_frame(view, limits).unwrap();
        assert_eq!(first, second);
        let decoded = decode_package_root_record_frame(&first, limits).unwrap();
        assert_eq!(decoded.canonical_index(), view.canonical_index());
        assert_eq!(decoded.family(), view.family());
        assert_eq!(decoded.record(), view.record());
        let (canonical_index, record) = decoded.into_parts();
        reconstruction = reconstruction.push_record(canonical_index, record).unwrap();
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
fn version_four_package_root_frames_match_and_decode_the_frozen_vectors() {
    let source = root_package();
    let exported = source.export_typed_records().unwrap();
    let encoded = exported
        .views()
        .map(|view| {
            encode_hex(
                &encode_package_root_record_frame(view, WorthQueryPackageArchiveLimits::DEFAULT)
                    .unwrap(),
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    assert_eq!(encoded, VERSION_FOUR_PACKAGE_ROOT_FRAMES_HEX);

    let independent_frames = VERSION_FOUR_PACKAGE_ROOT_FRAMES_HEX
        .split('|')
        .map(decode_hex)
        .collect::<Vec<_>>();
    assert_eq!(independent_frames.len(), exported.records().len());
    for (expected, bytes) in exported.views().zip(independent_frames) {
        let decoded =
            decode_package_root_record_frame(&bytes, WorthQueryPackageArchiveLimits::DEFAULT)
                .unwrap();
        assert_eq!(decoded.canonical_index(), expected.canonical_index());
        assert_eq!(decoded.family(), expected.family());
        assert_eq!(decoded.record(), expected.record());
    }
}

#[test]
fn malformed_record_headers_and_payloads_fail_closed() {
    let source = root_package();
    let exported = source.export_typed_records().unwrap();
    let capability = exported
        .views()
        .find(|view| view.family() == Family::CapabilityRequirement)
        .unwrap();
    let bytes =
        encode_package_root_record_frame(capability, WorthQueryPackageArchiveLimits::DEFAULT)
            .unwrap();
    for length in 0..bytes.len() {
        assert!(decode_package_root_record_frame(
            &bytes[..length],
            WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .is_err());
    }

    assert_mutation_denial(&bytes, 1, 0, Kind::UnsupportedRecordVersion);
    assert_mutation_denial(&bytes, 1, 2, Kind::UnsupportedRecordVersion);
    assert_mutation_denial(&bytes, 3, 0xff, Kind::UnsupportedRecordFamily);
    assert_mutation_denial(&bytes, 3, 6, Kind::Truncated);
    assert_mutation_denial(&bytes, 11, bytes[11] - 1, Kind::InvalidRecordLength);
    assert_mutation_denial(&bytes, 11, bytes[11] + 1, Kind::InvalidRecordLength);
    assert_mutation_denial(&bytes, 15, bytes[15] + 1, Kind::Truncated);
    assert_mutation_denial(&bytes, 15, bytes[15] - 1, Kind::TrailingBytes);

    let mut invalid_utf8 = bytes.clone();
    invalid_utf8[16] = 0xff;
    assert_eq!(
        decode_package_root_record_frame(&invalid_utf8, WorthQueryPackageArchiveLimits::DEFAULT,)
            .unwrap_err()
            .kind(),
        Kind::InvalidUtf8
    );

    let mut trailing_payload = bytes;
    trailing_payload[11] += 1;
    trailing_payload.push(0);
    assert_eq!(
        decode_package_root_record_frame(
            &trailing_payload,
            WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap_err()
        .kind(),
        Kind::TrailingBytes
    );
}

#[test]
fn unsupported_definition_kind_fails_before_phase_three() {
    let source = root_package();
    let exported = source.export_typed_records().unwrap();
    let definition = exported
        .views()
        .find(|view| view.family() == Family::Definition)
        .unwrap();
    let mut bytes =
        encode_package_root_record_frame(definition, WorthQueryPackageArchiveLimits::DEFAULT)
            .unwrap();
    for unsupported_tag in [4, 0xff] {
        bytes[13] = unsupported_tag;
        assert_eq!(
            decode_package_root_record_frame(&bytes, WorthQueryPackageArchiveLimits::DEFAULT)
                .unwrap_err()
                .kind(),
            Kind::UnsupportedDefinitionKind
        );
    }
}

#[test]
fn narrow_record_budget_rejects_before_payload_allocation() {
    let source = root_package();
    let exported = source.export_typed_records().unwrap();
    let capability = exported
        .views()
        .find(|view| view.family() == Family::CapabilityRequirement)
        .unwrap();
    let defaults = WorthQueryPackageArchiveLimits::DEFAULT;
    let bytes = encode_package_root_record_frame(capability, defaults).unwrap();
    let payload_length = u64::try_from(bytes.len() - 12).unwrap();
    let exact = WorthQueryPackageArchiveLimits::new(
        defaults.maximum_manifest_frame_bytes(),
        defaults.maximum_records(),
        payload_length,
        defaults.maximum_canonical_work_bytes(),
    );
    assert_eq!(
        encode_package_root_record_frame(capability, exact).unwrap(),
        bytes
    );
    assert!(decode_package_root_record_frame(&bytes, exact).is_ok());
    let narrow = WorthQueryPackageArchiveLimits::new(
        defaults.maximum_manifest_frame_bytes(),
        defaults.maximum_records(),
        payload_length - 1,
        defaults.maximum_canonical_work_bytes(),
    );
    assert_eq!(
        encode_package_root_record_frame(capability, narrow)
            .unwrap_err()
            .kind(),
        Kind::RecordFrameByteBudgetExceeded
    );
    assert_eq!(
        decode_package_root_record_frame(&bytes, narrow)
            .unwrap_err()
            .kind(),
        Kind::RecordFrameByteBudgetExceeded
    );
}

#[test]
fn canonical_index_ceiling_rejects_before_payload_decode() {
    let source = root_package();
    let exported = source.export_typed_records().unwrap();
    let capability = exported
        .views()
        .find(|view| view.family() == Family::CapabilityRequirement)
        .unwrap();
    let defaults = WorthQueryPackageArchiveLimits::DEFAULT;
    let bytes = encode_package_root_record_frame(capability, defaults).unwrap();
    let exact_index = WorthQueryPackageArchiveLimits::new(
        defaults.maximum_manifest_frame_bytes(),
        capability.canonical_index() + 1,
        defaults.maximum_logical_bytes(),
        defaults.maximum_canonical_work_bytes(),
    );
    assert!(encode_package_root_record_frame(capability, exact_index).is_ok());
    assert!(decode_package_root_record_frame(&bytes, exact_index).is_ok());
    let no_records = WorthQueryPackageArchiveLimits::new(
        defaults.maximum_manifest_frame_bytes(),
        0,
        defaults.maximum_logical_bytes(),
        defaults.maximum_canonical_work_bytes(),
    );
    assert_eq!(
        encode_package_root_record_frame(capability, no_records)
            .unwrap_err()
            .kind(),
        Kind::RecordIndexBudgetExceeded
    );
    assert_eq!(
        decode_package_root_record_frame(&bytes, no_records)
            .unwrap_err()
            .kind(),
        Kind::RecordIndexBudgetExceeded
    );
}

#[test]
fn phase_three_rejects_reordered_record_frames() {
    let source = root_package();
    let exported = source.export_typed_records().unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let frames = exported
        .views()
        .map(|view| encode_package_root_record_frame(view, limits).unwrap())
        .collect::<Vec<_>>();

    let manifest = decode_manifest_frame(
        &encode_manifest_frame(exported.manifest(), limits).unwrap(),
        limits,
    )
    .unwrap();
    let reconstruction = WorthQueryPortablePackageReconstruction::begin(
        manifest,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();
    let second = decode_package_root_record_frame(&frames[1], limits).unwrap();
    let (index, record) = second.into_parts();
    assert!(matches!(
        reconstruction.push_record(index, record),
        Err(
            WorthQueryPortablePackageReconstructionDenial::RecordIndexMismatch {
                expected: 0,
                observed: 1,
            }
        )
    ));
}

#[test]
fn semantic_tamper_remains_untrusted_until_fresh_identity_validation() {
    let source = root_package();
    let exported = source.export_typed_records().unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let manifest = decode_manifest_frame(
        &encode_manifest_frame(exported.manifest(), limits).unwrap(),
        limits,
    )
    .unwrap();
    let mut tampered = exported
        .views()
        .map(|view| encode_package_root_record_frame(view, limits).unwrap())
        .collect::<Vec<_>>();
    let capability_index = exported
        .views()
        .position(|view| view.family() == Family::CapabilityRequirement)
        .unwrap();
    tampered[capability_index][16] = b'x';
    let mut reconstruction = WorthQueryPortablePackageReconstruction::begin(
        manifest,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();
    for bytes in tampered {
        let decoded = decode_package_root_record_frame(&bytes, limits).unwrap();
        let (index, record) = decoded.into_parts();
        reconstruction = reconstruction.push_record(index, record).unwrap();
    }
    assert!(matches!(
        reconstruction
            .close()
            .unwrap()
            .materialize()
            .unwrap()
            .validate_freshly(
                WorthQueryExpectedPortablePackageIdentity::from_untrusted_identity(
                    source.identity().clone(),
                ),
            ),
        Err(WorthQueryPortablePackageReconstructionDenial::ManifestPackageIdentityMismatch { .. })
    ));
}

type Kind = WorthQueryPackageArchiveDenialKind;

fn assert_mutation_denial(bytes: &[u8], offset: usize, value: u8, expected: Kind) {
    let mut mutated = bytes.to_vec();
    mutated[offset] = value;
    assert_eq!(
        decode_package_root_record_frame(&mutated, WorthQueryPackageArchiveLimits::DEFAULT)
            .unwrap_err()
            .kind(),
        expected
    );
}

fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;

    bytes.iter().fold(
        String::with_capacity(bytes.len() * 2),
        |mut encoded, byte| {
            write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
            encoded
        },
    )
}

fn decode_hex(encoded: &str) -> Vec<u8> {
    assert_eq!(encoded.len() % 2, 0, "golden hex must contain whole bytes");
    encoded
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let digits = std::str::from_utf8(pair).expect("golden hex is ASCII");
            u8::from_str_radix(digits, 16).expect("golden hex contains valid digits")
        })
        .collect()
}

fn encode_package_root_record_frame(
    view: worth_query_installation::facade::WorthQueryPortablePackageRecordView<'_>,
    limits: WorthQueryPackageArchiveLimits,
) -> Result<Vec<u8>, WorthQueryPackageArchiveDenial> {
    encode_record_frame(view, limits)
}

fn decode_package_root_record_frame(
    bytes: &[u8],
    limits: WorthQueryPackageArchiveLimits,
) -> Result<WorthQueryUntrustedPortablePackageRecordFrame, WorthQueryPackageArchiveDenial> {
    WorthQueryPackageArchiveRecordDecoder::new(limits).decode_frame(bytes)
}
