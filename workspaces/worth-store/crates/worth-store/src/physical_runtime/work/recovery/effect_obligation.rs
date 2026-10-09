use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
};

use worth_store_physical_backend::{
    ArtifactTreeDirectory, ArtifactTreeFile, QualifiedFilesystemMedia,
};
use worth_store_physical_format::physical_work_obligation::{
    encode_physical_work_obligation_v6, PhysicalWorkObligationV6,
    PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES,
};

use super::{
    super::{PhysicalWorkIdentity, PhysicalWorkOperationFamily},
    effect_classification::{journaling, PhysicalEffectJournaling},
    format_mapping::{operation_to_format, target_to_format},
    journal_counters::PhysicalRecoveryJournalCounters,
    PhysicalWorkRecoveryTarget,
};

/// Durable recovery obligations of in-flight physical effects.
pub(in crate::physical_runtime) struct PhysicalEffectJournal {
    directory: ArtifactTreeDirectory,
    initialized: Mutex<bool>,
    records_written: AtomicU64,
    directory_barriers: AtomicU64,
}

/// One effect's recovery obligation between its dispatch and its settlement.
pub(in crate::physical_runtime) struct PreparedPhysicalEffect {
    artifact: ArtifactTreeFile,
    record: PreparedObligationRecord,
}

enum PreparedObligationRecord {
    Durable,
    Deferred([u8; PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES]),
}

impl PhysicalEffectJournal {
    pub(in crate::physical_runtime) fn new(media: &QualifiedFilesystemMedia) -> Self {
        let directory = journal_directory();
        let initialized = media
            .artifact_tree()
            .directory_exists(&directory)
            .unwrap_or(false);
        Self {
            directory,
            initialized: Mutex::new(initialized),
            records_written: AtomicU64::new(0),
            directory_barriers: AtomicU64::new(0),
        }
    }

    pub(in crate::physical_runtime) fn prepare(
        &self,
        media: &QualifiedFilesystemMedia,
        identity: PhysicalWorkIdentity,
        operation: PhysicalWorkOperationFamily,
        target: PhysicalWorkRecoveryTarget,
        payload_digest: Option<[u8; 32]>,
    ) -> Result<PreparedPhysicalEffect, ()> {
        let artifact = self
            .directory
            .file(&format!(
                "effect-{:016x}-{:016x}-{:016x}.pending",
                identity.runtime().get(),
                identity.generation().lifecycle().get(),
                identity.operation().get(),
            ))
            .map_err(|_| ())?;
        let record = encode_record(identity, operation, target, payload_digest);
        let record = match journaling(operation, target) {
            PhysicalEffectJournaling::OnlyIfRetained => PreparedObligationRecord::Deferred(record),
            PhysicalEffectJournaling::BeforeEffect => {
                self.persist(media, &artifact, &record)?;
                PreparedObligationRecord::Durable
            }
        };
        Ok(PreparedPhysicalEffect { artifact, record })
    }

    /// Settles the obligation of an effect that completed: a durable record
    /// is removed and its removal synchronized.
    pub(in crate::physical_runtime) fn finish(
        &self,
        media: &QualifiedFilesystemMedia,
        prepared: PreparedPhysicalEffect,
    ) -> Result<(), ()> {
        let PreparedObligationRecord::Durable = prepared.record else {
            return Ok(());
        };
        self.directory_barriers.fetch_add(1, Ordering::AcqRel);
        media
            .artifact_tree()
            .remove_file_durably(&prepared.artifact)
            .map_err(|_| ())
    }

    /// Keeps the obligation of an effect whose outcome is retained for
    /// inspection, writing the deferred record of a retained flush. A failed
    /// write leaves only this runtime's fence; the caller reports it as
    /// retained without a record.
    pub(in crate::physical_runtime) fn retain(
        &self,
        media: &QualifiedFilesystemMedia,
        prepared: PreparedPhysicalEffect,
    ) -> Result<(), ()> {
        let PreparedObligationRecord::Deferred(record) = prepared.record else {
            return Ok(());
        };
        self.persist(media, &prepared.artifact, &record)
    }

    pub(in crate::physical_runtime) fn counters(&self) -> PhysicalRecoveryJournalCounters {
        PhysicalRecoveryJournalCounters::new(
            self.records_written.load(Ordering::Acquire),
            self.directory_barriers.load(Ordering::Acquire),
        )
    }

    fn persist(
        &self,
        media: &QualifiedFilesystemMedia,
        artifact: &ArtifactTreeFile,
        record: &[u8; PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES],
    ) -> Result<(), ()> {
        self.ensure_directory(media)?;
        media
            .artifact_tree()
            .write_new_obligation_record(artifact, record)
            .map_err(|_| ())?;
        self.records_written.fetch_add(1, Ordering::AcqRel);
        self.directory_barriers.fetch_add(1, Ordering::AcqRel);
        media
            .artifact_tree()
            .synchronize_directory(&self.directory)
            .map_err(|_| ())
    }

    fn ensure_directory(&self, media: &QualifiedFilesystemMedia) -> Result<(), ()> {
        let mut initialized = self
            .initialized
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if *initialized {
            return Ok(());
        }
        let tree = media.artifact_tree();
        if !tree.directory_exists(&self.directory).map_err(|_| ())? {
            tree.create_directory(&self.directory).map_err(|_| ())?;
            self.directory_barriers.fetch_add(1, Ordering::AcqRel);
            tree.synchronize_directory(&ArtifactTreeDirectory::families())
                .map_err(|_| ())?;
        }
        *initialized = true;
        Ok(())
    }
}

pub(super) fn journal_directory() -> ArtifactTreeDirectory {
    ArtifactTreeDirectory::families()
        .child("physical-work")
        .expect("portable physical-work recovery path")
}

pub(super) fn encode_record(
    identity: PhysicalWorkIdentity,
    operation: PhysicalWorkOperationFamily,
    target: PhysicalWorkRecoveryTarget,
    payload_digest: Option<[u8; 32]>,
) -> [u8; PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES] {
    let value = PhysicalWorkObligationV6::new(
        identity.store().bytes(),
        identity.runtime().get(),
        identity.generation().lifecycle().get(),
        identity.operation().get(),
        operation_to_format(operation),
        target_to_format(target),
        payload_digest,
    )
    .expect("Store physical-work identity and target satisfy v6 format");
    encode_physical_work_obligation_v6(value)
}
