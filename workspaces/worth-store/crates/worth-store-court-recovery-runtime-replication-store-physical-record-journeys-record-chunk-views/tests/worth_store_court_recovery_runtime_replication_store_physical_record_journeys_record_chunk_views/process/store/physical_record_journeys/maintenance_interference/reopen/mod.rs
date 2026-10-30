use std::io::Write;
use std::path::Path;
#[allow(unused_imports)]
use worth_store_testkit_store::physical_record_journeys::maintenance_interference::reopen::*;

use {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::PhysicalReadProtectionPolicy,
    worth_store::physical_runtime::PhysicalRecordAccessPolicy,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalWalPolicy,
    worth_store::physical_runtime::WalSegmentByteLimit,
    worth_store::physical_runtime::WalSegmentInventoryLimit,
};

#[test]
fn serving_child_reopens_the_checkpointed_baseline() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving =
        worth_store_testkit_store::physical_record_journeys::maintenance_interference::initialize(
            &root,
        );
    let id = worth_store_testkit_store::physical_record_journeys::maintenance_interference::append(
        &serving,
        worth_store_testkit_store::physical_record_journeys::maintenance_interference::placement(),
        1,
        BASELINE,
    );
    let mut session = serving
        .records()
        .unwrap()
        .open(
            id,
            worth_store_testkit_store::physical_record_journeys::maintenance_interference::limits(),
        )
        .unwrap();
    assert_eq!(session.next_chunk().unwrap().unwrap().bytes(), BASELINE);
    drop(session);
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::checkpoint(
        &serving, 1,
    );
    let generation = serving
        .observer()
        .acquisition_snapshot()
        .unwrap()
        .root_generation();
    serving.close();
    let output =
        worth_store_testkit_store::physical_record_journeys::child_process::invocation::run_child(
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
        peak <= worth_store_testkit_store::physical_record_journeys::maintenance_interference::RESIDENT_BYTES,
        "resident frames peaked at {peak}"
    );
}
