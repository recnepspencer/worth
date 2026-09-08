use worth_store_contracts::{BackgroundPressureDeclaration, BackgroundPressureKind};

use crate::{
    BandwidthToken, CacheResidencyHint, DirtyPageBudget, FlushPermit,
    IoSchedulerBackendCapabilityRequirement, QueueSlot, ReadAheadWindow, ReclaimPermit, SyncDebt,
    WorkerPermit, WriteBackWindow,
};

use super::{BackgroundIoPressureClass, BackgroundResourceBudget};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackgroundIoPressureShape {
    class: BackgroundIoPressureClass,
    requested: BackgroundResourceBudget,
    backend_requirement: IoSchedulerBackendCapabilityRequirement,
    secure_scope_required: bool,
}

impl BackgroundIoPressureShape {
    pub const fn compaction_rewrite() -> Self {
        Self::new(
            BackgroundIoPressureClass::CompactionRewrite,
            IoSchedulerBackendCapabilityRequirement::DirectIo,
        )
    }

    pub const fn checkpoint_flush() -> Self {
        Self::new(
            BackgroundIoPressureClass::CheckpointFlush,
            IoSchedulerBackendCapabilityRequirement::Fsync,
        )
    }

    pub const fn filesystem_admitted_checkpoint_flush() -> Self {
        Self::new(
            BackgroundIoPressureClass::CheckpointFlush,
            IoSchedulerBackendCapabilityRequirement::FilesystemAdmittedFsync,
        )
    }

    pub const fn scrub_scan() -> Self {
        Self::new(
            BackgroundIoPressureClass::ScrubScan,
            IoSchedulerBackendCapabilityRequirement::DirectIo,
        )
    }

    /// Store-owned buffered-file inspection; does not claim direct I/O.
    pub const fn buffered_file_scrub_scan() -> Self {
        Self::new(
            BackgroundIoPressureClass::ScrubScan,
            IoSchedulerBackendCapabilityRequirement::BufferedFile,
        )
    }

    pub const fn replication_prep_read() -> Self {
        Self::new(
            BackgroundIoPressureClass::ReplicationPrepRead,
            IoSchedulerBackendCapabilityRequirement::DirectIo,
        )
    }

    pub const fn blob_ingest_pressure() -> Self {
        Self::new(
            BackgroundIoPressureClass::IngestPressure,
            IoSchedulerBackendCapabilityRequirement::AsyncIo,
        )
    }

    pub const fn blob_migration_pressure() -> Self {
        Self::new(
            BackgroundIoPressureClass::MigrationPressure,
            IoSchedulerBackendCapabilityRequirement::DirectIo,
        )
    }

    pub const fn backup_prep_read() -> Self {
        Self::new_secure_scope_required(
            BackgroundIoPressureClass::BackupPrepRead,
            IoSchedulerBackendCapabilityRequirement::DirectIo,
        )
    }

    pub const fn repair_scan() -> Self {
        Self::new_secure_scope_required(
            BackgroundIoPressureClass::RepairScan,
            IoSchedulerBackendCapabilityRequirement::DirectIo,
        )
    }

    pub const fn verification_pressure() -> Self {
        Self::new_secure_scope_required(
            BackgroundIoPressureClass::VerificationPressure,
            IoSchedulerBackendCapabilityRequirement::DirectIo,
        )
    }

    pub const fn requesting(mut self, requested: BackgroundResourceBudget) -> Self {
        self.requested = requested;
        self
    }

    pub fn from_background_pressure_declaration(
        declaration: BackgroundPressureDeclaration,
    ) -> Self {
        let shape = match declaration.kind() {
            BackgroundPressureKind::CompactionRewrite => Self::compaction_rewrite(),
            BackgroundPressureKind::CheckpointFlush => Self::checkpoint_flush(),
            BackgroundPressureKind::ScrubScan => Self::scrub_scan(),
            BackgroundPressureKind::ReplicationPrepRead => Self::replication_prep_read(),
            BackgroundPressureKind::BlobIngestPressure => Self::blob_ingest_pressure(),
            BackgroundPressureKind::BlobMigrationPressure => Self::blob_migration_pressure(),
            BackgroundPressureKind::BackupPrepRead => Self::backup_prep_read(),
            BackgroundPressureKind::RepairScan => Self::repair_scan(),
            BackgroundPressureKind::VerificationPressure => Self::verification_pressure(),
        };
        shape.requesting(background_budget_from_declaration(declaration))
    }

