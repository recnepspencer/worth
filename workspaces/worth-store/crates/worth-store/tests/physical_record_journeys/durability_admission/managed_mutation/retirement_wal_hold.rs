use std::num::{NonZeroU32, NonZeroU64};
use std::path::Path;

use worth_proof::TransitionOutcome;
use worth_signal::facade::TemporalDuration;
use worth_store::physical_runtime::{
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest, PhysicalRecordInitialization, PhysicalRecordOpen,
    PhysicalRetirementDenial, PhysicalWalPolicy,
    ServingPhysicalRuntime, WalSegmentByteLimit, WalSegmentInventoryLimit,
};

use super::selected_segment_rewrite::prepare_rewrite;
use super::*;

const SEGMENT_BYTES: u64 = 35_268;

#[test]
fn unresolved_retirement_survives_wal_rotation_and_checkpoint() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let wal = wal_policy();
    let serving = open_initialized(&root, wal);
    let (format, placement, _) = configuration();
    let page_bytes = u64::from(format.declaration().page_size().bytes());
    completed(prepare(&serving, placement, [71; 32], b"hold-source").execute());
    completed(prepare_rewrite(&serving, placement, [72; 32]).execute());
    serving.certification_stop_after_retirement_delete();
    assert!(serving.retire_displaced_segment().is_err());
    let mut rotated = false;
    for index in 0..8 {
        let byte = 80 + u8::try_from(index).unwrap();
        completed(prepare(&serving, placement, [byte; 32], b"rotate-tail").execute());
        if intent_is_behind_the_active_tail(&root) {
            rotated = true;
            break;
        }
    }
    assert!(rotated, "a later append must leave the retirement intent off the active tail");
    let holders = retirement_wal_files(&root);
    assert!(!holders.is_empty());
    checkpoint(&serving);
    let after = wal_names(&root);
    assert!(
        holders.iter().all(|name| after.contains(name)),
        "checkpoint reclamation must keep the unresolved retirement segment"
    );
    serving.close();
    let serving = open_existing(&root, wal);
    let charged = serving.certification_charged_growth_bytes();
    serving.close();
    let serving = open_existing(&root, wal);
    assert_eq!(serving.certification_charged_growth_bytes(), charged);
    serving.retire_displaced_segment().unwrap();
    assert_eq!(
        charged - serving.certification_charged_growth_bytes(),
        page_bytes
    );
    serving.close();
    let serving = open_existing(&root, wal);
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(PhysicalRetirementDenial::Absent)
    );
    let released = serving.certification_charged_growth_bytes();
    serving.close();
    let serving = open_existing(&root, wal);
    assert_eq!(serving.certification_charged_growth_bytes(), released);
    serving.close();
}

fn wal_policy() -> PhysicalWalPolicy {
    PhysicalWalPolicy::segmented(
        WalSegmentByteLimit::new(NonZeroU64::new(SEGMENT_BYTES).unwrap()),
        WalSegmentInventoryLimit::new(NonZeroU32::new(16).unwrap()),
    )
}

fn open_initialized(root: &Path, wal: PhysicalWalPolicy) -> ServingPhysicalRuntime {
    let (format, placement, access) = configuration();
    let media = crate::media(root);
    let durability = crate::durability_with_wal_policy(&media, wal);
    crate::success(media.initialize_record_store(PhysicalRecordInitialization::new(
        format, placement, access, durability,
    )))
}

fn open_existing(root: &Path, wal: PhysicalWalPolicy) -> ServingPhysicalRuntime {
    let (format, _, access) = configuration();
    let media = crate::media(root);
    let durability = crate::durability_with_wal_policy(&media, wal);
    crate::success(
        media.open_record_store(PhysicalRecordOpen::new(format, access, durability)),
    )
}

fn checkpoint(serving: &ServingPhysicalRuntime) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([41; 32]),
        PhysicalCheckpointDeadline::at(
            TemporalDuration::temporal_duration(30_000).expect("deadline is positive"),
        ),
    );
    let handle = match serving.checkpoints().start(request).into_raw() {
        TransitionOutcome::Success(handle) => handle,
        _ => panic!("checkpoint admission did not produce a handle"),
    };
    match handle.wait() {
        PhysicalCheckpointOutcome::Completed(_) => {}
        _ => panic!("checkpoint after WAL rotation must complete"),
    }
}

fn intent_is_behind_the_active_tail(root: &Path) -> bool {
    let names = wal_names(root);
    let Some(newest) = names.last() else {
        return false;
    };
    names.len() > 1 && !file_contains_retirement(newest)
}

fn retirement_wal_files(root: &Path) -> Vec<String> {
    wal_names(root)
        .into_iter()
        .filter(|name| file_contains_retirement(name))
        .collect()
}

fn wal_names(root: &Path) -> Vec<String> {
    let mut names = Vec::new();
    collect_files(&root.join("families").join("wal"), &mut names);
    names.sort();
    names
}

fn collect_files(path: &Path, names: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, names);
        } else {
            names.push(path.to_string_lossy().into_owned());
        }
    }
}

fn file_contains_retirement(path: &str) -> bool {
    std::fs::read(path).is_ok_and(|bytes| {
        bytes
            .windows(b"store.physical.retirement.v1".len())
            .any(|window| window == b"store.physical.retirement.v1")
    })
}
