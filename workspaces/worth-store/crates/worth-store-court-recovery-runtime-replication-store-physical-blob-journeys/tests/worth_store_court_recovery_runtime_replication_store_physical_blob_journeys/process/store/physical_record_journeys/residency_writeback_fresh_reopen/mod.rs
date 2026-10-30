use std::io::Write;
use std::path::Path;
#[allow(unused_imports)]
use worth_store_testkit_store::physical_record_journeys::residency_writeback_fresh_reopen::*;

use {sha2::Digest, sha2::Sha256};
use {
    worth_store::physical_runtime::ExternalPhysicalRecordLocator,
    worth_store::physical_runtime::PhysicalWorkEffectFate,
    worth_store::physical_runtime::PhysicalWritebackExecution,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadLimits,
};
use {
    worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement,
    worth_store_physical_backend::MediaOperationRole,
};
use {
    worth_store_physical_format::RecordArtifactFile,
    worth_store_physical_format::RecordFrameCoordinate,
};

#[test]
fn ordinary_record_and_exact_writeback_survive_fresh_store_admission() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let writer =
        worth_store_testkit_store::physical_record_journeys::child_process::invocation::run_child(
            "physical_writeback_writer",
            &root,
            None,
        );
    let evidence = writer
        .lines()
        .find_map(|line| line.strip_prefix("PHYSICAL_WRITEBACK "))
        .expect("writer must publish digest and record locator");
    let (digest, locator) = evidence
        .split_once(' ')
        .expect("writer evidence must separate digest from locator");
    let observer =
        worth_store_testkit_store::physical_record_journeys::child_process::invocation::run_child(
            "physical_writeback_observer",
            &root,
            Some(digest),
        );
    assert!(observer
        .lines()
        .any(|line| line == "PHYSICAL_WRITEBACK_OBSERVED"));
    let reopener =
        worth_store_testkit_store::physical_record_journeys::child_process::invocation::run_child(
            "physical_writeback_reopener",
            &root,
            Some(locator),
        );
    assert!(reopener
        .lines()
        .any(|line| line == "PHYSICAL_WRITEBACK_REOPENED"));
}
