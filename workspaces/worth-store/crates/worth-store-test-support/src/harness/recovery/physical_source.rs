use std::num::NonZeroU64;
use std::path::Path;

use worth_store_physical_format::{
    durable_artifact_checksum, CheckpointBindingCompactionHeader, CheckpointRootBasis,
    CheckpointStreamEncoder, CheckpointWalSourceRange, CurrentPhysicalRecordPlacement,
    DurableInlineRecordPlacement, DurablePhysicalRootManifest, DurableRootSelector,
    FreeSpaceBlockReference, FreeSpaceKey, PersistedRecordIdentity, PhysicalGeneration,
    PhysicalGenerationAuthority, PhysicalPageId, PhysicalRecordFormatDeclaration,
    PhysicalRecordSlot, PhysicalRootRoutingBlock, PhysicalSegmentId, RootSelectorIdentity,
    RootSelectorRole, CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES,
    CHECKPOINT_STREAM_FOOTER_RECORD_BYTES, CHECKPOINT_STREAM_HEADER_RECORD_BYTES,
};
use worth_store_physical_integrity::{
    validate_checkpoint_binding_compaction, validate_checkpoint_footer,
    validate_checkpoint_stream_header, CheckpointBindingCompactionIntegrityValidation,
    CheckpointFooterIntegrityValidation, CheckpointFooterValidationBasis,
    CheckpointStreamHeaderIntegrityValidation, CheckpointStreamHeaderScopeIdentity,
    PhysicalArtifactScope, PhysicalByteRange, UntrustedPhysicalArtifact, VerifiedCheckpointStream,
};
use worth_store_recovery_physics::{
    admit_physical_page_facts, admit_physical_wal_tail, observe_structured_physical_root_candidate,
    select_current_previous_root, select_physical_recovery_sources,
    PhysicalManifestBlockProjection, PhysicalSourceSelection, PhysicalWalSegmentCandidate,
    SelectedCompactionProduct,
};
use worth_store_wal::{
    inspect_verified_wal_segment, prepare_wal_frame_append, WalSegmentArtifactIdentity,
    WalSegmentGeneration, WalSegmentId,
};

/// Builds the exact current recovery-physics source selection required by the
/// layout replay fixture. The root is a typed structured physics candidate;
/// checkpoint and tail admission plus replay ownership remain with the fixture.
pub fn deterministic_checkpoint_plus_tail_source() -> PhysicalSourceSelection {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let store = stable_store();
    let (manifest, block_reference, block) = root_manifest(format);
    let selector = DurableRootSelector::new(
        store,
        format,
        RootSelectorIdentity::new(1).unwrap(),
        RootSelectorRole::Current,
        1,
        None,
        None,
    )
    .unwrap();
    let current = observe_structured_physical_root_candidate(selector, manifest, format);
    let root = select_current_previous_root(
        current,
        worth_store_recovery_physics::PhysicalRootSlotObservation::Absent,
        None,
    )
    .unwrap();
    let page_facts = admit_physical_page_facts(
        root.selected(),
        vec![PhysicalManifestBlockProjection::from_projected_block(
            block_reference,
            block,
        )],
        1,
        1,
    )
    .unwrap();
    let checkpoint = checkpoint_base(format, store);
    let wal_tail = wal_tail(checkpoint.wal_tail_begin_lsn());
    select_physical_recovery_sources(
        root,
        page_facts,
        None,
        Some(checkpoint),
        wal_tail,
        None,
        Vec::new(),
    )
    .unwrap()
}

pub fn deterministic_selected_compaction_product() -> SelectedCompactionProduct {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let checkpoint = checkpoint_base(format, stable_store());
    SelectedCompactionProduct::admit(&checkpoint)
}

