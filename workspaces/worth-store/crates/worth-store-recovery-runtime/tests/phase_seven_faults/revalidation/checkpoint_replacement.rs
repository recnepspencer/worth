use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, CheckpointBindingCompactionHeader,
    CheckpointDirtyFrameBasis, CheckpointStreamEncoder, PhysicalCheckpointSource,
    RecordArtifactFile, RecordFrameCoordinate, CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES,
    CHECKPOINT_BINDING_RECORD_PREFIX_BYTES, CHECKPOINT_DIRTY_FRAME_RECORD_BYTES,
    CHECKPOINT_STREAM_FOOTER_RECORD_BYTES, CHECKPOINT_STREAM_HEADER_RECORD_BYTES,
};
use worth_store_physical_integrity::{
    project_checkpoint_binding_frame_length, validate_checkpoint_binding,
    validate_checkpoint_binding_compaction, validate_checkpoint_dirty_basis,
    validate_checkpoint_footer, validate_checkpoint_footer_envelope,
    validate_checkpoint_stream_header, CheckpointBindingCompactionIntegrityValidation,
    CheckpointBindingIntegrityValidation, CheckpointDirtyBasisIntegrityValidation,
    CheckpointFooterEnvelopeIntegrityValidation, CheckpointFooterIntegrityValidation,
    CheckpointFooterValidationBasis, CheckpointStreamHeaderIntegrityValidation,
    CheckpointStreamHeaderScopeIdentity, IntegrityValidatedCheckpointBinding,
    IntegrityValidatedCheckpointBindingCompaction, IntegrityValidatedCheckpointDirtyBasis,
    PhysicalArtifactScope, PhysicalByteRange, UntrustedPhysicalArtifact,
};

pub(super) fn replace_with_distinct_valid_checkpoint(
    root: &std::path::Path,
    store: StableStoreIdentity,
) -> u64 {
    let path = root.join("families").join("checkpoint.current");
    let original_bytes = std::fs::read(&path).unwrap();
    let original = admit_original(&original_bytes, store);
    let (mut encoder, header) = CheckpointStreamEncoder::begin(original.source);
    let mut replacement = header;
    for ordinal in 0..original.dirty_records {
        let coordinate =
            RecordFrameCoordinate::new(RecordArtifactFile::BootstrapCatalog, ordinal, 1).unwrap();
        replacement.extend_from_slice(&encoder.encode_dirty_basis(CheckpointDirtyFrameBasis::new(
            coordinate,
            ordinal.saturating_add(17),
        )));
    }
    let header = CheckpointBindingCompactionHeader::new(
        original.compaction_generation.saturating_add(1),
        original.wal_cutoff_lsn_exclusive,
    )
    .unwrap();
    let (mut compaction, header_record) = encoder.begin_binding_compaction(header);
    replacement.extend_from_slice(&header_record);
    for binding in original.bindings {
        replacement.extend_from_slice(&compaction.encode_binding_record(&binding).unwrap());
    }
    let (_, footer) = compaction.finish();
    replacement.extend_from_slice(&footer);
    assert_eq!(replacement.len(), original_bytes.len());
    std::fs::write(path, &replacement).unwrap();
    replacement.len() as u64
}

struct AdmittedOriginal {
    source: PhysicalCheckpointSource,
    dirty_records: u64,
    compaction_generation: u64,
    wal_cutoff_lsn_exclusive: u64,
    bindings: Vec<Box<[u8]>>,
}

fn admit_original(bytes: &[u8], store: StableStoreIdentity) -> AdmittedOriginal {
    let header_range = range(0, CHECKPOINT_STREAM_HEADER_RECORD_BYTES);
    let header_scope = PhysicalArtifactScope::checkpoint_stream_header(
        CheckpointStreamHeaderScopeIdentity::staged(store),
        header_range,
    );
    let CheckpointStreamHeaderIntegrityValidation::Intact(header) =
        validate_checkpoint_stream_header(input(bytes, header_range), header_scope).0
    else {
        panic!("original checkpoint header must be admitted")
    };
    let source = header.source();
    let identity = source.identity();
    let footer_range = range(
        bytes.len() - CHECKPOINT_STREAM_FOOTER_RECORD_BYTES,
        CHECKPOINT_STREAM_FOOTER_RECORD_BYTES,
    );
    let footer_scope = PhysicalArtifactScope::checkpoint_footer(identity, footer_range);
    let CheckpointFooterEnvelopeIntegrityValidation::Intact(envelope) =
        validate_checkpoint_footer_envelope(input(bytes, footer_range), footer_scope).0
    else {
        panic!("original checkpoint footer envelope must be admitted")
    };
    let footer = envelope.routing_projection().footer();
    let (dirty, offset) =
        admit_dirty_records(bytes, identity, &footer, header_range.end_exclusive());
    let (compaction, offset) = admit_compaction(bytes, identity, offset);
    let (bindings, payloads) =
        admit_bindings(bytes, identity, footer.binding_record_count(), offset);
    assert_eq!(offset_of_footer(&bindings, offset), footer_range.offset());
    let CheckpointFooterIntegrityValidation::Intact(_) = validate_checkpoint_footer(
        input(bytes, footer_range),
        footer_scope,
        CheckpointFooterValidationBasis::new(&header, &dirty, &compaction, &bindings),
    )
    .0
    else {
        panic!("original checkpoint aggregate must be admitted")
    };
    AdmittedOriginal {
        source,
        dirty_records: footer.dirty_record_count(),
        compaction_generation: compaction.generation(),
        wal_cutoff_lsn_exclusive: compaction.wal_cutoff_lsn_exclusive(),
        bindings: payloads,
    }
}

