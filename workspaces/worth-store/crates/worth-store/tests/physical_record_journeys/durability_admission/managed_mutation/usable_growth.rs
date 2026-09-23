use worth_store::physical_runtime::{
    PhysicalMutationOutcome, PhysicalMutationProvenNoEffectCause, ServingPhysicalRuntime,
};
use worth_store_physical_backend::MediaOperationRole;

use super::selected_segment_rewrite::{completed, prepare_rewrite};
use super::*;

#[test]
fn append_is_denied_one_byte_over_usable_growth() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (format, placement, _) = configuration();
    let page_bytes = u64::from(format.declaration().page_size().bytes());
    serving.certification_limit_candidate_growth_bytes(page_bytes.saturating_sub(1));
    let writes = positioned_writes(&serving);
    let outcome = prepare(&serving, placement, [41; 32], b"growth-append").execute();
    assert_denied_before_effects(&serving, outcome, writes, "one-byte-over append");
    serving.close();
}

#[test]
fn append_is_denied_when_usable_growth_is_only_the_data_page() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (format, placement, _) = configuration();
    let page_bytes = u64::from(format.declaration().page_size().bytes());
    serving.certification_limit_candidate_growth_bytes(page_bytes);
    let writes = positioned_writes(&serving);
    let outcome = prepare(&serving, placement, [46; 32], b"wal-root-growth").execute();
    assert_denied_before_effects(
        &serving,
        outcome,
        writes,
        "append without WAL and root budget",
    );
    serving.close();
}

#[test]
fn selected_segment_rewrite_is_denied_one_byte_over_usable_growth() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (format, placement, _) = configuration();
    let page_bytes = u64::from(format.declaration().page_size().bytes());
    completed(prepare(&serving, placement, [39; 32], b"growth-source").execute());
    serving.certification_limit_candidate_growth_bytes(page_bytes.saturating_sub(1));
    let writes = positioned_writes(&serving);
    let outcome = prepare_rewrite(&serving, placement, [40; 32]).execute();
    assert_denied_before_effects(&serving, outcome, writes, "one-byte-over rewrite");
    serving.close();
}

fn positioned_writes(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .media_counters()
        .attempts_for(MediaOperationRole::PositionedWrite)
}

fn assert_denied_before_effects(
    serving: &ServingPhysicalRuntime,
    outcome: PhysicalMutationOutcome,
    writes: u64,
    journey: &str,
) {
    match outcome {
        PhysicalMutationOutcome::ProvenNoEffect(fate) => assert_eq!(
            fate.cause(),
            PhysicalMutationProvenNoEffectCause::RetentionPressure
        ),
        PhysicalMutationOutcome::Completed(_) => {
            panic!("{journey} must be denied before WAL effects")
        }
        PhysicalMutationOutcome::Indeterminate(_) => {
            panic!("{journey} must prove no effect before WAL effects")
        }
    }
    assert_eq!(serving.certification_pending_publication_count(), 0);
    assert_eq!(positioned_writes(serving), writes);
}