    pub const fn class(self) -> BackgroundIoPressureClass {
        self.class
    }

    pub const fn requested_budget(self) -> BackgroundResourceBudget {
        self.requested
    }

    pub const fn backend_requirement(self) -> IoSchedulerBackendCapabilityRequirement {
        self.backend_requirement
    }

    pub const fn secure_scope_required(self) -> bool {
        self.secure_scope_required
    }

    const fn new(
        class: BackgroundIoPressureClass,
        backend_requirement: IoSchedulerBackendCapabilityRequirement,
    ) -> Self {
        Self {
            class,
            requested: BackgroundResourceBudget::new(),
            backend_requirement,
            secure_scope_required: false,
        }
    }

    const fn new_secure_scope_required(
        class: BackgroundIoPressureClass,
        backend_requirement: IoSchedulerBackendCapabilityRequirement,
    ) -> Self {
        Self {
            class,
            requested: BackgroundResourceBudget::new(),
            backend_requirement,
            secure_scope_required: true,
        }
    }
}

fn background_budget_from_declaration(
    declaration: BackgroundPressureDeclaration,
) -> BackgroundResourceBudget {
    let mut budget = BackgroundResourceBudget::new();
    if declaration.queue_slots() > 0 {
        budget = budget.with_queue_slots(
            QueueSlot::new(declaration.queue_slots())
                .expect("positive declaration queue slots should lower to queue slots"),
        );
    }
    if declaration.bytes() > 0 {
        budget = budget.with_bandwidth(
            BandwidthToken::bytes(declaration.bytes())
                .expect("positive declaration bytes should lower to bandwidth tokens"),
        );
    }
    if declaration.flush_permits() > 0 {
        budget = budget.with_flush_permits(
            FlushPermit::new(declaration.flush_permits())
                .expect("positive declaration flush permits should lower to flush permits"),
        );
    }
    if declaration.sync_debt_units() > 0 {
        budget = budget.with_sync_debt(
            SyncDebt::units(declaration.sync_debt_units())
                .expect("positive declaration sync debt should lower to sync debt"),
        );
    }
    if declaration.read_ahead_pages() > 0 {
        budget = budget.with_read_ahead(
            ReadAheadWindow::pages(declaration.read_ahead_pages())
                .expect("positive declaration pages should lower to read-ahead windows"),
        );
    }
    if declaration.write_back_pages() > 0 {
        budget = budget.with_write_back(
            WriteBackWindow::pages(declaration.write_back_pages())
                .expect("positive declaration write-back pages should lower to write-back windows"),
        );
    }
    if declaration.dirty_pages() > 0 {
        budget = budget.with_dirty_pages(
            DirtyPageBudget::pages(declaration.dirty_pages())
                .expect("positive declaration dirty pages should lower to dirty page budgets"),
        );
    }
    if declaration.worker_permits() > 0 {
        budget = budget.with_worker_permits(
            WorkerPermit::new(declaration.worker_permits())
                .expect("positive declaration workers should lower to worker permits"),
        );
    }
    if declaration.cache_residency_frames() > 0 {
        budget = budget.with_cache_residency(
            CacheResidencyHint::frames(declaration.cache_residency_frames()).expect(
                "positive declaration cache residency should lower to cache residency hints",
            ),
        );
    }
    if declaration.reclaim_permits() > 0 {
        budget =
            budget
                .with_reclaim_permits(ReclaimPermit::new(declaration.reclaim_permits()).expect(
                    "positive declaration reclaim permits should lower to reclaim permits",
                ));
    }
    budget
}
