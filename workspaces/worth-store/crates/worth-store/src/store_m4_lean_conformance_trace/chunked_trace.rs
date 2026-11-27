//! Conformance trace emitter for `StoreM4.ChunkedCoverage`, driven by the real
//! compiled `worth_store_blob_chunks::BlobChunkSequenceAdmission` fixed-size
//! partition enforcer.

use super::Trace;
use worth_foundational::{
    aspects, AspectContract, AspectKey, AspectValue, InternedString, ScalarAspectType,
};
use worth_proof::TransitionOutcome;
use worth_store_aspect_native::{
    StoreAspectAuthorityInput, StoreAspectBoundaryFact, StoreAspectIdentity,
    StorePhysicalBoundaryWitness,
};
use worth_store_authority::require_current_store_authority;
use worth_store_blob_chunks::{
    AdmittedBlobChunkSecurity, BlobChunkSecurityScope, BlobChunkSequenceAdmission, BlobChunkSize,
    BlobChunkingRuleAdmission,
};
use worth_store_contracts::{StorePhysicalAuthorityWitness, ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE};
use worth_store_physical_format::{
    PhysicalBinaryEncodingWitness, PhysicalChunkChecksumAuthority,
    PhysicalChunkPayloadIntegrityWitness, PhysicalGeneration, PhysicalGenerationAuthority,
    PhysicalHeaderAuthority, PhysicalPageId, PhysicalPageKind, PhysicalPageRecordAuthority,
    PhysicalRecordSlot, PhysicalReferenceAuthority, PhysicalSegmentId, SlotAppendRequest,
    StorePhysicalChunkWriteReceipt, PHYSICAL_HEADER_LENGTH,
};
use worth_store_security::{
    admit_store_security_scope, StoreAuthenticityRequirement, StoreAuthenticityRequirementClass,
    StoreCustodyPosture, StoreKeyScope, StoreKeyVersionPosture,
    StoreSecurityScopeAdmissionExpectation, StoreSecurityScopeAdmissionRequest, StoreTenantScope,
};

fn blob_scope(identity_key: &str) -> BlobChunkSecurityScope {
    let key: AspectKey = aspects().vocabulary().key(identity_key).unwrap();
    let contract: AspectContract = aspects()
        .contract()
        .for_key(key.clone())
        .identified_by(aspects().vocabulary().identity(1))
        .at_revision(aspects().vocabulary().revision(1))
        .scalar(ScalarAspectType::String);
    let validated = match aspects()
        .validate()
        .against(&contract)
        .value(AspectValue::String(InternedString::from("chunk-authority")))
    {
        TransitionOutcome::Success(v) => v,
        outcome => panic!("scalar validation failed: {outcome:?}"),
    };
    let state = match aspects().authoritative_state().admit([validated]) {
        TransitionOutcome::Success(s) => s,
        outcome => panic!("state admission failed: {outcome:?}"),
    };
    let physical = StorePhysicalBoundaryWitness::from_physical_authority(
        StorePhysicalAuthorityWitness::for_aspect_native_boundary(
            ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
        )
        .unwrap(),
    )
    .unwrap();
    let fact = StoreAspectBoundaryFact::from_admitted_state(
        StoreAspectIdentity::from_aspect_key(key),
        StoreAspectAuthorityInput::new(state, physical),
    )
    .unwrap();
    let authority = require_current_store_authority(fact);
    let authenticity = StoreAuthenticityRequirement::required(
        StoreAuthenticityRequirementClass::AuthenticatedBlobChunk,
    );
    let expectation = StoreSecurityScopeAdmissionExpectation::new(
        StoreKeyScope::BlobChunkEnvelope,
        StoreTenantScope::TenantPhysicalBoundary,
        authenticity,
        StoreCustodyPosture::InternalStoreCustody,
    );
    let request = StoreSecurityScopeAdmissionRequest::new(
        &authority,
        StoreKeyScope::BlobChunkEnvelope,
        StoreKeyVersionPosture::Current,
        StoreTenantScope::TenantPhysicalBoundary,
        authenticity,
        StoreCustodyPosture::InternalStoreCustody,
        expectation,
    );
    let admitted = match admit_store_security_scope(request) {
        TransitionOutcome::Success(s) => s,
        outcome => panic!("security scope admission failed: {outcome:?}"),
    };
    let handoff = AdmittedBlobChunkSecurity::from_admitted_security_scope(admitted).unwrap();
    BlobChunkSecurityScope::from_admitted_blob_security(handoff)
}

