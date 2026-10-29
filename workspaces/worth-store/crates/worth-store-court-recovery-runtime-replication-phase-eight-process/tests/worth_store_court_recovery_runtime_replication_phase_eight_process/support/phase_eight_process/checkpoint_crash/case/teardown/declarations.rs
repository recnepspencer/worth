pub(crate) use {crate::phase_eight_process::checkpoint_crash::case::independent_recoveries::RecoverySet};

pub(crate) fn remove(recoveries: RecoverySet) {
    std::fs::remove_dir_all(&recoveries.crash.fixture.root)
        .expect("remove exact killed-writer root");
    std::fs::remove_dir_all(&recoveries.first_root).expect("remove exact first recovery root");
    std::fs::remove_dir_all(&recoveries.second_root).expect("remove exact second recovery root");
}