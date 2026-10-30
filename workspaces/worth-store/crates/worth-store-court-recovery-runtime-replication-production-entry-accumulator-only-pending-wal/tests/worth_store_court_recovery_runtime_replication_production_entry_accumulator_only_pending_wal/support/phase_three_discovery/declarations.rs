pub(crate) use {
    crate::phase_three_discovery::phase_three_support::synthetic_topology::publish_synthetic_branched_genesis,
    crate::phase_three_discovery::phase_three_support::synthetic_topology::publish_synthetic_nonempty_genesis,
};
pub(crate) use {
    std::path::Path, worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery,
    worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery_with_limits,
    worth_store_testkit_recovery_runtime::phase_three_support::expect_blocked,
    worth_store_testkit_recovery_runtime::phase_three_support::initialize_store,
    worth_store_testkit_recovery_runtime::phase_three_support::limit_declaration,
    worth_store_testkit_recovery_runtime::phase_three_support::limits_for,
    worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_checkpoint,
    worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_genesis,
    worth_store_testkit_recovery_runtime::phase_three_support::publish_synthetic_wal_tail,
};
pub(crate) use {
    worth_store_recovery_runtime::PhysicalRecoveryBlockKind,
    worth_store_recovery_runtime::PhysicalRecoveryLimitDimension,
    worth_store_recovery_runtime::PhysicalRecoveryLimits,
    worth_store_recovery_runtime::PhysicalRecoveryPlatformAuthority,
    worth_store_recovery_runtime::PhysicalRecoverySourceDenial,
    worth_store_recovery_runtime::PhysicalRecoveryWalIntegrityObservationOutcome,
};

pub(crate) fn write_two_eight_byte_residue_files(root: &std::path::Path) {
    let wal = root.join("families").join("wal");
    std::fs::create_dir_all(&wal).unwrap();
    std::fs::write(wal.join("residue-a"), b"12345678").unwrap();
    std::fs::write(wal.join("residue-b"), b"abcdefgh").unwrap();
}
