use worth_store_physical_backend::{
    ArtifactTreeDirectory, ArtifactTreeFile, ArtifactTreeMedia, QualifiedFilesystemMedia,
};
use worth_store_physical_format::{
    CheckpointBindingCompactionHeader, CheckpointStreamFooter, PhysicalCheckpointIdentity,
    CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES, CHECKPOINT_BINDING_RECORD_PREFIX_BYTES,
    CHECKPOINT_DIRTY_FRAME_RECORD_BYTES, CHECKPOINT_STREAM_FOOTER_RECORD_BYTES,
    CHECKPOINT_STREAM_HEADER_RECORD_BYTES,
};

use super::integrity_admission::{
    admit_binding_compaction, admit_footer_envelope, admit_stream_header, physical_range,
};
use super::{
    PhysicalBindingCompactionReopenCounters, PhysicalBindingCompactionReopenFailure,
    ReopenedPhysicalBindingCompaction,
};

pub(in crate::physical_runtime) struct NamespaceDurablePhysicalBindingCompactionReopen {
    artifact: ArtifactTreeFile,
    checkpoint: PhysicalCheckpointIdentity,
    header: CheckpointBindingCompactionHeader,
    footer: CheckpointStreamFooter,
    artifact_bytes: u64,
    compaction_offset: u64,
    records_offset: u64,
    footer_offset: u64,
}

pub(in crate::physical_runtime) struct PhysicalBindingCompactionRebuildBasis<'reopen> {
    artifact: &'reopen ArtifactTreeFile,
    checkpoint: PhysicalCheckpointIdentity,
    artifact_bytes: u64,
    compaction_offset: u64,
    records_offset: u64,
    footer_offset: u64,
    expected_records: u64,
    expected_encoded_bytes: u64,
    expected_digest: [u8; 32],
}

pub(in crate::physical_runtime) fn reopen_binding_compaction(
    media: &QualifiedFilesystemMedia,
) -> Result<ReopenedPhysicalBindingCompaction, PhysicalBindingCompactionReopenFailure> {
    let tree = media.artifact_tree();
    let artifact = checkpoint_artifact();
    if !tree
        .file_exists(&artifact)
        .map_err(PhysicalBindingCompactionReopenFailure::Media)?
    {
        return Ok(ReopenedPhysicalBindingCompaction::GenerationZero);
    }
    let artifact_bytes = tree
        .file_length(&artifact)
        .map_err(PhysicalBindingCompactionReopenFailure::Media)?;
    let reopened = NamespaceDurablePhysicalBindingCompactionReopen::read(
        &tree,
        artifact,
        artifact_bytes,
        media.store_identity(),
    )?;
    Ok(ReopenedPhysicalBindingCompaction::NamespaceDurable(
        reopened,
    ))
}

impl NamespaceDurablePhysicalBindingCompactionReopen {
    pub(in crate::physical_runtime) const fn generation(&self) -> u64 {
        self.header.generation()
    }

    pub(in crate::physical_runtime) const fn wal_cutoff_lsn_exclusive(&self) -> u64 {
        self.header.wal_cutoff_lsn_exclusive()
    }

