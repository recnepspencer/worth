//! Independent reservation oracle: the test declares its buffer counts and
//! state layouts rather than calling the production reservation arithmetic.
use crate::{CancellationToken, MapKernelFailure, MapKernelStop, SerialMemoryBudget};
use std::{
    mem::size_of,
    sync::{
        atomic::{AtomicU64, AtomicUsize},
        Arc, Mutex,
    },
    time::Instant,
};
use worth_foundational::PartitionIdentity;

// These are size declarations, not runtime authority or executable meters.
#[allow(dead_code)]
struct LimitsLayout {
    tokens: Vec<CancellationToken>,
    deadline: Option<Instant>,
    ceiling: u64,
    workers: Arc<[AtomicUsize; 2]>,
    physical: Arc<[AtomicU64; 3]>,
    flags: [bool; 3],
    serial_memory: Option<SerialMemoryBudget>,
}
#[allow(dead_code)]
struct MeterLayout {
    limits: LimitsLayout,
    work: u64,
    span: u64,
    nested_stopped: bool,
    checkpoint_stop: Option<MapKernelStop>,
}
#[allow(dead_code)]
struct OutcomeLayout<R, E> {
    result: Result<R, MapKernelFailure<E>>,
    work: u64,
    span: u64,
}

pub(super) fn bytes<T, R, E>(
    items: usize,
    heap: u64,
    access: u64,
    tokens: usize,
    workers: usize,
    owns_physical: bool,
) -> u64 {
    // Every fixture declares zero scratch and result heap capacity.
    let input_and_output = size_of::<T>() + size_of::<R>();
    let identity_and_capacity = size_of::<PartitionIdentity>() + size_of::<u64>();
    let scheduling = size_of::<usize>() + size_of::<Mutex<Option<OutcomeLayout<R, E>>>>();
    let lineage = tokens * size_of::<CancellationToken>();
    let allocation_header = 2 * size_of::<usize>();
    let kernel = size_of::<MeterLayout>() + allocation_header + lineage + size_of::<(usize, u64)>();
    let per_item = input_and_output + identity_and_capacity + scheduling + kernel;
    let worker_state = size_of::<[AtomicUsize; 2]>() + allocation_header;
    let worker_holds = workers * size_of::<crate::authority::ResourceReservation>();
    let physical_state = if owns_physical {
        size_of::<[AtomicU64; 3]>() + allocation_header
    } else {
        0
    };
    (items * per_item + lineage + worker_state + worker_holds + physical_state) as u64
        + heap
        + access
}
