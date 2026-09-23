use std::io::Write;
use std::path::Path;

use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, PhysicalReadProtectionPolicy, PhysicalRecordAccessPolicy,
    PhysicalRecordFormatDeclaration, PhysicalRecordOpen, PhysicalWalPolicy, WalSegmentByteLimit,
    WalSegmentInventoryLimit,
};

const BASELINE: &[u8] = b"phase6-baseline";

#[test]
fn serving_child_reopens_the_checkpointed_baseline() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = super::initialize(&root);
    let id = super::append(&serving, super::placement(), 1, BASELINE);
    let mut session = serving
        .records()
        .unwrap()
        .open(id, super::limits())
        .unwrap();
    assert_eq!(session.next_chunk().unwrap().unwrap().bytes(), BASELINE);
    drop(session);
    super::checkpoint(&serving, 1);
    let generation = serving
        .observer()
        .acquisition_snapshot()
        .unwrap()
        .root_generation();
    serving.close();
    let output = super::super::child_process::run_child(
        "interference_baseline_reopener",
        &root,
        Some(&generation.to_string()),
    );
    let line = output
        .lines()
        .find(|line| line.starts_with("PHASE6_REOPEN "))
        .expect("serving child must report the reopened baseline");
    let mut fields = line.split_whitespace();
    assert_eq!(fields.next(), Some("PHASE6_REOPEN"));
    let child_generation: u64 = fields.next().unwrap().parse().unwrap();
    let peak: u64 = fields.next().unwrap().parse().unwrap();
    assert_eq!(fields.next(), Some("phase6-baseline"));
    assert_eq!(child_generation, generation);
    assert!(
        peak <= super::RESIDENT_BYTES,
        "resident frames peaked at {peak}"
    );
}

pub(crate) fn serving_child(root: &Path) {
    let expected = std::env::var(super::super::child_process::LOCATOR_ENV)
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
    let records = super::super::scan_journeys::collect_scan(&serving, 4, 64_000);
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

pub(super) fn open_interference(
    root: &Path,
) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let media = super::super::media(root);
    let durability = super::super::durability_with_wal_policy(
        &media,
        PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(std::num::NonZeroU64::new(512 * 1024).unwrap()),
            WalSegmentInventoryLimit::new(std::num::NonZeroU32::new(64).unwrap()),
        ),
    );
    super::super::success(
        media.open_record_store(
            PhysicalRecordOpen::new(format, access, durability)
                .with_residency_policy(super::residency(format, super::RESIDENT_BYTES))
                .with_read_protection_policy(PhysicalReadProtectionPolicy::default()),
        ),
    )
}
