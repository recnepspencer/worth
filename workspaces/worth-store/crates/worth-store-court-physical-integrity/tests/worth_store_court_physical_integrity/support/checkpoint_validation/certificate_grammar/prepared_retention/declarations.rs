pub(crate) use {worth_store_physical_integrity::CheckpointFooterValidationBasis, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::UntrustedPhysicalArtifact, worth_store_physical_integrity::VerifiedCheckpointStream, worth_store_physical_integrity::VerifiedCheckpointStreamAssemblyDenial};

pub(crate) use {crate::literal_vectors::checkpoint_validation::BINDING, crate::literal_vectors::checkpoint_validation::BINDING_COMPACTION, crate::literal_vectors::checkpoint_validation::DIRTY_BASIS, crate::literal_vectors::checkpoint_validation::HEADER};
pub(crate) use {crate::checkpoint_validation::support::binding_scope, crate::checkpoint_validation::support::compaction_scope, crate::checkpoint_validation::support::dirty_scope, crate::checkpoint_validation::support::header_scope_staged, crate::checkpoint_validation::support::identity, crate::checkpoint_validation::support::validate_binding, crate::checkpoint_validation::support::validate_compaction, crate::checkpoint_validation::support::validate_dirty, crate::checkpoint_validation::support::validate_footer, crate::checkpoint_validation::support::validate_header, crate::checkpoint_validation::support::BINDING_OFFSET, crate::checkpoint_validation::support::COMPACTION_OFFSET, crate::checkpoint_validation::support::DIRTY_OFFSET, crate::checkpoint_validation::support::FOOTER_OFFSET};

pub(crate) fn assert_exact_prepared_retention(frames: &[Vec<u8>], footer_bytes: &[u8]) {
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
                worth_store_physical_integrity::PhysicalByteRange::new(start as u64, frame.len() as u64).unwrap(),
                &bytes[start..offset],
            )
        })
        .collect();
    let dirty_records = [&dirty];
    let bindings = [&binding];
    let footer = validate_footer(
        &bytes[footer_offset..],
        worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_footer(
            checkpoint,
            worth_store_physical_integrity::PhysicalByteRange::new(footer_offset as u64, footer_bytes.len() as u64).unwrap(),
        ),
        worth_store_physical_integrity::CheckpointFooterValidationBasis::from_record_references(
            &header,
            &dirty_records,
            &compaction,
            &bindings,
        )
        .with_certificates(&certificates),
    );
    let preflight = || {
        worth_store_physical_integrity::VerifiedCheckpointStream::validate_records_with_certificates(
            worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes),
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
        worth_store_physical_integrity::VerifiedCheckpointStreamAssemblyDenial::RecordScopeMismatch
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
        worth_store_physical_integrity::VerifiedCheckpointStreamAssemblyDenial::InputIncarnationMismatch
    );
}