fn root_manifest(
    format: PhysicalRecordFormatDeclaration,
) -> (
    DurablePhysicalRootManifest,
    worth_store_physical_format::ManifestBlockReference,
    PhysicalRootRoutingBlock,
) {
    let record = PersistedRecordIdentity::new([1; 16], 1).unwrap();
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let segment = authority
        .segment_cell(PhysicalSegmentId::from_raw(1).unwrap())
        .with_segment_generation(PhysicalGeneration::from_raw(1).unwrap());
    let page = authority
        .page_cell(
            PhysicalSegmentId::from_raw(1).unwrap(),
            PhysicalPageId::from_raw(1).unwrap(),
        )
        .with_page_generation(PhysicalGeneration::from_raw(1).unwrap());
    let slot = authority
        .slot_cell(
            PhysicalSegmentId::from_raw(1).unwrap(),
            PhysicalPageId::from_raw(1).unwrap(),
            PhysicalRecordSlot::from_raw(1).unwrap(),
        )
        .with_slot_generation(PhysicalGeneration::from_raw(1).unwrap());
    let placement = DurableInlineRecordPlacement::new(record, segment, page, slot, 1, 1).unwrap();
    let block = PhysicalRootRoutingBlock::leaf(
        1,
        1,
        1,
        vec![CurrentPhysicalRecordPlacement::Inline(placement)],
        2,
    )
    .unwrap();
    let block_bytes = block.encode(format);
    let block_reference = block.reference(durable_artifact_checksum(&block_bytes));
    let free_space_key = FreeSpaceKey::new(
        worth_store_physical_format::RecordAllocationClass::InlinePage,
        1,
    )
    .unwrap();
    let free_space = FreeSpaceBlockReference::new(1, 1, 0, 1, free_space_key, free_space_key);
    let manifest = DurablePhysicalRootManifest::builder(1, 1, 2, 1)
        .record_count(1)
        .next_block(2)
        .routing_root(Some(block_reference))
        .free_space_root(free_space)
        .admit()
        .unwrap();
    (manifest, block_reference, block)
}

fn checkpoint_base(
    format: PhysicalRecordFormatDeclaration,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
) -> worth_store_recovery_physics::PhysicalCheckpointBase {
    let identity = worth_store_physical_format::PhysicalCheckpointIdentity::new(
        store,
        NonZeroU64::new(1).unwrap(),
    );
    let source = worth_store_physical_format::PhysicalCheckpointSource::concurrent(
        identity,
        CheckpointWalSourceRange::new(10, 20).unwrap(),
        CheckpointRootBasis::new(1, 1),
        1,
    );
    let (encoder, header) = CheckpointStreamEncoder::begin(source);
    let (encoder, compaction) =
        encoder.begin_binding_compaction(CheckpointBindingCompactionHeader::new(1, 20).unwrap());
    let (_, footer) = encoder.finish();
    let mut bytes = header;
    bytes.extend_from_slice(&compaction);
    bytes.extend_from_slice(&footer);
    let verified = admit_empty_checkpoint_stream(&bytes, source);
    let root = root_for_checkpoint(store);
    let root_bytes = root.selected().manifest().encode(format);
    let scope = PhysicalArtifactScope::root_manifest(
        store,
        format,
        1,
        PhysicalByteRange::new(0, root_bytes.len() as u64).unwrap(),
    )
    .unwrap();
    let worth_store_physical_integrity::RootManifestIntegrityValidation::Intact(source_root) =
        worth_store_physical_integrity::validate_root_manifest(
            UntrustedPhysicalArtifact::from_bounded_bytes(&root_bytes),
            scope,
        )
        .0
    else {
        panic!("canonical checkpoint source root must validate")
    };
    worth_store_recovery_physics::PhysicalCheckpointBase::admit(&root, verified, &source_root)
        .unwrap()
}

