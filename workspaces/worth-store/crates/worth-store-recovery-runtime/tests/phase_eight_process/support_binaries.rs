use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const WRITER_ENV: &str = "WORTH_STORE_PHASE8_WRITER";
const OBSERVER_ENV: &str = "WORTH_STORE_PHASE8_OBSERVER";
const RECOVERY_ENV: &str = "WORTH_STORE_PHASE8_RECOVERY";

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
        let writer = provided(WRITER_ENV);
        let observer = provided(OBSERVER_ENV);
        let recovery = provided(RECOVERY_ENV);
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

fn provided(name: &str) -> ProvidedProcessBinary {
    let encoded = std::env::var_os(name)
        .unwrap_or_else(|| panic!("Phase 8 suite owner omitted required environment `{name}`"));
    let path = PathBuf::from(encoded)
        .canonicalize()
        .unwrap_or_else(|error| panic!("Phase 8 suite binary `{name}` is unavailable: {error}"));
    assert!(
        path.is_file(),
        "Phase 8 suite binary `{name}` is not a file"
    );
    ProvidedProcessBinary { path }
}
