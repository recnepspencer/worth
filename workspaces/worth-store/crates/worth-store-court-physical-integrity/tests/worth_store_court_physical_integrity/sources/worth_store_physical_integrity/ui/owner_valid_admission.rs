use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_format::{
    DurableInlineRecordPlacement, ExtentChunkCoordinate, PhysicalCheckpointIdentity,
    WalSegmentIdentity,
};
use worth_store_physical_integrity::{
    CheckpointFooterValidationBasis, IntegrityValidatedCheckpointBinding,
    IntegrityValidatedCheckpointBindingCompaction, IntegrityValidatedCheckpointDirtyBasis,
    IntegrityValidatedCheckpointFooter, IntegrityValidatedCheckpointStreamHeader,
    IntegrityValidatedCurrentRootSelector, IntegrityValidatedPhysicalWorkObligation,
    IntegrityValidatedPreviousRootSelector, IntegrityValidatedRootManifest, PhysicalArtifactScope,
    IntegrityValidatedExtentChunkFrame, IntegrityValidatedExtentManifest,
    IntegrityValidatedPageFrame, IntegrityValidatedWalFrame, PhysicalByteRange,
    PhysicalIntegrityValidationRecord, UntrustedPhysicalArtifact,
};

fn main() {
    let _input = UntrustedPhysicalArtifact::from_bounded_bytes(b"untrusted");
}

fn valid_checkpoint_header<'media>(
    validated: IntegrityValidatedCheckpointStreamHeader<'media>,
    input: UntrustedPhysicalArtifact<'media>,
) -> PhysicalIntegrityValidationRecord {
    let scope = validated.scope();
    let _identity = validated.checkpoint_identity();
    let _source = validated.source();
    let _same_incarnation = validated.matches_input(input);
    let record = validated.into_validation_record();
    let _same_scope = record.matches_scope(scope);
    record
}

fn valid_checkpoint_dirty<'media>(
    validated: IntegrityValidatedCheckpointDirtyBasis<'media>,
    input: UntrustedPhysicalArtifact<'media>,
) -> PhysicalIntegrityValidationRecord {
    let scope = validated.scope();
    let _basis = validated.basis();
    let _same_incarnation = validated.matches_input(input);
    let record = validated.into_validation_record();
    let _same_scope = record.matches_scope(scope);
    record
}

fn valid_checkpoint_compaction<'media>(
    validated: IntegrityValidatedCheckpointBindingCompaction<'media>,
    input: UntrustedPhysicalArtifact<'media>,
) -> PhysicalIntegrityValidationRecord {
    let scope = validated.scope();
    let _generation = validated.generation();
    let _wal_cutoff = validated.wal_cutoff_lsn_exclusive();
    let _same_incarnation = validated.matches_input(input);
    let record = validated.into_validation_record();
    let _same_scope = record.matches_scope(scope);
    record
}

fn valid_checkpoint_binding<'media>(
    validated: IntegrityValidatedCheckpointBinding<'media>,
    input: UntrustedPhysicalArtifact<'media>,
    checkpoint: PhysicalCheckpointIdentity,
) -> PhysicalIntegrityValidationRecord {
    let scope = validated.scope();
    let _payload_bytes = validated.payload_bytes();
    let _encoded_bytes = validated.encoded_bytes();
    let _same_incarnation = validated.matches_input(input);
    let projection = validated.project_payload(input, checkpoint).unwrap();
    let _payload_range = projection.payload_range();
    let record = validated.into_validation_record();
    let _same_scope = record.matches_scope(scope);
    record
}

fn valid_checkpoint_footer<'media>(
    validated: IntegrityValidatedCheckpointFooter<'media>,
    input: UntrustedPhysicalArtifact<'media>,
) -> PhysicalIntegrityValidationRecord {
    let scope = validated.scope();
    let _footer = validated.footer();
    let _same_incarnation = validated.matches_input(input);
    let record = validated.into_validation_record();
    let _same_scope = record.matches_scope(scope);
    record
}

fn valid_checkpoint_progression<'records, 'media>(
    header: &'records IntegrityValidatedCheckpointStreamHeader<'media>,
    dirty: &'records [IntegrityValidatedCheckpointDirtyBasis<'media>],
    compaction: &'records IntegrityValidatedCheckpointBindingCompaction<'media>,
    bindings: &'records [IntegrityValidatedCheckpointBinding<'media>],
    range: PhysicalByteRange,
) {
    let identity = header.checkpoint_identity();
    let _later_scope = PhysicalArtifactScope::checkpoint_dirty_basis(identity, range);
    let _footer_basis = CheckpointFooterValidationBasis::new(header, dirty, compaction, bindings);
}

