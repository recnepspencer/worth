//! Durable O(1) root pins against Store's live displaced-segment retirement
//! (Store Milestone 4, Phase M4.1, `S4` / `M5`).
use worth_store::physical_runtime::stability::durable_pins::{
    PhysicalRootPinWrite, PHYSICAL_ROOT_PINS,
};
use worth_store::physical_runtime::{
    OrderedJournalRegistry, PhysicalMutationDeadline, PhysicalRecordInitialization,
    PhysicalRecordOpen, PhysicalRetirementDenial, ServingPhysicalRuntime,
};
use worth_store::pin_table::{RootPinDenial, RootPinDropPolicy, RootPinId, RootPinState};
use worth_store_testkit_store::durability_admission::managed_mutation::published_segments::segment_names;
use worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::read_from_reader;
use worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::selected_segment_rewrite::prepare_rewrite;
use worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::{
    completed, prepare,
};
use worth_store_testkit_store::physical_record_journeys::{
    configuration, durability, media, success,
};

const RETAIN: RootPinDropPolicy = RootPinDropPolicy::RetainUntilChildrenDrain;

fn registry() -> OrderedJournalRegistry {
    OrderedJournalRegistry::declare(1, &[PHYSICAL_ROOT_PINS], &[]).unwrap()
}

fn initialized(root: &std::path::Path) -> ServingPhysicalRuntime {
    let (format, placement, access) = configuration();
    let media = media(root);
    let durability = durability(&media);
    success(
        media.initialize_record_store(
            PhysicalRecordInitialization::new(format, placement, access, durability)
                .with_journal_registry(registry()),
        ),
    )
}

fn reopened(root: &std::path::Path) -> ServingPhysicalRuntime {
    let (format, _, access) = configuration();
    let media = media(root);
    let durability = durability(&media);
    success(media.open_record_store(
        PhysicalRecordOpen::new(format, access, durability).with_journal_registry(registry()),
    ))
}

fn write() -> PhysicalRootPinWrite {
    PhysicalRootPinWrite {
        placement: configuration().1,
        deadline: PhysicalMutationDeadline::at(
            worth_signal::facade::TemporalDuration::temporal_duration(1_000).unwrap(),
        ),
    }
}

/// Retire until Store refuses; returns the refusal.
fn retire_all(serving: &ServingPhysicalRuntime) -> PhysicalRetirementDenial {
    loop {
        if let Err(denial) = serving.retire_displaced_segment() {
            return denial;
        }
    }
}

fn pin(serving: &ServingPhysicalRuntime, parent: Option<RootPinId>, payload: &[u8]) -> RootPinId {
    serving
        .root_pins()
        .unwrap()
        .pin_current(parent, RETAIN, payload.to_vec(), write())
        .unwrap()
        .pin()
}

