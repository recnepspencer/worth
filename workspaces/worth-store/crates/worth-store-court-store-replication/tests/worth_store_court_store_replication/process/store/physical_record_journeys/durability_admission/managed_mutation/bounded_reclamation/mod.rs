//! S6 (`M5`, `M8`, `D5`): bounded `ReclaimBudget` maintenance against live
//! in-process registrations and durable root pins. Every pass retires at most
//! its budget slice, held roots keep their bytes through repeated
//! edit/retire cycles, and disk space returns once the holders release.
use worth_store::physical_runtime::stability::durable_pins::{
    PhysicalRootPinWrite, PHYSICAL_ROOT_PINS,
};
use worth_store::physical_runtime::{
    OrderedJournalRegistry, PhysicalMutationDeadline, PhysicalReclaimBudget, PhysicalReclaimPass,
    PhysicalReclaimPassEnd, PhysicalRecordInitialization, PhysicalRetirementDenial,
    ServingPhysicalRuntime,
};
use worth_store::pin_table::{RootPinDropPolicy, RootPinState};
use worth_store_testkit_store::durability_admission::managed_mutation::published_segments::segment_names;
use worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::read_from_reader;
use worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::selected_segment_rewrite::prepare_rewrite;
use worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::{
    completed, prepare,
};
use worth_store_testkit_store::physical_record_journeys::{
    configuration, durability, media, success,
};

fn initialized(root: &std::path::Path) -> ServingPhysicalRuntime {
    let (format, placement, access) = configuration();
    let media = media(root);
    let durability = durability(&media);
    let registry = OrderedJournalRegistry::declare(1, &[PHYSICAL_ROOT_PINS], &[]).unwrap();
    success(
        media.initialize_record_store(
            PhysicalRecordInitialization::new(format, placement, access, durability)
                .with_journal_registry(registry),
        ),
    )
}

fn write() -> PhysicalRootPinWrite {
    PhysicalRootPinWrite {
        placement: configuration().1,
        deadline: PhysicalMutationDeadline::at(
            worth_signal::facade::TemporalDuration::temporal_duration(1_000).unwrap(),
        ),
    }
}

fn segment_bytes(root: &std::path::Path) -> u64 {
    let segments = root.join("families").join("records").join("segments");
    segment_names(root)
        .iter()
        .map(|name| std::fs::metadata(segments.join(name)).unwrap().len())
        .sum()
}

fn bounded_pass(
    serving: &ServingPhysicalRuntime,
    budget: PhysicalReclaimBudget,
) -> PhysicalReclaimPass {
    let pass = serving.reclaim_within_budget(budget);
    assert!(pass.retired_artifacts <= budget.max_artifacts(), "{pass:?}");
    assert!(pass.retired_bytes <= budget.max_bytes(), "{pass:?}");
    pass
}

#[test]
fn bounded_reclamation_keeps_held_roots_and_returns_space_after_release() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = initialized(&root);
    let placement = configuration().1;
    let written = completed(prepare(&serving, placement, [81; 32], b"held-history").execute());
    let record = written.into_acknowledgment().record_ids().next().unwrap();
    // An in-process registration (a live reader) holds the first root, and a
    // durable pin holds the root after one edit.
    let reader = serving.records().unwrap();
    completed(prepare_rewrite(&serving, placement, [82; 32]).execute());
    let pin = serving
        .root_pins()
        .unwrap()
        .pin_current(
            None,
            RootPinDropPolicy::RetainUntilChildrenDrain,
            b"s6".to_vec(),
            write(),
        )
        .unwrap()
        .pin();

    // Repeated edit -> bounded maintenance cycles while both holders live.
    let one = PhysicalReclaimBudget::new(1, u64::MAX);
    for material in 83..89 {
        completed(prepare_rewrite(&serving, placement, [material; 32]).execute());
        bounded_pass(&serving, one);
        assert_eq!(read_from_reader(&reader, record), b"held-history");
        let pins = serving.root_pins().unwrap();
        let loan = pins.loan(pin).unwrap();
        assert_eq!(
            read_from_reader(&loan.records().unwrap(), record),
            b"held-history"
        );
    }
    // A zero or undersized budget retires nothing, however much awaits.
    let starved = bounded_pass(&serving, PhysicalReclaimBudget::new(0, u64::MAX));
    assert_eq!(starved.retired_artifacts, 0);
    let undersized = bounded_pass(&serving, PhysicalReclaimBudget::new(8, 1));
    assert_eq!(undersized.retired_artifacts, 0);
    assert_eq!(undersized.end, PhysicalReclaimPassEnd::BudgetExhausted);
    // The registration still blocks its displaced generation (`M5`).
    let held = bounded_pass(&serving, PhysicalReclaimBudget::new(u64::MAX, u64::MAX));
    assert_eq!(
        held.end,
        PhysicalReclaimPassEnd::Denied(PhysicalRetirementDenial::Protected)
    );
    let held_bytes = segment_bytes(&root);
    let held_segments = segment_names(&root).len();
    assert_eq!(read_from_reader(&reader, record), b"held-history");

    // Release every holder; bounded passes then drain all garbage.
    drop(reader);
    assert_eq!(
        serving.root_pins().unwrap().expire(pin, write()),
        Ok(RootPinState::Expired)
    );
    let two = PhysicalReclaimBudget::new(2, u64::MAX);
    let mut passes = 0;
    let mut retired = 0;
    loop {
        let pass = bounded_pass(&serving, two);
        retired += pass.retired_artifacts;
        passes += 1;
        if pass.end == PhysicalReclaimPassEnd::Drained {
            break;
        }
        assert_eq!(
            pass.end,
            PhysicalReclaimPassEnd::BudgetExhausted,
            "{pass:?}"
        );
        assert!(passes < 64, "bounded passes must converge");
    }
    assert!(retired >= 2, "held garbage retires once released");
    assert!(passes >= 2, "a two-artifact budget needs several passes");
    assert!(segment_names(&root).len() < held_segments);
    assert!(
        segment_bytes(&root) < held_bytes,
        "disk space returns measurably after release"
    );
    assert_eq!(
        read_from_reader(&serving.records().unwrap(), record),
        b"held-history"
    );
    serving.close();
}
