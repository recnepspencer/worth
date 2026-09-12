mod fixture;

use worth_query_installation::facade::{
    WorthQueryPortableApplicationOperationContractRecord,
    WorthQueryPortablePackageRecordFamily as Family,
};

use crate::binary_input::BinaryInput;
use crate::binary_output::BinaryOutput;
use crate::denial::WorthQueryPackageArchiveDenialKind as Kind;
use crate::limits::WorthQueryPackageArchiveLimits;
use crate::record::decode_budget::{RecordDecodeAttempt, WorthQueryPackageArchiveDecodeWork};
use crate::record::frame::{RecordFrameEncoding, WorthQueryPackageArchiveRecordDecoder};

#[test]
fn complete_operation_contract_round_trips_every_payload_family_with_exact_work() {
    let record = fixture::complete_record();
    let bytes = encode_untrusted(&record, WorthQueryPackageArchiveLimits::DEFAULT);
    let frozen_hex = include_str!("tests/application_operation_contract_v2.hex").trim();
    assert_eq!(encode_hex(&bytes), frozen_hex);
    assert_eq!(u16::from_be_bytes(bytes[2..4].try_into().unwrap()), 12);
    let mut decoder =
        WorthQueryPackageArchiveRecordDecoder::new(WorthQueryPackageArchiveLimits::DEFAULT);
    let decoded = decoder.decode_frame(&decode_hex(frozen_hex)).unwrap();
    assert_eq!(decoded.record(), &fixture::wrapped_complete_record());
    assert_eq!(decoder.work().record_frames(), 1);
    assert_eq!(decoder.work().logical_bytes(), (bytes.len() - 12) as u64);
    assert_eq!(decoder.work().nested_entries(), 15);
    let mut retired_frame = bytes;
    retired_frame[..2].copy_from_slice(&1_u16.to_be_bytes());
    let mut retired_decoder =
        WorthQueryPackageArchiveRecordDecoder::new(WorthQueryPackageArchiveLimits::DEFAULT);
    assert_eq!(
        retired_decoder
            .decode_frame(&retired_frame)
            .unwrap_err()
            .kind(),
        Kind::UnsupportedRecordVersion
    );
    assert_eq!(retired_decoder.work().record_frames(), 0);
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
fn absent_optional_contracts_round_trip_without_synthetic_defaults() {
    let mut parts = fixture::complete_parts();
    parts.external_effect = None;
    parts.reconciliation = None;
    let record = WorthQueryPortableApplicationOperationContractRecord::from_untrusted_parts(parts);
    let bytes = encode_untrusted(&record, WorthQueryPackageArchiveLimits::DEFAULT);
    let decoded =
        WorthQueryPackageArchiveRecordDecoder::new(WorthQueryPackageArchiveLimits::DEFAULT)
            .decode_frame(&bytes)
            .unwrap();
    assert_eq!(decoded.record().clone(), fixture::wrapped(record));
}

#[test]
fn nested_entry_budget_is_symmetric_and_failed_attempts_are_atomic() {
    let record = fixture::complete_record();
    let defaults = WorthQueryPackageArchiveLimits::DEFAULT;
    let bytes = encode_untrusted(&record, defaults);
    let exact = defaults.with_maximum_nested_entries(15);
    assert_eq!(encode_untrusted(&record, exact), bytes);
    let mut exact_decoder = WorthQueryPackageArchiveRecordDecoder::new(exact);
    exact_decoder.decode_frame(&bytes).unwrap();
    assert_eq!(exact_decoder.work().nested_entries(), 15);

    let narrow = defaults.with_maximum_nested_entries(14);
    assert_eq!(
        super::payload_byte_length(&record, narrow)
            .unwrap_err()
            .kind(),
        Kind::NestedEntryBudgetExceeded
    );
    let mut narrow_decoder = WorthQueryPackageArchiveRecordDecoder::new(narrow);
    assert_eq!(
        narrow_decoder.decode_frame(&bytes).unwrap_err().kind(),
        Kind::NestedEntryBudgetExceeded
    );
    assert_eq!(
        narrow_decoder.work(),
        WorthQueryPackageArchiveDecodeWork::default()
    );
}

#[test]
fn noncanonical_operation_contract_sequences_fail_without_normalization() {
    let mut graph_parts = fixture::complete_parts();
    graph_parts.graph_reads.swap(0, 1);
    assert_decode_kind(
        &encode_untrusted(
            &WorthQueryPortableApplicationOperationContractRecord::from_untrusted_parts(
                graph_parts,
            ),
            WorthQueryPackageArchiveLimits::DEFAULT,
        ),
        Kind::NonCanonicalRecordSequence,
    );

    let mut touch_parts = fixture::complete_parts();
    touch_parts.touches.swap(0, 1);
    assert_decode_kind(
        &encode_untrusted(
            &WorthQueryPortableApplicationOperationContractRecord::from_untrusted_parts(
                touch_parts,
            ),
            WorthQueryPackageArchiveLimits::DEFAULT,
        ),
        Kind::NonCanonicalRecordSequence,
    );

    let mut emission_parts = fixture::complete_parts();
    emission_parts.emissions[1] = emission_parts.emissions[0].clone();
    assert_decode_kind(
        &encode_untrusted(
            &WorthQueryPortableApplicationOperationContractRecord::from_untrusted_parts(
                emission_parts,
            ),
            WorthQueryPackageArchiveLimits::DEFAULT,
        ),
        Kind::NonCanonicalRecordSequence,
    );
}

#[test]
fn duplicate_operation_contract_loci_fail_without_normalization() {
    let mut graph_parts = fixture::complete_parts();
    graph_parts.graph_reads[1] = graph_parts.graph_reads[0].clone();
    assert_untrusted_parts_decode_kind(graph_parts, Kind::NonCanonicalRecordSequence);

    let mut touch_parts = fixture::complete_parts();
    touch_parts.touches[1] = touch_parts.touches[0].clone();
    assert_untrusted_parts_decode_kind(touch_parts, Kind::NonCanonicalRecordSequence);
}

#[test]
fn unknown_scope_tags_and_invalid_external_protocols_fail_closed() {
    assert_scope_tag_denied(true, 4);
    assert_scope_tag_denied(false, 6);

    let invalid_identity = effect_bytes("Invalid.protocol", 1);
    assert_eq!(
        super::external_effect::decode(&mut BinaryInput::new(&invalid_identity))
            .unwrap_err()
            .kind(),
        Kind::InvalidRecordShape
    );
    let zero_version = effect_bytes("archive.effect", 0);
    assert_eq!(
        super::external_effect::decode(&mut BinaryInput::new(&zero_version))
            .unwrap_err()
            .kind(),
        Kind::InvalidRecordShape
    );

    let invalid_correlation = effect_bytes_with_correlation("dispatch.rail", "archive.effect", 1);
    assert_eq!(
        super::external_effect::decode(&mut BinaryInput::new(&invalid_correlation))
            .unwrap_err()
            .kind(),
        Kind::InvalidRecordShape
    );
}

#[test]
fn truncation_and_trailing_payload_fail_without_committing_work() {
    let bytes = encode_untrusted(
        &fixture::complete_record(),
        WorthQueryPackageArchiveLimits::DEFAULT,
    );
    for length in 0..bytes.len() {
        assert!(WorthQueryPackageArchiveRecordDecoder::new(
            WorthQueryPackageArchiveLimits::DEFAULT
        )
        .decode_frame(&bytes[..length])
        .is_err());
    }
    let mut trailing = bytes;
    let payload_length = u32::from_be_bytes(trailing[8..12].try_into().unwrap()) + 1;
    trailing[8..12].copy_from_slice(&payload_length.to_be_bytes());
    trailing.push(0);
    assert_decode_kind(&trailing, Kind::TrailingBytes);
}

fn encode_untrusted(
    record: &WorthQueryPortableApplicationOperationContractRecord,
    limits: WorthQueryPackageArchiveLimits,
) -> Vec<u8> {
    let payload_bytes = super::payload_byte_length(record, limits).unwrap();
    let mut frame = RecordFrameEncoding::begin(
        Family::ApplicationOperationContract,
        0,
        payload_bytes,
        limits,
    )
    .unwrap();
    super::write_payload(record, frame.payload_output()).unwrap();
    frame.finish().unwrap()
}

fn assert_decode_kind(bytes: &[u8], expected: Kind) {
    let mut decoder =
        WorthQueryPackageArchiveRecordDecoder::new(WorthQueryPackageArchiveLimits::DEFAULT);
    assert_eq!(decoder.decode_frame(bytes).unwrap_err().kind(), expected);
    assert_eq!(
        decoder.work(),
        WorthQueryPackageArchiveDecodeWork::default()
    );
}

fn assert_untrusted_parts_decode_kind(
    parts: worth_query_installation::facade::WorthQueryPortableApplicationOperationContractParts,
    expected: Kind,
) {
    let record = WorthQueryPortableApplicationOperationContractRecord::from_untrusted_parts(parts);
    let bytes = encode_untrusted(&record, WorthQueryPackageArchiveLimits::DEFAULT);
    assert_decode_kind(&bytes, expected);
}

fn assert_scope_tag_denied(graph_read: bool, tag: u16) {
    let bytes = tag.to_be_bytes();
    let mut input = BinaryInput::new(&bytes);
    let mut attempt = RecordDecodeAttempt::begin(
        WorthQueryPackageArchiveDecodeWork::default(),
        2,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    let denial = if graph_read {
        super::graph_read::decode(&mut input, &mut attempt).unwrap_err()
    } else {
        super::touch::decode(&mut input, &mut attempt).unwrap_err()
    };
    assert_eq!(denial.kind(), Kind::UnsupportedRecordVariant);
}

fn effect_bytes(protocol_identity: &str, protocol_version: u32) -> Vec<u8> {
    effect_bytes_with_correlation("dispatch-rail", protocol_identity, protocol_version)
}

fn effect_bytes_with_correlation(
    correlation_family: &str,
    protocol_identity: &str,
    protocol_version: u32,
) -> Vec<u8> {
    let mut output = BinaryOutput::with_capacity(96);
    output.u16(1);
    output.text(correlation_family);
    output.text("payment");
    output.text("archive.effect.payload");
    output.text(protocol_identity);
    output.u32(protocol_version);
    output.u64(1_024);
    output.into_bytes()
}
