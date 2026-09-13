mod fixture;
mod invariant;
mod mutation_description;

use worth_query_declaration::facade::application_schema::{
    ApplicationRelationIntegrity, ApplicationSchemaMember,
};
use worth_query_installation::facade::WorthQueryPortablePackageRecord;

use crate::binary_output::BinaryOutput;
use crate::facade::WorthQueryPackageArchiveDenialKind;
use crate::facade::{WorthQueryPackageArchiveDecodeWork, WorthQueryPackageArchiveRecordDecoder};
use crate::limits::WorthQueryPackageArchiveLimits;

use super::super::decode_budget::RecordDecodeAttempt;
use super::{decode_payload, payload_byte_length, write_payload};

#[test]
fn every_application_schema_member_family_round_trips_exact_owned_meaning() {
    let record = fixture::complete_untrusted_schema_record();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let payload_bytes = payload_byte_length(&record, limits).unwrap();
    let capacity = usize::try_from(payload_bytes).unwrap();
    let mut output = BinaryOutput::with_capacity(capacity);
    write_payload(&record, &mut output, limits).unwrap();
    let bytes = output.into_bytes();
    assert_eq!(bytes.len(), capacity);

    let mut input = crate::binary_input::BinaryInput::new(&bytes);
    let mut attempt = RecordDecodeAttempt::begin(
        Default::default(),
        u64::try_from(bytes.len()).unwrap(),
        limits,
    )
    .unwrap();
    let decoded = decode_payload(&mut input, &mut attempt).unwrap();
    assert!(input.is_finished());
    let WorthQueryPortablePackageRecord::ApplicationSchema(decoded) = decoded else {
        panic!("tag-8 payload must decode as an application schema")
    };
    assert_eq!(decoded, record);
    assert_eq!(decoded.members().len(), fixture::EXPECTED_MEMBER_COUNT);
    assert_eq!(decoded.contributions().len(), 2);
    assert_eq!(
        decoded.contributions()[0].member_ordinals(),
        &(0..13).collect::<Vec<_>>()
    );
    assert!(attempt.finish().nested_entries() > decoded.members().len() as u64);
}

#[test]
fn noncanonical_nested_capability_values_are_encoded_and_decoded_without_normalization() {
    let record = fixture::complete_untrusted_schema_record();
    let capability = record
        .members()
        .iter()
        .find_map(|member| {
            let worth_query_declaration::facade::application_schema::ApplicationSchemaMember::ApplicationCapability { contract } = member else { return None };
            Some(contract)
        })
        .unwrap();
    let accepted = capability
        .composition()
        .decision()
        .allow()
        .graph()
        .requirements()[0]
        .clauses()[0]
        .guard()
        .requirements()[0]
        .values();
    assert!(accepted[0] > accepted[1]);

    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let mut output = BinaryOutput::with_capacity(4096);
    write_payload(&record, &mut output, limits).unwrap();
    let bytes = output.into_bytes();
    let mut input = crate::binary_input::BinaryInput::new(&bytes);
    let mut attempt = RecordDecodeAttempt::begin(
        Default::default(),
        u64::try_from(bytes.len()).unwrap(),
        limits,
    )
    .unwrap();
    let WorthQueryPortablePackageRecord::ApplicationSchema(decoded) =
        decode_payload(&mut input, &mut attempt).unwrap()
    else {
        unreachable!()
    };
    assert_eq!(decoded, record);
}

#[test]
fn version_one_application_schema_member_tags_are_frozen() {
    let record = fixture::complete_untrusted_schema_record();
    let tags = record.members()[..24]
        .iter()
        .map(super::member::member_tag)
        .collect::<Vec<_>>();
    let mut expected = (1_u16..=24).collect::<Vec<_>>();
    expected[2] = 25;
    assert_eq!(tags, expected);

    let mut legacy_field = record.members()[2].clone();
    let worth_query_declaration::facade::application_schema::ApplicationSchemaMember::Field {
        frame,
        ..
    } = &mut legacy_field
    else {
        unreachable!()
    };
    *frame = None;
    assert_eq!(super::member::member_tag(&legacy_field), 3);
    assert_eq!(super::member::member_tag(&record.members()[3]), 4);
}

#[test]
fn tag_four_relation_bytes_retain_self_edge_meaning_and_reencode_exactly() {
    let member = ApplicationSchemaMember::Relation {
        relation: "Peer".to_owned(),
        from: "Entity".to_owned(),
        to: "Entity".to_owned(),
        integrity: ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
    };
    let bytes = encode_member(&member);

    assert_eq!(
        bytes,
        vec![
            0, 4, 0, 0, 0, 4, b'P', b'e', b'e', b'r', 0, 0, 0, 6, b'E', b'n', b't', b'i', b't',
            b'y', 0, 0, 0, 6, b'E', b'n', b't', b'i', b't', b'y',
        ]
    );
    let decoded = decode_member(&bytes);
    let ApplicationSchemaMember::Relation { integrity, .. } = &decoded else {
        panic!("tag 4 must decode as a relation")
    };
    assert!(integrity.endpoints.self_edges_allowed);
    assert_eq!(encode_member(&decoded), bytes);
}

