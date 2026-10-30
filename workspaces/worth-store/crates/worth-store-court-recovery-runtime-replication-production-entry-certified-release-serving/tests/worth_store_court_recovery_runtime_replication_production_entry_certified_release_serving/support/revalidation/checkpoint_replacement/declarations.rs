//! Rebuilds the persisted checkpoint as a distinct, equally long stream on
//! the current production encoder, including the C.11 certified schema and
//! its certificate (custody) records.

pub(crate) use {
    worth_store_physical_format::checkpoint_certificate_frame_bytes,
    worth_store_physical_format::store_namespace::StableStoreIdentity,
    worth_store_physical_format::CheckpointBindingCompactionHeader,
    worth_store_physical_format::CheckpointDirtyFrameBasis,
    worth_store_physical_format::CheckpointStreamEncoder,
    worth_store_physical_format::PhysicalCheckpointSource,
    worth_store_physical_format::RecordArtifactFile,
    worth_store_physical_format::RecordFrameCoordinate,
    worth_store_physical_format::CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES,
    worth_store_physical_format::CHECKPOINT_BINDING_RECORD_PREFIX_BYTES,
    worth_store_physical_format::CHECKPOINT_CERTIFICATE_PREFIX_BYTES,
    worth_store_physical_format::CHECKPOINT_CERTIFIED_FOOTER_RECORD_BYTES,
    worth_store_physical_format::CHECKPOINT_CERTIFIED_SCHEMA,
    worth_store_physical_format::CHECKPOINT_DIRTY_FRAME_RECORD_BYTES,
    worth_store_physical_format::CHECKPOINT_STREAM_FOOTER_RECORD_BYTES,
    worth_store_physical_format::CHECKPOINT_STREAM_HEADER_RECORD_BYTES,
};
pub(crate) use {
    worth_store_physical_integrity::project_checkpoint_binding_frame_length,
    worth_store_physical_integrity::validate_checkpoint_binding,
    worth_store_physical_integrity::validate_checkpoint_binding_compaction,
    worth_store_physical_integrity::validate_checkpoint_dirty_basis,
    worth_store_physical_integrity::validate_checkpoint_footer,
    worth_store_physical_integrity::validate_checkpoint_footer_envelope,
    worth_store_physical_integrity::validate_checkpoint_stream_header,
    worth_store_physical_integrity::CheckpointBindingCompactionIntegrityValidation,
    worth_store_physical_integrity::CheckpointBindingIntegrityValidation,
    worth_store_physical_integrity::CheckpointDirtyBasisIntegrityValidation,
    worth_store_physical_integrity::CheckpointFooterEnvelopeIntegrityValidation,
    worth_store_physical_integrity::CheckpointFooterIntegrityValidation,
    worth_store_physical_integrity::CheckpointFooterValidationBasis,
    worth_store_physical_integrity::CheckpointStreamHeaderIntegrityValidation,
    worth_store_physical_integrity::CheckpointStreamHeaderScopeIdentity,
    worth_store_physical_integrity::IntegrityValidatedCheckpointBinding,
    worth_store_physical_integrity::IntegrityValidatedCheckpointBindingCompaction,
    worth_store_physical_integrity::IntegrityValidatedCheckpointDirtyBasis,
    worth_store_physical_integrity::PhysicalArtifactScope,
    worth_store_physical_integrity::PhysicalByteRange,
    worth_store_physical_integrity::UntrustedPhysicalArtifact,
};

pub(crate) fn replace_with_distinct_valid_checkpoint(
    root: &std::path::Path,
    store: StableStoreIdentity,
) -> u64 {
    let path = root.join("families").join("checkpoint.current");
    let original_bytes = std::fs::read(&path).unwrap();
    let original = admit_original(&original_bytes, store);
    let (mut encoder, header) = if original.certificates.is_some() {
        worth_store_physical_format::CheckpointStreamEncoder::begin_certified(original.source)
    } else {
        worth_store_physical_format::CheckpointStreamEncoder::begin(original.source)
    };
    let mut replacement = header;
    for ordinal in 0..original.dirty_records {
        let coordinate = worth_store_physical_format::RecordFrameCoordinate::new(
            worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
            ordinal,
            1,
        )
        .unwrap();
        replacement.extend_from_slice(&encoder.encode_dirty_basis(
            worth_store_physical_format::CheckpointDirtyFrameBasis::new(
                coordinate,
                ordinal.saturating_add(17),
            ),
        ));
    }
    let header = worth_store_physical_format::CheckpointBindingCompactionHeader::new(
        original.compaction_generation.saturating_add(1),
        original.wal_cutoff_lsn_exclusive,
    )
    .unwrap();
    let (mut compaction, header_record) = encoder.begin_binding_compaction(header);
    replacement.extend_from_slice(&header_record);
    for binding in original.bindings {
        replacement.extend_from_slice(&compaction.encode_binding_record(&binding).unwrap());
    }
    for certificate in original.certificates.unwrap_or_default() {
        compaction.include_certificate_record(&certificate).unwrap();
        replacement.extend_from_slice(&certificate);
    }
    let (_, footer) = compaction.finish();
    replacement.extend_from_slice(&footer);
    assert_eq!(replacement.len(), original_bytes.len());
    std::fs::write(path, &replacement).unwrap();
    replacement.len() as u64
}

