//! Checkpoint liveness against foreign pool holds. A checkpoint draws only on
//! the standing capture reservation, so an admitted drop's checkpoint, and the
//! retirement waiting on it, still progress when another owner holds every
//! byte the pool would grant to an ordinary scope but one. Ordinary reopen
//! never holds released-drop custody (it requires C.8), so the entry
//! reservation is proven on the C.8 rejoin in production_entry.

#[allow(unused_imports)]
use crate::physical_blob_journeys::checkpoint_liveness::*;

use worth_store_testkit_store::fixture::serving_from_initialization;
use worth_store_testkit_store::physical_blob_journeys::checkpoint_capture_envelope::release_one_batch;

#[test]
fn admitted_drop_checkpoints_and_retires_under_a_foreign_hold() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let mut receipt = release_one_batch(&serving, 0x31, true);
    assert_eq!(
        receipt.retirement(),
        worth_store::physical_runtime::BlobReclaimRetirement::Pending(
            worth_store::physical_runtime::PhysicalRetirementDenial::Checkpoint
        )
    );
    let hold = hold_all_but_one_byte(&serving);
    checkpoint(&serving, 0x91);
    let retirement = serving
        .blobs()
        .unwrap()
        .continue_reclaim_retirement(&mut receipt, retirement_budget())
        .unwrap();
    assert_ne!(
        retirement,
        worth_store::physical_runtime::BlobReclaimRetirement::Pending(
            worth_store::physical_runtime::PhysicalRetirementDenial::Checkpoint
        ),
        "the committed checkpoint must release the drop's retirement"
    );
    drop(hold);
    serving.close();
}
