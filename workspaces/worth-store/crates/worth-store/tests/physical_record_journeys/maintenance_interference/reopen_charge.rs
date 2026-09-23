use worth_store::physical_runtime::{PhysicalMutationOutcome, PhysicalRetirementDenial};

use super::reopen::open_interference;
use super::{append, checkpoint, initialize, placement, segment_files};

const PAGE_BYTES: u64 = 16 * 1024;

/// A retirement stopped after its unlink is reconstructed on reopen from both
/// the displaced chain and the unresolved WAL intent. It must be charged once.
#[test]
fn reopen_with_pending_retirement_charges_the_displaced_page_once() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = initialize(&root);
    let policy = placement();
    append(&serving, policy, 1, b"reopen-charge");
    checkpoint(&serving, 1);
    match super::interleave::rewrite(&serving, policy, 2).execute() {
        PhysicalMutationOutcome::Completed(_) => {}
        PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            panic!("rewrite had no effect: {:?}", fate.cause())
        }
        PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!("rewrite became indeterminate at {:?}", fate.stage())
        }
    }
    serving.close();
    let displaced_only = charge_on_reopen(&root);

    let serving = open_interference(&root);
    let files = segment_files(&root);
    serving.certification_stop_after_retirement_delete();
    assert!(serving.retire_displaced_segment().is_err());
    assert!(files.difference(&segment_files(&root)).next().is_some());
    serving.close();
    let pending = charge_on_reopen(&root);
    let pending_again = charge_on_reopen(&root);

    let serving = open_interference(&root);
    assert_eq!(serving.certification_charged_growth_bytes(), pending);
    serving.retire_displaced_segment().unwrap();
    let released = serving.certification_charged_growth_bytes();
    serving.close();
    let settled = open_interference(&root);
    assert_eq!(
        settled.retire_displaced_segment(),
        Err(PhysicalRetirementDenial::Absent)
    );
    let settled_charge = settled.certification_charged_growth_bytes();
    settled.close();

    // The intent adds WAL bytes; a second charge would add a whole page.
    assert!(pending >= displaced_only);
    assert!(
        pending - displaced_only < PAGE_BYTES,
        "reopen charged the pending retirement twice: {displaced_only} -> {pending}"
    );
    assert_eq!(
        pending_again, pending,
        "reopen alone must not move the charge"
    );
    assert_eq!(pending - released, PAGE_BYTES);
    // Completion leaves only publication and WAL bytes; nothing is resurrected.
    assert!(settled_charge.abs_diff(released) < PAGE_BYTES);
}

fn charge_on_reopen(root: &std::path::Path) -> u64 {
    let serving = open_interference(root);
    let charged = serving.certification_charged_growth_bytes();
    serving.close();
    charged
}