fn admit_dirty_records<'a>(
    bytes: &'a [u8],
    identity: worth_store_physical_format::PhysicalCheckpointIdentity,
    footer: &worth_store_physical_format::CheckpointStreamFooter,
    mut offset: u64,
) -> (Vec<IntegrityValidatedCheckpointDirtyBasis<'a>>, u64) {
    let mut records = Vec::new();
    for _ in 0..footer.dirty_record_count() {
        let record_range = range(offset as usize, CHECKPOINT_DIRTY_FRAME_RECORD_BYTES);
        let scope = PhysicalArtifactScope::checkpoint_dirty_basis(identity, record_range);
        let CheckpointDirtyBasisIntegrityValidation::Intact(record) =
            validate_checkpoint_dirty_basis(input(bytes, record_range), scope).0
        else {
            panic!("dirty basis must be admitted")
        };
        records.push(record);
        offset = record_range.end_exclusive();
    }
    (records, offset)
}

fn admit_compaction<'a>(
    bytes: &'a [u8],
    identity: worth_store_physical_format::PhysicalCheckpointIdentity,
    offset: u64,
) -> (IntegrityValidatedCheckpointBindingCompaction<'a>, u64) {
    let record_range = range(
        offset as usize,
        CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES,
    );
    let scope = PhysicalArtifactScope::checkpoint_binding_compaction(identity, record_range);
    let CheckpointBindingCompactionIntegrityValidation::Intact(record) =
        validate_checkpoint_binding_compaction(input(bytes, record_range), scope).0
    else {
        panic!("binding compaction must be admitted")
    };
    (record, record_range.end_exclusive())
}

fn admit_bindings<'a>(
    bytes: &'a [u8],
    identity: worth_store_physical_format::PhysicalCheckpointIdentity,
    count: u64,
    mut offset: u64,
) -> (Vec<IntegrityValidatedCheckpointBinding<'a>>, Vec<Box<[u8]>>) {
    let mut records = Vec::new();
    let mut payloads = Vec::new();
    for _ in 0..count {
        let prefix = range(offset as usize, CHECKPOINT_BINDING_RECORD_PREFIX_BYTES);
        let prefix_scope = PhysicalArtifactScope::checkpoint_binding(identity, prefix);
        let encoded = project_checkpoint_binding_frame_length(input(bytes, prefix), prefix_scope)
            .unwrap()
            .encoded_bytes();
        let record_range = PhysicalByteRange::new(offset, encoded).unwrap();
        let scope = PhysicalArtifactScope::checkpoint_binding(identity, record_range);
        let record_input = input(bytes, record_range);
        let CheckpointBindingIntegrityValidation::Intact(record) =
            validate_checkpoint_binding(record_input, scope).0
        else {
            panic!("binding must be admitted")
        };
        let projection = record.project_payload(record_input, identity).unwrap();
        payloads.push(record_input.bytes()[projection.payload_range()].into());
        records.push(record);
        offset = record_range.end_exclusive();
    }
    (records, payloads)
}

fn offset_of_footer(bindings: &[IntegrityValidatedCheckpointBinding<'_>], initial: u64) -> u64 {
    bindings.last().map_or(initial, |binding| {
        binding.scope().byte_range().end_exclusive()
    })
}

fn range(offset: usize, length: usize) -> PhysicalByteRange {
    PhysicalByteRange::new(offset as u64, length as u64).unwrap()
}

fn input(bytes: &[u8], range: PhysicalByteRange) -> UntrustedPhysicalArtifact<'_> {
    UntrustedPhysicalArtifact::from_bounded_bytes(
        &bytes[range.offset() as usize..range.end_exclusive() as usize],
    )
}