pub(crate) struct AdmittedOriginal {
    pub(crate) source: PhysicalCheckpointSource,
    pub(crate) dirty_records: u64,
    pub(crate) compaction_generation: u64,
    pub(crate) wal_cutoff_lsn_exclusive: u64,
    pub(crate) bindings: Vec<Box<[u8]>>,
    /// Present exactly when the stream uses the certified schema.
    pub(crate) certificates: Option<Vec<Box<[u8]>>>,
}

pub(crate) fn admit_original(bytes: &[u8], store: StableStoreIdentity) -> AdmittedOriginal {
    let header_range = range(0, CHECKPOINT_STREAM_HEADER_RECORD_BYTES);
    let header_scope =
        worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_stream_header(
            worth_store_physical_integrity::CheckpointStreamHeaderScopeIdentity::staged(store),
            header_range,
        );
    let worth_store_physical_integrity::CheckpointStreamHeaderIntegrityValidation::Intact(header) =
        validate_checkpoint_stream_header(input(bytes, header_range), header_scope).0
    else {
        panic!("original checkpoint header must be admitted")
    };
    let source = header.source();
    let identity = source.identity();
    let certified = bytes[8] == CHECKPOINT_CERTIFIED_SCHEMA;
    let (footer_range, footer) = admit_footer_envelope(bytes, identity, certified);
    let footer_scope = worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_footer(
        identity,
        footer_range,
    );
    let (dirty, offset) =
        admit_dirty_records(bytes, identity, &footer, header_range.end_exclusive());
    let (compaction, offset) = admit_compaction(bytes, identity, offset);
    let (bindings, payloads) =
        admit_bindings(bytes, identity, footer.binding_record_count(), offset);
    let certificates = frame_certificates(
        bytes,
        offset_of_footer(&bindings, offset),
        footer_range.offset(),
    );
    assert_eq!(certificates.len() as u64, footer.certificate_record_count());
    assert!(certified || certificates.is_empty());
    let worth_store_physical_integrity::CheckpointFooterIntegrityValidation::Intact(_) =
        validate_checkpoint_footer(
            input(bytes, footer_range),
            footer_scope,
            worth_store_physical_integrity::CheckpointFooterValidationBasis::new(
                &header,
                &dirty,
                &compaction,
                &bindings,
            )
            .with_certificates(&certificates),
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
        certificates: certified.then(|| owned_records(&certificates)),
    }
}

/// Admits the footer envelope whose size the stream schema selects.
pub(crate) fn admit_footer_envelope(
    bytes: &[u8],
    identity: worth_store_physical_format::PhysicalCheckpointIdentity,
    certified: bool,
) -> (
    PhysicalByteRange,
    worth_store_physical_format::CheckpointStreamFooter,
) {
    let footer_bytes = if certified {
        CHECKPOINT_CERTIFIED_FOOTER_RECORD_BYTES
    } else {
        CHECKPOINT_STREAM_FOOTER_RECORD_BYTES
    };
    let footer_range = range(bytes.len() - footer_bytes, footer_bytes);
    let footer_scope = worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_footer(
        identity,
        footer_range,
    );
    let worth_store_physical_integrity::CheckpointFooterEnvelopeIntegrityValidation::Intact(
        envelope,
    ) = validate_checkpoint_footer_envelope(input(bytes, footer_range), footer_scope).0
    else {
        panic!("original checkpoint footer envelope must be admitted")
    };
    (footer_range, envelope.routing_projection().footer())
}

