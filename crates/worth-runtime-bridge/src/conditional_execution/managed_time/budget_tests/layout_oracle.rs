//! The test oracle enumerates the retained fields/containers independently of
//! production charge helpers or ledger readings. The BTree bound is the installed
//! runtime's documented maximum-node representation, not observed allocator RSS.
use super::super::super::retention::BridgeRetentionReservation;
use super::super::*;
use std::alloc::Layout;
use std::mem::{align_of, size_of};
use std::sync::{Arc, Mutex};

pub(super) fn arc<T>() -> u64 {
    Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
        .extend(Layout::new::<T>())
        .unwrap()
        .0
        .pad_to_align()
        .size() as u64
}

pub(super) fn text(bytes: usize) -> u64 {
    Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
        .extend(Layout::array::<u8>(bytes).unwrap())
        .unwrap()
        .0
        .pad_to_align()
        .size() as u64
}

fn tree<K, V>(count: usize) -> u64 {
    let alignment = align_of::<K>()
        .max(align_of::<V>())
        .max(align_of::<usize>());
    ((11 * size_of::<K>()
        + 11 * size_of::<V>()
        + 13 * size_of::<usize>()
        + 2 * size_of::<u16>()
        + 5 * alignment)
        * (count + 1)) as u64
}

pub(super) fn clock(wakes: usize) -> u64 {
    arc::<Mutex<BridgeManagedClockLane>>()
        + arc::<()>()
        + size_of::<Arc<Mutex<BridgeManagedClockLane>>>() as u64
        + tree::<Arc<str>, Arc<Mutex<BridgeManagedClockLane>>>(1)
        + tree::<
            BridgeManagedTemporalIntentIdentity,
            super::super::clock_lane::BridgeManagedTemporalIntentRecord,
        >(wakes)
        + tree::<worth_signal::facade::TemporalWakeId, BridgeManagedTemporalIntentIdentity>(wakes)
        + 2 * text(512) * wakes as u64
}

pub(super) fn binding() -> u64 {
    arc::<super::super::contract::BridgeManagedClockLease>() + 3 * text(512)
}

pub(super) fn baseline() -> u64 {
    arc::<super::super::super::observation_retention::BridgeObservationBaselines>()
}

pub(super) fn due(wakes: usize) -> u64 {
    arc::<BridgeRetentionReservation>()
        + size_of::<BridgeManagedDueWake>() as u64 * wakes as u64
        + 3 * text(512) * wakes as u64
}

pub(super) fn core(snapshot: &str, execution: &str) -> u64 {
    arc::<super::super::super::retained_decision::BridgeRetainedConditionalDecisionCore>()
        + text(snapshot.len())
        + text(execution.len())
}

pub(super) fn evidence(binding: &str) -> u64 {
    size_of::<super::super::super::BridgeConditionalDecisionEvidence>() as u64 + text(binding.len())
}

pub(super) fn decision(snapshot: &str, execution: &str, binding: &str) -> u64 {
    core(snapshot, execution) + evidence(binding)
}

pub(super) fn reentry(binding: &str) -> u64 {
    evidence(binding)
}
