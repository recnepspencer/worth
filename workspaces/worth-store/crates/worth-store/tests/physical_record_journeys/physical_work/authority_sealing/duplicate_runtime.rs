use super::super::{
    executor::admitted_write,
    fixture::{serving_from_initialization_with_work_profile, work_fixture},
};
use tempfile::tempdir;
use worth_store::physical_runtime::{PhysicalExecutorCommand, PhysicalStoreCloseOutcome};

#[test]
fn a_second_pending_work_registry_is_forbidden() {
    let root = tempdir().unwrap();
    let (profile, _, mutation) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let command = PhysicalExecutorCommand::exact_write(
        admitted_write(&serving, mutation),
        b"registry".as_slice(),
    )
    .unwrap();
    serving.execute_physical_work(command).unwrap();

    assert!(
        matches!(
            serving.close_plan().execute(),
            PhysicalStoreCloseOutcome::Closed { .. }
        ),
        "C5_PREDICATE:duplicate-work-registry: shadow registry stole settlement from the canonical command arena"
    );
}

#[test]
fn a_second_physical_lifecycle_is_forbidden() {
    let root = tempdir().unwrap();
    let (profile, read, _) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let receipt = super::super::readiness::success(serving.physical_read_submission().submit(read));

    serving.admit_physical_work(receipt).unwrap_or_else(|denial| {
        panic!(
            "C5_PREDICATE:lifecycle-duplication: lifecycle detached from the canonical runtime owner: {denial:?}"
        )
    });
    serving.close();
}