fn physical_payload(bytes: &[u8]) -> PhysicalChunkPayloadIntegrityWitness {
    let encoding = PhysicalBinaryEncodingWitness::physical_format_canonical().unwrap();
    let headers = PhysicalHeaderAuthority::for_canonical_physical_format(encoding.clone());
    let records = PhysicalPageRecordAuthority::for_canonical_physical_format(
        PhysicalHeaderAuthority::for_canonical_physical_format(encoding),
    );
    let generations = PhysicalGenerationAuthority::for_canonical_physical_format();
    let references = PhysicalReferenceAuthority::for_canonical_physical_format();
    let page_cell = generations
        .page_cell(
            PhysicalSegmentId::from_raw(7).unwrap(),
            PhysicalPageId::from_raw(11).unwrap(),
        )
        .with_page_generation(PhysicalGeneration::from_raw(5).unwrap());
    let slot_cell = generations
        .slot_cell(
            PhysicalSegmentId::from_raw(7).unwrap(),
            PhysicalPageId::from_raw(11).unwrap(),
            PhysicalRecordSlot::from_raw(1).unwrap(),
        )
        .with_slot_generation(PhysicalGeneration::from_raw(9).unwrap());
    let encode_page = |payload: &[u8]| {
        let mut buf = Vec::with_capacity(usize::from(PHYSICAL_HEADER_LENGTH) + payload.len());
        buf.extend_from_slice(&headers.encode_page_header(
            page_cell,
            PhysicalPageKind::DataPage,
            u32::try_from(payload.len()).unwrap(),
        ));
        buf.extend_from_slice(payload);
        buf
    };
    let empty_page = encode_page(&[]);
    let empty_hdr = records
        .decode_record_page_header(page_cell, &empty_page, PhysicalPageKind::DataPage)
        .unwrap();
    let empty_payload = records
        .admit_record_page_payload(&empty_page, empty_hdr.witness())
        .unwrap();
    let append = records
        .append_record(empty_payload, SlotAppendRequest::ordinary(slot_cell, bytes))
        .unwrap();
    let full_page = encode_page(append.page_payload());
    let full_hdr = records
        .decode_record_page_header(page_cell, &full_page, PhysicalPageKind::DataPage)
        .unwrap();
    let full_payload = records
        .admit_record_page_payload(&full_page, full_hdr.witness())
        .unwrap();
    let validation = references
        .validate_page_slot(append.reference_admission(), slot_cell)
        .unwrap();
    let located = records.locate_record(full_payload, validation).unwrap();
    let receipt =
        StorePhysicalChunkWriteReceipt::from_page_record_view(located.record_view()).unwrap();
    PhysicalChunkChecksumAuthority::canonical_blob_checksum()
        .admit_store_payload(receipt)
        .unwrap()
}

fn ranges_json(ranges: &[(u64, u64)]) -> String {
    let items: Vec<String> = ranges.iter().map(|(s, e)| format!("[{s},{e}]")).collect();
    format!("[{}]", items.join(","))
}

fn run_partition_case(
    trace: &mut Trace,
    label: &str,
    chunk_width: u64,
    total: u64,
    submitted: &[(u64, u64)],
) {
    let outcome = BlobChunkSize::from_bytes(chunk_width)
        .and_then(BlobChunkingRuleAdmission::fixed_size)
        .and_then(|rule| BlobChunkSequenceAdmission::start(blob_scope(label), rule, total))
        .and_then(|mut seq| {
            for &(start, end) in submitted {
                let len = usize::try_from(end.saturating_sub(start)).unwrap();
                let bytes = vec![0xA5; len];
                seq = seq.push_payload(start, physical_payload(&bytes))?;
            }
            seq.finish()
        });

    let (admitted, output_ranges, chunk_count) = match outcome {
        Ok(seq) => {
            let leaves: Vec<(u64, u64)> = seq
                .proof_frontier()
                .ordered_leaves()
                .iter()
                .map(|leaf| (leaf.byte_range().start(), leaf.byte_range().end()))
                .collect();
            (true, leaves, seq.proof_frontier().chunk_count())
        }
        Err(_) => (false, Vec::new(), 0),
    };

    trace.push(
        "chunked_partition",
        format!(
            r#""chunk_width":{chunk_width},"total":{total},"submitted":{},"admitted":{admitted},"output_ranges":{},"chunk_count":{chunk_count}"#,
            ranges_json(submitted),
            ranges_json(&output_ranges),
        ),
    );
}

pub(super) fn record(trace: &mut Trace) {
    // Accepted exact, ragged-tail, and single-chunk fixed partitions.
    run_partition_case(trace, "chunk.exact", 4, 12, &[(0, 4), (4, 8), (8, 12)]);
    run_partition_case(trace, "chunk.ragged", 4, 10, &[(0, 4), (4, 8), (8, 10)]);
    run_partition_case(trace, "chunk.single", 8, 5, &[(0, 5)]);
    run_partition_case(trace, "chunk.three", 3, 7, &[(0, 3), (3, 6), (6, 7)]);

    // Refused partitions: missing tail, leading gap, overlap, short interior, zero width/total.
    run_partition_case(trace, "chunk.missing_tail", 4, 8, &[(0, 4)]);
    run_partition_case(trace, "chunk.gap", 4, 8, &[(4, 8)]);
    run_partition_case(trace, "chunk.overlap", 4, 8, &[(0, 4), (0, 4)]);
    run_partition_case(
        trace,
        "chunk.short_interior",
        4,
        8,
        &[(0, 2), (2, 6), (6, 8)],
    );
    run_partition_case(trace, "chunk.zero_width", 0, 8, &[]);
    run_partition_case(trace, "chunk.zero_total", 4, 0, &[]);
}