#[test]
fn no_self_edges_uses_the_revised_relation_record() {
    let member = ApplicationSchemaMember::Relation {
        relation: "Peer".to_owned(),
        from: "Entity".to_owned(),
        to: "Entity".to_owned(),
        integrity:
            ApplicationRelationIntegrity::same_context_no_self_edges_unbounded_retain_dangling(),
    };
    let bytes = encode_member(&member);

    assert_eq!(&bytes[..2], &26_u16.to_be_bytes());
    let decoded = decode_member(&bytes);
    let ApplicationSchemaMember::Relation { integrity, .. } = &decoded else {
        panic!("tag 26 must decode as a relation")
    };
    assert!(!integrity.endpoints.self_edges_allowed);
    assert_eq!(decoded, member);
}

#[test]
fn complete_schema_nested_work_is_symmetric_and_failed_decode_does_not_commit() {
    let record = fixture::complete_untrusted_schema_record();
    let ordinary_limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let payload = encode_payload(&record, ordinary_limits);
    let mut input = crate::binary_input::BinaryInput::new(&payload);
    let mut attempt = RecordDecodeAttempt::begin(
        Default::default(),
        u64::try_from(payload.len()).unwrap(),
        ordinary_limits,
    )
    .unwrap();
    decode_payload(&mut input, &mut attempt).unwrap();
    let nested_entries = attempt.finish().nested_entries();
    assert!(nested_entries > record.members().len() as u64);

    let exact_limits = ordinary_limits.with_maximum_nested_entries(nested_entries);
    assert_eq!(encode_payload(&record, exact_limits), payload);
    let frame = frame_payload(&payload);
    let mut exact_decoder = WorthQueryPackageArchiveRecordDecoder::new(exact_limits);
    exact_decoder.decode_frame(&frame).unwrap();
    assert_eq!(exact_decoder.work().nested_entries(), nested_entries);

    let narrow_limits = ordinary_limits.with_maximum_nested_entries(nested_entries - 1);
    assert_eq!(
        payload_byte_length(&record, narrow_limits)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::NestedEntryBudgetExceeded
    );
    let mut narrow_decoder = WorthQueryPackageArchiveRecordDecoder::new(narrow_limits);
    assert_eq!(
        narrow_decoder.decode_frame(&frame).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::NestedEntryBudgetExceeded
    );
    assert_eq!(
        narrow_decoder.work(),
        WorthQueryPackageArchiveDecodeWork::default()
    );
}

#[test]
fn result_shape_nesting_depth_is_bounded_on_encode_and_decode() {
    let record = fixture::complete_untrusted_schema_record();
    let narrow_limits = WorthQueryPackageArchiveLimits::DEFAULT.with_maximum_nesting_depth(1);
    assert_eq!(
        payload_byte_length(&record, narrow_limits)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::NestingDepthBudgetExceeded
    );

    let ordinary_limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let payload_bytes = payload_byte_length(&record, ordinary_limits).unwrap();
    let mut output = BinaryOutput::with_capacity(usize::try_from(payload_bytes).unwrap());
    write_payload(&record, &mut output, ordinary_limits).unwrap();
    let bytes = output.into_bytes();
    let mut input = crate::binary_input::BinaryInput::new(&bytes);
    let mut attempt = RecordDecodeAttempt::begin(
        Default::default(),
        u64::try_from(bytes.len()).unwrap(),
        narrow_limits,
    )
    .unwrap();
    assert_eq!(
        decode_payload(&mut input, &mut attempt).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::NestingDepthBudgetExceeded
    );
}

fn encode_payload(
    record: &worth_query_declaration::facade::application_schema::WorthQueryPortableApplicationSchemaRecord,
    limits: WorthQueryPackageArchiveLimits,
) -> Vec<u8> {
    let payload_bytes = payload_byte_length(record, limits).unwrap();
    let mut output = BinaryOutput::with_capacity(usize::try_from(payload_bytes).unwrap());
    write_payload(record, &mut output, limits).unwrap();
    output.into_bytes()
}

fn encode_member(member: &ApplicationSchemaMember) -> Vec<u8> {
    let mut output = BinaryOutput::with_capacity(256);
    super::member::write(&mut output, member).unwrap();
    output.into_bytes()
}

fn decode_member(bytes: &[u8]) -> ApplicationSchemaMember {
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let mut input = crate::binary_input::BinaryInput::new(bytes);
    let mut attempt = RecordDecodeAttempt::begin(
        Default::default(),
        u64::try_from(bytes.len()).unwrap(),
        limits,
    )
    .unwrap();
    let member = super::member::decode(&mut input, &mut attempt).unwrap();
    assert!(input.is_finished());
    member
}

fn frame_payload(payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(12 + payload.len());
    frame.extend_from_slice(
        &crate::record::WORTH_QUERY_PACKAGE_ARCHIVE_RECORD_PROTOCOL_VERSION.to_be_bytes(),
    );
    frame.extend_from_slice(&8_u16.to_be_bytes());
    frame.extend_from_slice(&0_u32.to_be_bytes());
    frame.extend_from_slice(&u32::try_from(payload.len()).unwrap().to_be_bytes());
    frame.extend_from_slice(payload);
    frame
}
