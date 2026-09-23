use std::path::{Path, PathBuf};

use worth_store::physical_runtime::{PhysicalMutationOutcome, ServingPhysicalRuntime};
use worth_store_physical_backend::MediaOperationRole;

use super::super::independent_wal_oracle::produced_rewrite_payloads;
use super::selected_segment_rewrite::prepare_rewrite;
use super::*;

/// Every media role that can change bytes or the namespace.
const EFFECT_ROLES: [MediaOperationRole; 7] = [
    MediaOperationRole::CreateNew,
    MediaOperationRole::PositionedWrite,
    MediaOperationRole::Append,
    MediaOperationRole::Truncate,
    MediaOperationRole::Allocate,
    MediaOperationRole::AtomicReplace,
    MediaOperationRole::Delete,
];

#[test]
fn a_damaged_source_page_denies_the_rewrite_before_any_write() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let segment = published_source(&root);
    let mut bytes = std::fs::read(&segment).unwrap();
    let (format, placement, _) = configuration();
    // Half a page in lands inside the first page frame's checksummed body,
    // past its header, so only integrity admission can notice the flip.
    let damage_offset = format.declaration().page_size().bytes() as usize / 2;
    bytes[damage_offset] ^= 0xFF;
    std::fs::write(&segment, &bytes).unwrap();

    let serving = crate::serving_from_open(&root);
    let (effects, refusals) = baseline(&serving);
    match prepare_rewrite(&serving, placement, [62; 32]).execute() {
        PhysicalMutationOutcome::ProvenNoEffect(fate) => assert_eq!(
            fate.cause(),
            PhysicalMutationProvenNoEffectCause::AdmissionDeniedBeforeGroupSeal
        ),
        PhysicalMutationOutcome::Completed(_) => panic!("a damaged source was rewritten"),
        PhysicalMutationOutcome::Indeterminate(_) => {
            panic!("a damaged source became effectful before its denial")
        }
    }
    let (after_effects, after_refusals) = baseline(&serving);
    assert_eq!(
        after_effects, effects,
        "the denial precedes every media effect"
    );
    assert!(
        after_refusals > refusals,
        "integrity admission localized the damaged source"
    );
    assert!(
        produced_rewrite_payloads(&root).is_empty(),
        "no rewrite redo reaches the WAL"
    );
    assert_eq!(std::fs::read(&segment).unwrap(), bytes);
    serving.close();
}

#[test]
fn the_same_source_undamaged_is_rewritten_after_reopen() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let segment = published_source(&root);
    let source_bytes = std::fs::read(&segment).unwrap();
    let (_, placement, _) = configuration();

    let serving = crate::serving_from_open(&root);
    let (effects, refusals) = baseline(&serving);
    match prepare_rewrite(&serving, placement, [62; 32]).execute() {
        PhysicalMutationOutcome::Completed(_) => {}
        PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            panic!("the undamaged twin proved no effect: {:?}", fate.cause())
        }
        PhysicalMutationOutcome::Indeterminate(_) => panic!("the undamaged twin is indeterminate"),
    }
    let (after_effects, after_refusals) = baseline(&serving);
    assert_eq!(after_refusals, refusals);
    assert!(after_effects > effects, "the twin reaches media");
    let rewrites = produced_rewrite_payloads(&root);
    assert_eq!(rewrites.len(), 1, "exactly one rewrite redo is durable");
    assert_eq!(rewrites[0].source_length, rewrites[0].destination_length);
    assert_eq!(
        std::fs::read(&segment).unwrap(),
        source_bytes,
        "the rewrite copies into a new generation and leaves the source intact"
    );
    assert_eq!(segment_files(&root).len(), 2);
    serving.close();
}

fn published_source(root: &Path) -> PathBuf {
    let serving = serving_from_initialization(root);
    let (_, placement, _) = configuration();
    match prepare(&serving, placement, [61; 32], b"damaged-rewrite-source").execute() {
        PhysicalMutationOutcome::Completed(_) => {}
        _ => panic!("the source append did not complete"),
    }
    serving.close();
    let mut segments = segment_files(root);
    assert_eq!(segments.len(), 1, "one append publishes one segment");
    segments.pop().unwrap()
}

fn segment_files(root: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(root.join("families/records/segments"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect()
}

fn baseline(serving: &ServingPhysicalRuntime) -> (u64, u64) {
    let counters = serving.media_counters();
    (
        EFFECT_ROLES
            .iter()
            .map(|role| counters.attempts_for(*role))
            .sum(),
        serving
            .resident_admission_counters()
            .refusals_before_owner_entry(),
    )
}
