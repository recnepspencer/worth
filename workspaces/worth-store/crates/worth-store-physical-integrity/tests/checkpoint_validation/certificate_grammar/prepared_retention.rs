use worth_store_physical_integrity::{
    CheckpointFooterValidationBasis, PhysicalArtifactScope, PhysicalByteRange,
    UntrustedPhysicalArtifact, VerifiedCheckpointStream, VerifiedCheckpointStreamAssemblyDenial,
};

use super::super::literal_vectors::{BINDING, BINDING_COMPACTION, DIRTY_BASIS, HEADER};
use super::super::support::{
    binding_scope, compaction_scope, dirty_scope, header_scope_staged, identity, validate_binding,
    validate_compaction, validate_dirty, validate_footer, validate_header, BINDING_OFFSET,
    COMPACTION_OFFSET, DIRTY_OFFSET, FOOTER_OFFSET,
};

pub(super) fn assert_exact_prepared_retention(frames: &[Vec<u8>], footer_bytes: &[u8]) {
    let mut bytes = Vec::new();
    for record in [
        &HEADER[..],
        &DIRTY_BASIS[..],
        &BINDING_COMPACTION[..],
        &BINDING[..],
    ] {
        bytes.extend_from_slice(record);
    }
    for frame in frames {
        bytes.extend_from_slice(frame);
    }
    let footer_offset = bytes.len();
    bytes.extend_from_slice(footer_bytes);
    let checkpoint = identity();
    let header = validate_header(&bytes[..DIRTY_OFFSET as usize], header_scope_staged());
    let dirty = validate_dirty(
        &bytes[DIRTY_OFFSET as usize..COMPACTION_OFFSET as usize],
        dirty_scope(checkpoint),
    );
    let compaction = validate_compaction(
        &bytes[COMPACTION_OFFSET as usize..BINDING_OFFSET as usize],
        compaction_scope(checkpoint),
    );
    let binding = validate_binding(
        &bytes[BINDING_OFFSET as usize..FOOTER_OFFSET as usize],
        binding_scope(checkpoint, BINDING.len() as u64),
    );
    let mut offset = FOOTER_OFFSET as usize;
    let certificates: Vec<_> = frames
        .iter()
        .map(|frame| {
            let start = offset;
            offset += frame.len();
            (
                PhysicalByteRange::new(start as u64, frame.len() as u64).unwrap(),
                &bytes[start..offset],
            )
        })
        .collect();
    let dirty_records = [&dirty];
    let bindings = [&binding];
    let footer = validate_footer(
        &bytes[footer_offset..],
        PhysicalArtifactScope::checkpoint_footer(
            checkpoint,
            PhysicalByteRange::new(footer_offset as u64, footer_bytes.len() as u64).unwrap(),
        ),
        CheckpointFooterValidationBasis::from_record_references(
            &header,
            &dirty_records,
            &compaction,
            &bindings,
        )
        .with_certificates(&certificates),
    );
    let preflight = || {
        VerifiedCheckpointStream::validate_records_with_certificates(
            UntrustedPhysicalArtifact::from_bounded_bytes(&bytes),
            &header,
            &dirty_records,
            &compaction,
            &bindings,
            &certificates,
            &footer,
        )
        .unwrap()
    };
    let facts = preflight().facts();
    assert_eq!(facts.encoded_bytes(), bytes.len() as u64);
    assert_eq!(facts.certificate_record_count(), frames.len() as u64);
    assert_eq!(
        facts.certificate_record_bytes(),
        frames.iter().map(|frame| frame.len() as u64).sum::<u64>()
    );
    assert_eq!(preflight().certificates(), certificates.as_slice());

    let prepared: Box<[Box<[u8]>]> = frames
        .iter()
        .map(|frame| frame.clone().into_boxed_slice())
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let outer_pointer = prepared.as_ptr();
    let frame_pointer = prepared[0].as_ptr();
    let stream = preflight().retain_prepared_certificates(prepared).unwrap();
    assert_eq!(stream.facts(), facts);
    assert_eq!(stream.certificate_records().as_ptr(), outer_pointer);
    assert_eq!(stream.certificate_records()[0].as_ptr(), frame_pointer);
    assert_eq!(
        stream.owned_heap_bytes(),
        Some(
            (frames.len() * core::mem::size_of::<Box<[u8]>>()) as u64
                + frames.iter().map(|frame| frame.len() as u64).sum::<u64>()
        )
    );

    assert_eq!(
        preflight()
            .retain_prepared_certificates(Box::new([]))
            .unwrap_err(),
        VerifiedCheckpointStreamAssemblyDenial::RecordScopeMismatch
    );
    let mut altered: Vec<_> = frames
        .iter()
        .map(|frame| frame.clone().into_boxed_slice())
        .collect();
    altered[0][0] ^= 1;
    assert_eq!(
        preflight()
            .retain_prepared_certificates(altered.into_boxed_slice())
            .unwrap_err(),
        VerifiedCheckpointStreamAssemblyDenial::InputIncarnationMismatch
    );
}
