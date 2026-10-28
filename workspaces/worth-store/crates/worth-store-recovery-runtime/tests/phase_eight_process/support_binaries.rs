use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use worth_store_test_support::{
    IndependentExecutable, FRESH_PROCESS_RECOVERY as RECOVERY,
    OFFLINE_RECOVERY_OBSERVER as OBSERVER,
};

// The writer is killed, never power-cut, so it skips the OS flush exactly as
// the in-process suites do.
const WRITER: IndependentExecutable =
    IndependentExecutable::new("worth-store", "physical_store_c8_writer")
        .with_features("volatile-sync-for-tests");

static PHASE_EIGHT_BINARIES: OnceLock<PhaseEightProcessBinaries> = OnceLock::new();

pub(super) struct PhaseEightProcessBinaries {
    writer: ProvidedProcessBinary,
    observer: ProvidedProcessBinary,
    recovery: ProvidedProcessBinary,
}

pub(super) struct ProvidedProcessBinary {
    path: PathBuf,
}

impl ProvidedProcessBinary {
    pub(super) fn path(&self) -> &Path {
        &self.path
    }
}

impl PhaseEightProcessBinaries {
    pub(super) fn writer(&self) -> &ProvidedProcessBinary {
        &self.writer
    }

    pub(super) fn observer(&self) -> &ProvidedProcessBinary {
        &self.observer
    }

    pub(super) fn recovery(&self) -> &ProvidedProcessBinary {
        &self.recovery
    }
}

pub(super) fn phase_eight_process_binaries() -> &'static PhaseEightProcessBinaries {
    PHASE_EIGHT_BINARIES.get_or_init(|| {
        let writer = provided(WRITER);
        let observer = provided(OBSERVER);
        let recovery = provided(RECOVERY);
        assert_ne!(writer.path, observer.path, "writer and observer alias");
        assert_ne!(writer.path, recovery.path, "writer and recovery alias");
        assert_ne!(observer.path, recovery.path, "observer and recovery alias");
        PhaseEightProcessBinaries {
            writer,
            observer,
            recovery,
        }
    })
}

fn provided(executable: IndependentExecutable) -> ProvidedProcessBinary {
    let path = executable
        .path()
        .canonicalize()
        .unwrap_or_else(|error| panic!("Phase 8 binary {executable:?} is unavailable: {error}"));
    assert!(
        path.is_file(),
        "Phase 8 binary {executable:?} is not a file"
    );
    ProvidedProcessBinary { path }
}
