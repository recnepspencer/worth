#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackgroundPressureKind {
    CompactionRewrite,
    CheckpointFlush,
    ScrubScan,
    ReplicationPrepRead,
    BlobIngestPressure,
    BlobMigrationPressure,
    BlobReclaimPressure,
    BackupPrepRead,
    RepairScan,
    VerificationPressure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackgroundPressureDeclaration {
    kind: BackgroundPressureKind,
    queue_slots: u64,
    bytes: u64,
    flush_permits: u64,
    sync_debt_units: u64,
    read_ahead_pages: u64,
    write_back_pages: u64,
    dirty_pages: u64,
    worker_permits: u64,
    cache_residency_frames: u64,
    reclaim_permits: u64,
}

impl BackgroundPressureDeclaration {
    pub const fn compaction_rewrite() -> Self {
        Self::new(BackgroundPressureKind::CompactionRewrite)
            .with_bytes(4096)
            .with_write_back_pages(1)
            .with_dirty_pages(1)
    }

    pub const fn checkpoint_flush() -> Self {
        Self::new(BackgroundPressureKind::CheckpointFlush)
            .with_flush_permits(1)
            .with_sync_debt_units(1)
            .with_write_back_pages(1)
    }

    pub const fn scrub_scan() -> Self {
        Self::new(BackgroundPressureKind::ScrubScan)
            .with_bytes(4096)
            .with_read_ahead_pages(1)
    }

    pub const fn replication_prep_read(read_ahead_pages: u64) -> Self {
        Self::new(BackgroundPressureKind::ReplicationPrepRead)
            .with_read_ahead_pages(read_ahead_pages)
    }

    pub const fn blob_ingest_pressure(bytes: u64) -> Self {
        Self::new(BackgroundPressureKind::BlobIngestPressure).with_bytes(bytes)
    }

    pub const fn blob_migration_pressure(bytes: u64) -> Self {
        Self::new(BackgroundPressureKind::BlobMigrationPressure)
            .with_bytes(bytes)
            .with_reclaim_permits(1)
    }

    pub const fn blob_reclaim_pressure(bytes: u64) -> Self {
        Self::new(BackgroundPressureKind::BlobReclaimPressure)
            .with_bytes(bytes)
            .with_reclaim_permits(1)
    }

    pub const fn backup_prep_read(bytes: u64, read_ahead_pages: u64) -> Self {
        Self::new(BackgroundPressureKind::BackupPrepRead)
            .with_bytes(bytes)
            .with_read_ahead_pages(read_ahead_pages)
    }

    pub const fn repair_scan(read_ahead_pages: u64) -> Self {
        Self::new(BackgroundPressureKind::RepairScan).with_read_ahead_pages(read_ahead_pages)
    }

    pub const fn verification_pressure(read_ahead_pages: u64) -> Self {
        Self::new(BackgroundPressureKind::VerificationPressure)
            .with_read_ahead_pages(read_ahead_pages)
    }

    pub const fn kind(self) -> BackgroundPressureKind {
        self.kind
    }

    pub const fn queue_slots(self) -> u64 {
        self.queue_slots
    }

    pub const fn bytes(self) -> u64 {
        self.bytes
    }

    pub const fn flush_permits(self) -> u64 {
        self.flush_permits
    }

    pub const fn sync_debt_units(self) -> u64 {
        self.sync_debt_units
    }

    pub const fn read_ahead_pages(self) -> u64 {
        self.read_ahead_pages
    }

    pub const fn write_back_pages(self) -> u64 {
        self.write_back_pages
    }

    pub const fn dirty_pages(self) -> u64 {
        self.dirty_pages
    }

    pub const fn worker_permits(self) -> u64 {
        self.worker_permits
    }

    pub const fn cache_residency_frames(self) -> u64 {
        self.cache_residency_frames
    }

    pub const fn reclaim_permits(self) -> u64 {
        self.reclaim_permits
    }

    const fn new(kind: BackgroundPressureKind) -> Self {
        Self {
            kind,
            queue_slots: 1,
            bytes: 0,
            flush_permits: 0,
            sync_debt_units: 0,
            read_ahead_pages: 0,
            write_back_pages: 0,
            dirty_pages: 0,
            worker_permits: 1,
            cache_residency_frames: 0,
            reclaim_permits: 0,
        }
    }

    const fn with_bytes(mut self, bytes: u64) -> Self {
        self.bytes = bytes;
        self
    }

    const fn with_flush_permits(mut self, flush_permits: u64) -> Self {
        self.flush_permits = flush_permits;
        self
    }

    const fn with_sync_debt_units(mut self, sync_debt_units: u64) -> Self {
        self.sync_debt_units = sync_debt_units;
        self
    }

    const fn with_read_ahead_pages(mut self, read_ahead_pages: u64) -> Self {
        self.read_ahead_pages = read_ahead_pages;
        self
    }

    const fn with_write_back_pages(mut self, write_back_pages: u64) -> Self {
        self.write_back_pages = write_back_pages;
        self
    }

    const fn with_dirty_pages(mut self, dirty_pages: u64) -> Self {
        self.dirty_pages = dirty_pages;
        self
    }

    const fn with_reclaim_permits(mut self, reclaim_permits: u64) -> Self {
        self.reclaim_permits = reclaim_permits;
        self
    }
}
