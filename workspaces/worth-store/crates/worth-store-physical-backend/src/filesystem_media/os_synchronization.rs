//! The single OS flush call behind every file and directory synchronization
//! media operation.
//!
//! Under the test-only `volatile-sync-for-tests` feature (debug builds only)
//! the flush itself is skipped while everything around it still runs: the
//! boundary attempt, fault interposition, counters, ordering, and mutation
//! guards. Test crashes are process kills or injected faults, never power
//! loss, and a process kill never loses OS-cached writes, so a real flush adds
//! no behavior any test can observe. Production builds never enable the
//! feature; release builds ignore it even under feature unification.

pub(super) trait OsSynchronization {
    fn flush_state(&self) -> std::io::Result<()>;
    fn flush_data(&self) -> std::io::Result<()>;
}

impl OsSynchronization for std::fs::File {
    fn flush_state(&self) -> std::io::Result<()> {
        self.sync_all()
    }

    fn flush_data(&self) -> std::io::Result<()> {
        self.sync_data()
    }
}

impl OsSynchronization for cap_std::fs::File {
    fn flush_state(&self) -> std::io::Result<()> {
        self.sync_all()
    }

    fn flush_data(&self) -> std::io::Result<()> {
        self.sync_data()
    }
}

/// Flushes file state (data and metadata), or a directory sync handle.
pub(super) fn synchronize_state<F: OsSynchronization>(file: &F) -> std::io::Result<()> {
    if VOLATILE {
        return Ok(());
    }
    file.flush_state()
}

/// Flushes file data.
pub(super) fn synchronize_data<F: OsSynchronization>(file: &F) -> std::io::Result<()> {
    if VOLATILE {
        return Ok(());
    }
    file.flush_data()
}

const VOLATILE: bool = cfg!(all(feature = "volatile-sync-for-tests", debug_assertions));