fn selector_scope(store: StableStoreIdentity) -> PhysicalArtifactScope {
    PhysicalArtifactScope::current_root_selector(
        store,
        worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
        PhysicalByteRange::new(0, 107).unwrap(),
    )
}

fn valid_current<'media>(
    validated: IntegrityValidatedCurrentRootSelector<'media>,
    input: UntrustedPhysicalArtifact<'media>,
) -> PhysicalIntegrityValidationRecord {
    let scope = validated.scope();
    let _same_incarnation = validated.matches_input(input);
    let record = validated.into_validation_record();
    let _same_scope = record.matches_scope(scope);
    record
}

fn valid_previous<'media>(
    validated: IntegrityValidatedPreviousRootSelector<'media>,
    input: UntrustedPhysicalArtifact<'media>,
) -> PhysicalIntegrityValidationRecord {
    let scope = validated.scope();
    let _same_incarnation = validated.matches_input(input);
    let record = validated.into_validation_record();
    let _same_scope = record.matches_scope(scope);
    record
}

fn valid_manifest<'media>(
    validated: IntegrityValidatedRootManifest<'media>,
    input: UntrustedPhysicalArtifact<'media>,
) -> PhysicalIntegrityValidationRecord {
    let scope = validated.scope();
    let _same_incarnation = validated.matches_input(input);
    let record = validated.into_validation_record();
    let _same_scope = record.matches_scope(scope);
    record
}

fn valid_physical_work<'media>(
    validated: IntegrityValidatedPhysicalWorkObligation<'media>,
    input: UntrustedPhysicalArtifact<'media>,
) -> PhysicalIntegrityValidationRecord {
    let scope = validated.scope();
    let _identity = validated.identity();
    let _operation = validated.operation_code();
    let _target = validated.target();
    let _payload_digest = validated.payload_digest();
    let _same_incarnation = validated.matches_input(input);
    let record = validated.into_validation_record();
    let _same_scope = record.matches_scope(scope);
    record
}

fn valid_page<'media>(
    validated: IntegrityValidatedPageFrame<'media>,
    input: UntrustedPhysicalArtifact<'media>,
    placement: DurableInlineRecordPlacement,
) -> PhysicalIntegrityValidationRecord {
    let scope = validated.scope();
    let _same_incarnation = validated.matches_input(input);
    let projection = validated.project_record(input, placement).unwrap();
    let _payload_range = projection.payload_range();
    let record = validated.into_validation_record();
    let _same_scope = record.matches_scope(scope);
    record
}

fn valid_wal<'media>(
    validated: IntegrityValidatedWalFrame<'media>,
    input: UntrustedPhysicalArtifact<'media>,
    segment: WalSegmentIdentity,
) -> PhysicalIntegrityValidationRecord {
    let scope = validated.scope();
    let _identity = validated.segment_identity();
    let _lsn_after_validation = (validated.lsn_start(), validated.lsn_end());
    let _same_incarnation = validated.matches_input(input);
    let projection = validated.project_payload(input, segment).unwrap();
    let _payload_range = projection.payload_range();
    let record = validated.into_validation_record();
    let _same_scope = record.matches_scope(scope);
    record
}

fn valid_extent_manifest<'media>(
    validated: IntegrityValidatedExtentManifest<'media>,
    input: UntrustedPhysicalArtifact<'media>,
) -> PhysicalIntegrityValidationRecord {
    let scope = validated.scope();
    let _same_incarnation = validated.matches_input(input);
    let record = validated.into_validation_record();
    let _same_scope = record.matches_scope(scope);
    record
}

fn valid_extent_chunk<'media>(
    validated: IntegrityValidatedExtentChunkFrame<'media>,
    input: UntrustedPhysicalArtifact<'media>,
    coordinate: ExtentChunkCoordinate,
) -> PhysicalIntegrityValidationRecord {
    let scope = validated.scope();
    let _same_incarnation = validated.matches_input(input);
    let projection = validated.project_chunk(input, coordinate).unwrap();
    let _payload_range = projection.payload_range();
    let record = validated.into_validation_record();
    let _same_scope = record.matches_scope(scope);
    record
}