    pub(in crate::physical_runtime) fn rebuild_basis(
        &self,
    ) -> PhysicalBindingCompactionRebuildBasis<'_> {
        PhysicalBindingCompactionRebuildBasis {
            artifact: &self.artifact,
            checkpoint: self.checkpoint,
            artifact_bytes: self.artifact_bytes,
            compaction_offset: self.compaction_offset,
            records_offset: self.records_offset,
            footer_offset: self.footer_offset,
            expected_records: self.footer.binding_record_count(),
            expected_encoded_bytes: self.footer.binding_record_bytes(),
            expected_digest: self.footer.binding_records_digest(),
        }
    }

    fn read(
        tree: &ArtifactTreeMedia<'_>,
        artifact: ArtifactTreeFile,
        artifact_bytes: u64,
        store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    ) -> Result<Self, PhysicalBindingCompactionReopenFailure> {
        if artifact_bytes < fixed_read_bytes()? {
            return Err(PhysicalBindingCompactionReopenFailure::ArtifactTooShort);
        }
        let header_record =
            read_fixed::<CHECKPOINT_STREAM_HEADER_RECORD_BYTES>(tree, &artifact, 0)?;
        let header_range = physical_range(0, CHECKPOINT_STREAM_HEADER_RECORD_BYTES as u64)?;
        let source = admit_stream_header(&header_record, store, header_range)?;
        let footer_offset = artifact_bytes
            .checked_sub(CHECKPOINT_STREAM_FOOTER_RECORD_BYTES as u64)
            .ok_or(PhysicalBindingCompactionReopenFailure::ArtifactTooShort)?;
        let footer_record =
            read_fixed::<CHECKPOINT_STREAM_FOOTER_RECORD_BYTES>(tree, &artifact, footer_offset)?;
        let footer_range =
            physical_range(footer_offset, CHECKPOINT_STREAM_FOOTER_RECORD_BYTES as u64)?;
        let footer = admit_footer_envelope(&footer_record, source.identity(), footer_range)?;
        let compaction_offset = footer.binding_compaction_header_offset();
        let records_offset = compaction_offset
            .checked_add(CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES as u64)
            .ok_or(PhysicalBindingCompactionReopenFailure::CounterOverflow)?;
        if compaction_offset < CHECKPOINT_STREAM_HEADER_RECORD_BYTES as u64
            || records_offset > footer_offset
            || footer_offset - records_offset != footer.binding_record_bytes()
        {
            return Err(PhysicalBindingCompactionReopenFailure::ArtifactLayoutMismatch);
        }
        let expected_compaction_offset = (CHECKPOINT_STREAM_HEADER_RECORD_BYTES as u64)
            .checked_add(
                footer
                    .dirty_record_count()
                    .checked_mul(CHECKPOINT_DIRTY_FRAME_RECORD_BYTES as u64)
                    .ok_or(PhysicalBindingCompactionReopenFailure::CounterOverflow)?,
            )
            .ok_or(PhysicalBindingCompactionReopenFailure::CounterOverflow)?;
        if compaction_offset != expected_compaction_offset {
            return Err(PhysicalBindingCompactionReopenFailure::ArtifactLayoutMismatch);
        }
        let compaction_record = read_fixed::<CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES>(
            tree,
            &artifact,
            compaction_offset,
        )?;
        let compaction_range = physical_range(
            compaction_offset,
            CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES as u64,
        )?;
        let header =
            admit_binding_compaction(&compaction_record, source.identity(), compaction_range)?;
        if footer.binding_compaction_generation() != header.generation()
            || footer.binding_wal_cutoff_lsn_exclusive() != header.wal_cutoff_lsn_exclusive()
        {
            return Err(PhysicalBindingCompactionReopenFailure::ArtifactLayoutMismatch);
        }
        let minimum_record_bytes = (CHECKPOINT_BINDING_RECORD_PREFIX_BYTES + 1 + 4) as u64;
        if footer.binding_record_count() > footer.binding_record_bytes() / minimum_record_bytes {
            return Err(PhysicalBindingCompactionReopenFailure::ArtifactLayoutMismatch);
        }
        Ok(Self {
            artifact,
            checkpoint: source.identity(),
            header,
            footer,
            artifact_bytes,
            compaction_offset,
            records_offset,
            footer_offset,
        })
    }
}

impl PhysicalBindingCompactionRebuildBasis<'_> {
    pub(in crate::physical_runtime) fn artifact(&self) -> &ArtifactTreeFile {
        self.artifact
    }

    pub(in crate::physical_runtime) const fn checkpoint(&self) -> PhysicalCheckpointIdentity {
        self.checkpoint
    }

    pub(in crate::physical_runtime) const fn records_offset(&self) -> u64 {
        self.records_offset
    }

    pub(in crate::physical_runtime) const fn footer_offset(&self) -> u64 {
        self.footer_offset
    }

    pub(in crate::physical_runtime) const fn expected_records(&self) -> u64 {
        self.expected_records
    }

    pub(in crate::physical_runtime) const fn expected_encoded_bytes(&self) -> u64 {
        self.expected_encoded_bytes
    }

    pub(in crate::physical_runtime) const fn expected_digest(&self) -> [u8; 32] {
        self.expected_digest
    }

    pub(in crate::physical_runtime) fn completed_counters(
        &self,
        records_read: u64,
    ) -> Result<PhysicalBindingCompactionReopenCounters, PhysicalBindingCompactionReopenFailure>
    {
        let checkpoint_bytes_read = fixed_read_bytes()?
            .checked_add(self.expected_encoded_bytes)
            .ok_or(PhysicalBindingCompactionReopenFailure::CounterOverflow)?;
        let integrity_admissions = records_read
            .checked_add(3)
            .ok_or(PhysicalBindingCompactionReopenFailure::CounterOverflow)?;
        Ok(PhysicalBindingCompactionReopenCounters {
            checkpoint_artifact_bytes: self.artifact_bytes,
            checkpoint_bytes_read,
            dirty_body_bytes_skipped: self.compaction_offset
                - CHECKPOINT_STREAM_HEADER_RECORD_BYTES as u64,
            binding_records_read: records_read,
            integrity_admissions,
        })
    }
}

fn read_fixed<const BYTES: usize>(
    tree: &ArtifactTreeMedia<'_>,
    artifact: &ArtifactTreeFile,
    offset: u64,
) -> Result<[u8; BYTES], PhysicalBindingCompactionReopenFailure> {
    let mut bytes = [0; BYTES];
    tree.read_exact_at(artifact, offset, &mut bytes)
        .map_err(PhysicalBindingCompactionReopenFailure::Media)?;
    Ok(bytes)
}

fn fixed_read_bytes() -> Result<u64, PhysicalBindingCompactionReopenFailure> {
    (CHECKPOINT_STREAM_HEADER_RECORD_BYTES as u64)
        .checked_add(CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES as u64)
        .and_then(|bytes| bytes.checked_add(CHECKPOINT_STREAM_FOOTER_RECORD_BYTES as u64))
        .ok_or(PhysicalBindingCompactionReopenFailure::CounterOverflow)
}

fn checkpoint_artifact() -> ArtifactTreeFile {
    ArtifactTreeDirectory::families()
        .file("checkpoint.current")
        .expect("the canonical checkpoint publication name is portable")
}
