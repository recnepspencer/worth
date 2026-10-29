pub use {std::io::Write};
pub use {std::path::Path};

pub use {worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::PhysicalReadProtectionPolicy, worth_store::physical_runtime::PhysicalRecordAccessPolicy, worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalWalPolicy, worth_store::physical_runtime::WalSegmentByteLimit, worth_store::physical_runtime::WalSegmentInventoryLimit};

pub const BASELINE: &[u8] = b"phase6-baseline";

pub fn serving_child(root: &Path) {
    let expected = std::env::var(crate::child_process::LOCATOR_ENV)
        .expect("baseline generation")
        .parse::<u64>()
        .unwrap();
    let serving = open_interference(root);
    let generation = serving
        .observer()
        .acquisition_snapshot()
        .unwrap()
        .root_generation();
    assert_eq!(generation, expected);
    let records = crate::physical_record_journeys::scan_journeys::collect_scan(&serving, 4, 64_000);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].1, BASELINE);
    let peak = serving
        .certification_physical_residency()
        .counters()
        .peak_resident_bytes();
    println!("PHASE6_REOPEN {generation} {peak} phase6-baseline");
    std::io::stdout().flush().unwrap();
    serving.close();
}

pub fn open_interference(
    root: &Path,
) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let access = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let media = crate::physical_record_journeys::media(root);
    let durability = crate::physical_record_journeys::durability_with_wal_policy(
        &media,
        worth_store::physical_runtime::PhysicalWalPolicy::segmented(
            worth_store::physical_runtime::WalSegmentByteLimit::new(std::num::NonZeroU64::new(512 * 1024).unwrap()),
            worth_store::physical_runtime::WalSegmentInventoryLimit::new(std::num::NonZeroU32::new(64).unwrap()),
        ),
    );
    crate::physical_record_journeys::success(
        media.open_record_store(
            worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
                .with_residency_policy(crate::physical_record_journeys::maintenance_interference::residency(format, crate::physical_record_journeys::maintenance_interference::RESIDENT_BYTES))
                .with_read_protection_policy(worth_store::physical_runtime::PhysicalReadProtectionPolicy::default()),
        ),
    )
}