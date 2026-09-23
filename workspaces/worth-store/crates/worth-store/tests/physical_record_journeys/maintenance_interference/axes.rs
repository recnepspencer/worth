use std::num::NonZeroU32;

use worth_store::physical_runtime::{PhysicalReadProtectionDenial, PhysicalReadProtectionPolicy};

use super::{append, initialize_protection, placement};

#[test]
fn live_leases_admit_one_and_reject_the_next() {
    admit_live_leases(1);
}

#[test]
fn live_leases_admit_eight_and_reject_the_next() {
    admit_live_leases(8);
}

#[test]
fn live_leases_admit_sixty_four_and_reject_the_next() {
    admit_live_leases(64);
}

#[test]
fn retained_roots_admit_one_and_reject_the_next() {
    admit_retained_roots(1);
}

#[test]
fn retained_roots_admit_four_and_reject_the_next() {
    admit_retained_roots(4);
}

#[test]
fn retained_roots_admit_sixteen_and_reject_the_next() {
    admit_retained_roots(16);
}

fn admit_live_leases(bound: u32) {
    let parent = tempfile::tempdir().unwrap();
    let serving = initialize_protection(
        &parent.path().join("store"),
        PhysicalReadProtectionPolicy::new(nonzero(bound), nonzero(1)),
    );
    let record = append(&serving, placement(), 1, b"lease");
    let mut readers = Vec::new();
    for index in 0..bound {
        let reader = serving.records().unwrap();
        if index == 0 {
            let mut session = reader.open(record, super::limits()).unwrap();
            assert_eq!(session.next_chunk().unwrap().unwrap().bytes(), b"lease");
        }
        readers.push(reader);
    }
    let observer = serving.read_protection_observer();
    assert_eq!(observer.snapshot().live_acquisitions(), bound);
    let before = serving.media_counters();
    assert!(matches!(
        serving.records(),
        Err(PhysicalReadProtectionDenial::ProtectionLimit)
    ));
    assert_eq!(serving.media_counters(), before);
    drop(readers);
    assert_eq!(observer.snapshot().live_acquisitions(), 0);
    serving.close();
}

fn admit_retained_roots(bound: u32) {
    let parent = tempfile::tempdir().unwrap();
    let serving = initialize_protection(
        &parent.path().join("store"),
        PhysicalReadProtectionPolicy::new(nonzero(64), nonzero(bound)),
    );
    let policy = placement();
    let mut readers = Vec::new();
    for index in 0..bound {
        append(&serving, policy, u64::from(index) + 1, b"generation");
        readers.push(serving.records().unwrap());
    }
    assert_eq!(readers.len() as u32, bound);
    append(&serving, policy, u64::from(bound) + 1, b"one-over");
    let before = serving.media_counters();
    assert!(matches!(
        serving.records(),
        Err(PhysicalReadProtectionDenial::RetainedRootLimit)
    ));
    assert_eq!(serving.media_counters(), before);
    assert_eq!(
        serving
            .read_protection_observer()
            .snapshot()
            .protected_roots(),
        bound
    );
    drop(readers);
    serving.close();
}

fn nonzero(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).unwrap()
}
