#[allow(unused_imports)] use crate::physical_record_journeys::durability_admission::managed_mutation::rewrite_integrity::*;
use {std::path::Path, std::path::PathBuf};

use {worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::ServingPhysicalRuntime};
use {worth_store_physical_backend::MediaOperationRole};

use {worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::rewrite_redo::produced_rewrite_payloads};
use {worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::selected_segment_rewrite::prepare_rewrite};
use {worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};

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

    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_open(&root);
    let (effects, refusals) = baseline(&serving);
    match prepare_rewrite(&serving, placement, [62; 32]).execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate) => assert_eq!(
            fate.cause(),
            worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause::AdmissionDeniedBeforeGroupSeal
        ),
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_) => panic!("a damaged source was rewritten"),
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(_) => {
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

    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_open(&root);
    let (effects, refusals) = baseline(&serving);
    match prepare_rewrite(&serving, placement, [62; 32]).execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_) => {}
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            panic!("the undamaged twin proved no effect: {:?}", fate.cause())
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(_) => panic!("the undamaged twin is indeterminate"),
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