pub(crate) fn owned_records(records: &[(PhysicalByteRange, &[u8])]) -> Vec<Box<[u8]>> {
    records
        .iter()
        .map(|(_, record)| Box::<[u8]>::from(*record))
        .collect()
}

/// Frames the certificate records between the last binding and the footer.
pub(crate) fn frame_certificates(
    bytes: &[u8],
    mut offset: u64,
    footer_offset: u64,
) -> Vec<(PhysicalByteRange, &[u8])> {
    let mut records = Vec::new();
    while offset < footer_offset {
        let start = offset as usize;
        let prefix = &bytes[start..start + CHECKPOINT_CERTIFICATE_PREFIX_BYTES];
        let encoded = checkpoint_certificate_frame_bytes(prefix).unwrap();
        let record_range = range(start, encoded);
        records.push((record_range, &bytes[start..start + encoded]));
        offset = record_range.end_exclusive();
    }
    assert_eq!(offset, footer_offset);
    records
}

pub(crate) fn admit_dirty_records<'a>(
    bytes: &'a [u8],
    identity: worth_store_physical_format::PhysicalCheckpointIdentity,
    footer: &worth_store_physical_format::CheckpointStreamFooter,
    mut offset: u64,
) -> (Vec<IntegrityValidatedCheckpointDirtyBasis<'a>>, u64) {
    let mut records = Vec::new();
    for _ in 0..footer.dirty_record_count() {
        let record_range = range(offset as usize, CHECKPOINT_DIRTY_FRAME_RECORD_BYTES);
        let scope = worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_dirty_basis(
            identity,
            record_range,
        );
        let worth_store_physical_integrity::CheckpointDirtyBasisIntegrityValidation::Intact(record) =
            validate_checkpoint_dirty_basis(input(bytes, record_range), scope).0
        else {
            panic!("dirty basis must be admitted")
        };
        records.push(record);
        offset = record_range.end_exclusive();
    }
    (records, offset)
}

pub(crate) fn admit_compaction<'a>(
    bytes: &'a [u8],
    identity: worth_store_physical_format::PhysicalCheckpointIdentity,
    offset: u64,
) -> (IntegrityValidatedCheckpointBindingCompaction<'a>, u64) {
    let record_range = range(
        offset as usize,
        CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES,
    );
    let scope =
        worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_binding_compaction(
            identity,
            record_range,
        );
    let worth_store_physical_integrity::CheckpointBindingCompactionIntegrityValidation::Intact(
        record,
    ) = validate_checkpoint_binding_compaction(input(bytes, record_range), scope).0
    else {
        panic!("binding compaction must be admitted")
    };
    (record, record_range.end_exclusive())
}

pub(crate) fn admit_bindings<'a>(
    bytes: &'a [u8],
    identity: worth_store_physical_format::PhysicalCheckpointIdentity,
    count: u64,
    mut offset: u64,
) -> (Vec<IntegrityValidatedCheckpointBinding<'a>>, Vec<Box<[u8]>>) {
    let mut records = Vec::new();
    let mut payloads = Vec::new();
    for _ in 0..count {
        let prefix = range(offset as usize, CHECKPOINT_BINDING_RECORD_PREFIX_BYTES);
        let prefix_scope =
            worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_binding(
                identity, prefix,
            );
        let encoded = project_checkpoint_binding_frame_length(input(bytes, prefix), prefix_scope)
            .unwrap()
            .encoded_bytes();
        let record_range =
            worth_store_physical_integrity::PhysicalByteRange::new(offset, encoded).unwrap();
        let scope = worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_binding(
            identity,
            record_range,
        );
        let record_input = input(bytes, record_range);
        let worth_store_physical_integrity::CheckpointBindingIntegrityValidation::Intact(record) =
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

pub(crate) fn offset_of_footer(
    bindings: &[IntegrityValidatedCheckpointBinding<'_>],
    initial: u64,
) -> u64 {
    bindings.last().map_or(initial, |binding| {
        binding.scope().byte_range().end_exclusive()
    })
}

pub(crate) fn range(offset: usize, length: usize) -> PhysicalByteRange {
    worth_store_physical_integrity::PhysicalByteRange::new(offset as u64, length as u64).unwrap()
}

pub(crate) fn input(bytes: &[u8], range: PhysicalByteRange) -> UntrustedPhysicalArtifact<'_> {
    worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(
        &bytes[range.offset() as usize..range.end_exclusive() as usize],
    )
}
