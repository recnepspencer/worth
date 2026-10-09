//! Repeated ordered effects must retain geometric native backing.

use super::*;

#[test]
fn repeated_single_effect_merges_reuse_capacity_and_bound_prefix_copying() {
    const FINAL_COUNT: usize = 128;
    let (_directory, _media, coordination) = recovery_world();
    let ports = coordination
        .sampling_allocation_basis()
        .unwrap()
        .0
        .ports()
        .clone();
    let window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let slot = size_of::<SelectedArtifactSlice>() as u64;
    let mut destination = funded(&window, 0);
    let mut expected = vec![slice(0)];
    let mut replacements = 0;
    let mut copied_prefix_entries = 0;
    for ordinal in 1..FINAL_COUNT {
        let previous_length = destination.slices.len();
        let previous_capacity = destination.slices.capacity();
        let previous_pointer = destination.slices.as_ptr();
        let previous_charge = destination.charged_bytes();
        let donor = funded(&window, ordinal as u64);
        assert_eq!(
            ports.counters().active_operation_bytes_for(Scope::Recovery),
            previous_charge + slot,
            "the source and donor are both native-funded before merge"
        );
        destination.merge(donor).unwrap();
        expected.push(slice(ordinal as u64));
        assert_eq!(destination.slices(), expected.as_slice());
        let capacity = destination.slices.capacity();
        let actual_backing = (capacity as u64) * slot;
        assert_eq!(destination.owned_heap_bytes(), Some(actual_backing));
        assert_eq!(destination.charged_bytes(), actual_backing);
        assert_eq!(
            ports.counters().active_operation_bytes_for(Scope::Recovery),
            actual_backing,
            "the donor charge is released after each merge"
        );
        if previous_length < previous_capacity {
            assert_eq!(destination.slices.as_ptr(), previous_pointer);
            assert_eq!(capacity, previous_capacity);
            assert_eq!(destination.charged_bytes(), previous_charge);
        } else {
            replacements += 1;
            copied_prefix_entries += previous_length;
            assert!(capacity >= previous_capacity.saturating_mul(2));
        }
    }
    assert!(
        replacements <= 7,
        "128 single additions permit at most seven doublings"
    );
    assert!(
        copied_prefix_entries < 2 * FINAL_COUNT,
        "geometric replacements copy fewer than twice the final roster"
    );
    drop(window);
    drop(coordination);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        destination.charged_bytes()
    );
    drop(destination);
    assert_eq!(ports.counters().active_operation_bytes(), 0);
}

#[test]
fn bounded_fingerprint_preflights_geometric_effect_backing() {
    use super::super::super::SelectedControlMediaFingerprint;

    let (_directory, _media, coordination) = recovery_world();
    let ports = coordination
        .sampling_allocation_basis()
        .unwrap()
        .0
        .ports()
        .clone();
    let events = ports.allocation_events();
    let window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let slot = size_of::<SelectedArtifactSlice>() as u64;
    let mut effects = funded(&window, 0);
    for ordinal in 1..4 {
        effects.merge(funded(&window, ordinal)).unwrap();
    }
    assert_eq!(effects.slices.capacity(), 4);
    assert_eq!(effects.charged_bytes(), 4 * slot);
    let mut fingerprint = SelectedControlMediaFingerprint::observed_head_effect(effects);
    assert_eq!(fingerprint.slices.len(), 0);
    assert_eq!(fingerprint.slices.capacity(), 0);
    let donor = SelectedControlMediaFingerprint::observed_head_effect(funded(&window, 4));
    let before = events.snapshot();
    assert!(matches!(
        fingerprint.try_extend_bounded(donor, 4 * 5 * slot),
        Err(Denial::BoundExceeded)
    ));
    let after = events.snapshot();
    for dimension in [
        Dimension::OperationScope(Scope::Recovery),
        Dimension::OperationBytes,
        Dimension::TotalBytes,
    ] {
        assert_eq!(
            after.for_dimension(dimension).admitted_units(),
            before.for_dimension(dimension).admitted_units(),
            "the insufficient logical-five limit must deny before native growth"
        );
    }
    assert_eq!(fingerprint.slices.len(), 0);
    assert_eq!(fingerprint.slices.capacity(), 0);
    let retained = fingerprint.effects.as_ref().unwrap();
    assert_eq!(retained.slices.capacity(), 4);
    assert_eq!(retained.slices(), &[slice(0), slice(1), slice(2), slice(3)]);
    assert_eq!(retained.charged_bytes(), 4 * slot);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        4 * slot
    );

    let donor = SelectedControlMediaFingerprint::observed_head_effect(funded(&window, 4));
    fingerprint.try_extend_bounded(donor, 4 * 8 * slot).unwrap();
    let retained = fingerprint.effects.as_ref().unwrap();
    assert_eq!(retained.slices.capacity(), 8);
    assert_eq!(
        retained.slices(),
        &[slice(0), slice(1), slice(2), slice(3), slice(4)]
    );
    assert_eq!(retained.owned_heap_bytes(), Some(8 * slot));
    assert_eq!(retained.charged_bytes(), 8 * slot);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        8 * slot
    );
    drop(fingerprint);
    assert_eq!(ports.counters().active_operation_bytes(), 0);
}