#[test]
fn pinned_root_survives_retirement_and_reopen_while_unrelated_garbage_reclaims() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = initialized(&root);
    let placement = configuration().1;
    let written = completed(prepare(&serving, placement, [61; 32], b"before-pin").execute());
    let record = written.into_acknowledgment().record_ids().next().unwrap();
    // Displaced before the pin: garbage no pinned root reads.
    completed(prepare_rewrite(&serving, placement, [62; 32]).execute());
    let frontier = pin(&serving, None, b"frontier");
    let at_pin = segment_names(&root);
    let pinned_generation = serving
        .root_pins()
        .unwrap()
        .root_generation(frontier)
        .unwrap();
    // HEAD moves on and displaces more garbage after the pin.
    completed(prepare_rewrite(&serving, placement, [63; 32]).execute());
    // A pin is never a global veto: every unrelated displaced segment retires.
    assert_eq!(retire_all(&serving), PhysicalRetirementDenial::Absent);
    let held = segment_names(&root);
    assert!(
        at_pin.iter().filter(|name| !held.contains(name)).count() >= 2,
        "pre-pin and post-pin garbage must retire while the pin is live"
    );
    let read_pinned = |serving: &ServingPhysicalRuntime| {
        let pins = serving.root_pins().unwrap();
        let loan = pins.loan(frontier).unwrap();
        assert_eq!(loan.payload(), b"frontier");
        read_from_reader(&loan.records().unwrap(), record)
    };
    assert_eq!(read_pinned(&serving), b"before-pin");
    serving.close();

    let serving = reopened(&root);
    let pins = serving.root_pins().unwrap();
    assert!(pins.restored());
    assert_eq!(pins.state(frontier), Some(RootPinState::Active));
    assert_eq!(pins.root_generation(frontier), Some(pinned_generation));
    assert_eq!(pins.payload(frontier).as_deref(), Some(&b"frontier"[..]));
    assert_eq!(retire_all(&serving), PhysicalRetirementDenial::Absent);
    let restored = segment_names(&root);
    assert!(
        held.iter().all(|name| restored.contains(name)),
        "reopen restores the pin before any retirement"
    );
    assert_eq!(read_pinned(&serving), b"before-pin");

    // Expiry fences the pinned reader door and releases the pin durably.
    assert_eq!(pins.expire(frontier, write()), Ok(RootPinState::Expired));
    assert_eq!(
        pins.loan(frontier).unwrap_err(),
        RootPinDenial::LeaseExpired
    );
    drop(pins);
    serving.close();
    let serving = reopened(&root);
    assert_eq!(
        serving.root_pins().unwrap().state(frontier),
        Some(RootPinState::Expired)
    );
    assert_eq!(serving.root_pins().unwrap().live_pins(), 0);
    serving.close();
}

#[test]
fn nested_grandchild_holds_every_ancestor_until_it_drains_across_a_crash() {
    let parent_dir = tempfile::tempdir().unwrap();
    let root = parent_dir.path().join("store");
    let serving = initialized(&root);
    let placement = configuration().1;
    completed(prepare(&serving, placement, [71; 32], b"parent-frontier").execute());
    let parent = pin(&serving, None, b"parent");
    completed(prepare(&serving, placement, [72; 32], b"child-frontier").execute());
    let child = pin(&serving, Some(parent), b"child");
    let grandchild = pin(&serving, Some(child), b"grandchild");
    let pins = serving.root_pins().unwrap();
    let loan = pins.loan(grandchild).unwrap();

    // Active -> Expiring fences new parent loans; maintenance cannot release it.
    assert_eq!(pins.expire(parent, write()), Ok(RootPinState::Expiring));
    assert_eq!(pins.settle_drained(write()), Ok(0));
    assert_eq!(pins.loan(parent).unwrap_err(), RootPinDenial::LeaseExpired);
    assert_eq!(
        pins.pin_current(Some(parent), RETAIN, Vec::new(), write())
            .unwrap_err(),
        RootPinDenial::ParentPinNotActive
    );
    assert_eq!(pins.expire(child, write()), Ok(RootPinState::Expiring));
    assert_eq!(pins.expire(grandchild, write()), Ok(RootPinState::Expiring));
    assert_eq!(pins.state(parent), Some(RootPinState::Expiring));
    assert!(
        loan.records().is_ok(),
        "an admitted grandchild loan keeps reading"
    );
    drop(loan);
    drop(pins);
    // Crash before maintenance completes any expiry.
    serving.abort();

    let serving = reopened(&root);
    let pins = serving.root_pins().unwrap();
    for pin in [parent, child, grandchild] {
        assert_eq!(pins.state(pin), Some(RootPinState::Expiring));
    }
    // Loans were process-local; maintenance drains bottom-up in one pass.
    assert_eq!(pins.settle_drained(write()), Ok(3));
    for pin in [grandchild, child, parent] {
        assert_eq!(pins.state(pin), Some(RootPinState::Expired));
    }
    assert_eq!(pins.live_pins(), 0);
    drop(pins);
    serving.close();
    let serving = reopened(&root);
    assert_eq!(serving.root_pins().unwrap().live_pins(), 0);
    serving.close();
}