fn admit_empty_checkpoint_stream(
    bytes: &[u8],
    source: worth_store_physical_format::PhysicalCheckpointSource,
) -> VerifiedCheckpointStream {
    let header_range = PhysicalByteRange::new(0, CHECKPOINT_STREAM_HEADER_RECORD_BYTES as u64)
        .expect("header range");
    let compaction_range = PhysicalByteRange::new(
        CHECKPOINT_STREAM_HEADER_RECORD_BYTES as u64,
        CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES as u64,
    )
    .expect("compaction range");
    let footer_range = PhysicalByteRange::new(
        compaction_range.end_exclusive(),
        CHECKPOINT_STREAM_FOOTER_RECORD_BYTES as u64,
    )
    .expect("footer range");
    let header_scope = PhysicalArtifactScope::checkpoint_stream_header(
        CheckpointStreamHeaderScopeIdentity::staged(source.identity().store_identity()),
        header_range,
    );
    let compaction_scope =
        PhysicalArtifactScope::checkpoint_binding_compaction(source.identity(), compaction_range);
    let footer_scope = PhysicalArtifactScope::checkpoint_footer(source.identity(), footer_range);
    let record = |range: PhysicalByteRange| {
        UntrustedPhysicalArtifact::from_bounded_bytes(
            &bytes[range.offset() as usize..range.end_exclusive() as usize],
        )
    };
    let CheckpointStreamHeaderIntegrityValidation::Intact(header) =
        validate_checkpoint_stream_header(record(header_range), header_scope).0
    else {
        panic!("fixture checkpoint header must validate")
    };
    let CheckpointBindingCompactionIntegrityValidation::Intact(compaction) =
        validate_checkpoint_binding_compaction(record(compaction_range), compaction_scope).0
    else {
        panic!("fixture checkpoint compaction must validate")
    };
    let CheckpointFooterIntegrityValidation::Intact(footer) = validate_checkpoint_footer(
        record(footer_range),
        footer_scope,
        CheckpointFooterValidationBasis::new(&header, &[], &compaction, &[]),
    )
    .0
    else {
        panic!("fixture checkpoint footer must validate")
    };
    VerifiedCheckpointStream::assemble_from_validated_records(
        UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        &header,
        &[],
        &compaction,
        &[],
        &footer,
    )
    .expect("fixture checkpoint stream must assemble from exact admitted records")
}

fn root_for_checkpoint(
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
) -> worth_store_recovery_physics::SelectedPhysicalRoot {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let (manifest, _, _) = root_manifest(format);
    let selector = DurableRootSelector::new(
        store,
        format,
        RootSelectorIdentity::new(1).unwrap(),
        RootSelectorRole::Current,
        1,
        None,
        None,
    )
    .unwrap();
    select_current_previous_root(
        observe_structured_physical_root_candidate(selector, manifest, format),
        worth_store_recovery_physics::PhysicalRootSlotObservation::Absent,
        None,
    )
    .unwrap()
}

fn wal_tail(frontier: u64) -> worth_store_recovery_physics::SelectedPhysicalWalTail {
    let frame = prepare_wal_frame_append(
        Path::new("recovery-tail-fixture"),
        1,
        1,
        frontier,
        frontier + 10,
        "layout-replay-tail",
        b"payload",
    )
    .unwrap();
    let identity = WalSegmentArtifactIdentity::new(
        WalSegmentId::new(1).unwrap(),
        WalSegmentGeneration::new(1).unwrap(),
    );
    let inspection = inspect_verified_wal_segment(identity, frame.encoded_frame())
        .unwrap()
        .inspection();
    admit_physical_wal_tail(
        frontier,
        vec![PhysicalWalSegmentCandidate::from_frame_facts(
            inspection,
            None,
            vec![worth_store_recovery_physics::PhysicalWalFrameFacts::new(
                inspection.lsn_range(),
                inspection.byte_count(),
            )
            .unwrap()],
        )
        .unwrap()],
    )
    .unwrap()
}

fn stable_store() -> worth_store_physical_format::store_namespace::StableStoreIdentity {
    worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord::new(
        worth_store_physical_format::store_namespace::StoreNamespaceVersion::CURRENT,
        worth_store_physical_format::store_namespace::ProposedStoreIdentity::from_nonzero_bytes(
            [7; 16],
        )
        .unwrap(),
    )
    .published_identity()
}
