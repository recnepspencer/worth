#[allow(unused_imports)] use crate::physical_record_journeys::durability_admission::managed_mutation::candidate_verification::*;

use {worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::selected_segment_rewrite::prepare_rewrite};
use worth_store_testkit_store::physical_record_journeys::configuration;

#[test]
fn a_candidate_corrupted_before_publication_is_never_published() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let record = published_source(&root);
    let source_segments = segment_files(&root);

    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_open(&root);
    // The sealed WAL group record is the rewrite's first positioned write;
    // the candidate page is the second.
    let candidate_write = serving
        .media_counters()
        .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
        + 2;
    serving.close();

    let (serving, gate) = serving_paused_after_write(&root, candidate_write);
    let before = root_generation(&serving);
    let (_, placement, _) = configuration();
    let handle = prepare_rewrite(&serving, placement, [64; 32]).start();
    gate.wait_until_reached();
    let page_bytes = configuration().0.declaration().page_size().bytes();
    assert_eq!(
        gate.reached_context().unwrap().requested_bytes(),
        u64::from(page_bytes),
        "the paused write is the candidate page"
    );
    let candidate = segment_files(&root)
        .difference(&source_segments)
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(candidate.len(), 1, "the paused write created one candidate");
    let mut bytes = std::fs::read(&candidate[0]).unwrap();
    bytes[candidate_damage_offset()] ^= 0xFF;
    std::fs::write(&candidate[0], &bytes).unwrap();
    gate.release();

    match handle.wait() {
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(fate) => assert_eq!(
            fate.stage(),
            worth_store::physical_runtime::PhysicalMutationIndeterminateStage::DataDispatch
        ),
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_) => {
            panic!("an unchecked candidate was published")
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            panic!("the sealed rewrite claimed no effect: {:?}", fate.cause())
        }
    }
    assert_eq!(
        root_generation(&serving),
        before,
        "no root names the damaged candidate"
    );
    assert_eq!(
        read_record(&serving, record),
        PAYLOAD,
        "the unchanged root keeps serving the intact source"
    );
    serving.close();

    // The durable rewrite redo without a verified candidate is recovery-owner
    // work: a fresh serving open fails closed instead of guessing its fate.
    let reopened = worth_store_testkit_store::physical_record_journeys::serving_from_open(&root);
    assert_eq!(
        reopened
            .records()
            .unwrap()
            .open(
                record,
                worth_store::physical_runtime::RecordReadLimits::new(
                    worth_store::physical_runtime::RecordByteLimit::new(PAYLOAD.len() as u32)
                        .unwrap()
                ),
            )
            .err()
            .map(|error| error.denial()),
        Some(worth_store::physical_runtime::RecordReadDenial::ServingRequiresInspection)
    );
    reopened.close();
}
