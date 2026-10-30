pub(crate) use std::sync::OnceLock;
pub(crate) use {std::path::Path, std::path::PathBuf};

pub(crate) use {
    worth_store_test_support::IndependentExecutable,
    worth_store_test_support::FRESH_PROCESS_RECOVERY as RECOVERY,
    worth_store_test_support::OFFLINE_RECOVERY_OBSERVER as OBSERVER,
};

// The writer is killed, never power-cut, so it skips the OS flush exactly as
// the in-process suites do.
pub(crate) const WRITER: IndependentExecutable =
    worth_store_test_support::IndependentExecutable::new("worth-store", "physical_store_c8_writer")
        .with_features("volatile-sync-for-tests");

pub(crate) static PHASE_EIGHT_BINARIES: OnceLock<PhaseEightProcessBinaries> =
    std::sync::OnceLock::new();

pub(crate) struct PhaseEightProcessBinaries {
    pub(crate) writer: ProvidedProcessBinary,
    pub(crate) observer: ProvidedProcessBinary,
    pub(crate) recovery: ProvidedProcessBinary,
}

pub(crate) struct ProvidedProcessBinary {
    pub(crate) path: PathBuf,
}

impl ProvidedProcessBinary {
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl PhaseEightProcessBinaries {
    pub(crate) fn writer(&self) -> &ProvidedProcessBinary {
        &self.writer
    }

    pub(crate) fn observer(&self) -> &ProvidedProcessBinary {
        &self.observer
    }

    pub(crate) fn recovery(&self) -> &ProvidedProcessBinary {
        &self.recovery
    }
}

pub(crate) fn phase_eight_process_binaries() -> &'static PhaseEightProcessBinaries {
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

pub(crate) fn provided(executable: IndependentExecutable) -> ProvidedProcessBinary {
    let path = executable
        .path()
        .expect("runner must prebuild the independent executable")
        .canonicalize()
        .unwrap_or_else(|error| panic!("Phase 8 binary {executable:?} is unavailable: {error}"));
    assert!(
        path.is_file(),
        "Phase 8 binary {executable:?} is not a file"
    );
    ProvidedProcessBinary { path }
}
