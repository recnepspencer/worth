use std::io::Write;
use std::path::Path;
#[allow(unused_imports)]
use worth_store_testkit_store::physical_record_journeys::residency_pressure_processes::*;

use worth_store_physical_backend::MediaOperationRole;
use {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy,
    worth_store::physical_runtime::ExternalPhysicalRecordLocator,
    worth_store::physical_runtime::PhysicalOperationAllocationScope,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalRecordResidencyPolicy,
    worth_store::physical_runtime::PhysicalResidencyDimension,
    worth_store::physical_runtime::PhysicalSpeculativeWorkKind,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadLimits,
};

use {
    worth_store_testkit_store::allocation_probe::physical_record_journeys::peak_live_bytes_during,
    worth_store_testkit_store::durable_publication,
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::media,
    worth_store_testkit_store::physical_record_journeys::success,
    worth_store_testkit_store::stream_fixture::RepeatedByteSource,
};

#[test]
fn bounded_residency_survives_pressure_across_three_processes() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("bounded-residency");
    let locators = parent.path().join("locators.txt");
    let writer =
        worth_store_testkit_store::physical_record_journeys::child_process::invocation::run_child(
            "residency_pressure_writer",
            &root,
            Some(locators.to_str().unwrap()),
        );
    assert!(writer
        .lines()
        .any(|line| line == "RESIDENCY_PRESSURE writer-ok"));
    assert!(directory_bytes(&root) >= RESIDENT_BYTES * 8);

    for _ in 0..2 {
        let reader = worth_store_testkit_store::physical_record_journeys::child_process::invocation::run_child(
            "residency_pressure_reader",
            &root,
            Some(locators.to_str().unwrap()),
        );
        assert!(reader
            .lines()
            .any(|line| line == "RESIDENCY_PRESSURE reader-ok"));
    }
}
