//! Partition-level evidence includes nested kernels, never just member windows.
use super::super::{cases, control, model, World};
pub(super) fn assert_kernel_observations(
    control: &control::Control,
    world: &World,
    expected: &model::Advancement,
    cell: cases::Cell,
    failure: Option<(u64, control::Phase)>,
) {
    if cell.family == 7 {
        let meters = control.nested_meters.lock().unwrap();
        let mut measured = meters
            .iter()
            .map(|meter| {
                let member = world
                    .members
                    .iter()
                    .find(|member| member.key == meter.member)
                    .unwrap();
                assert_eq!(
                    meter.cost,
                    Some((
                        u128::from(member.partitions.iter().map(|part| part.work).sum::<u64>()),
                        u128::from(
                            member
                                .partitions
                                .iter()
                                .map(|part| part.work)
                                .max()
                                .unwrap()
                        )
                    )),
                    "nested kernels charge their declared sum and critical path by value"
                );
                meter.member
            })
            .collect::<Vec<_>>();
        measured.sort_unstable();
        let mut healthy = expected
            .dispatched
            .iter()
            .copied()
            .filter(|key| failure != Some((*key, control::Phase::Compute)))
            .collect::<Vec<_>>();
        healthy.sort_unstable();
        assert_eq!(
            measured, healthy,
            "every completed nested map has one measured kernel cost"
        );
        let declared = control.partition_declarations.lock().unwrap();
        let mut keys = declared
            .iter()
            .map(|(key, parts)| {
                let expected = if cell.declaration == 1 {
                    vec![3, 2, 1, 0]
                } else {
                    vec![0, 1, 2, 3]
                };
                assert_eq!(
                    *parts, expected,
                    "a nested member declares all four partitions in one map"
                );
                *key
            })
            .collect::<Vec<_>>();
        keys.sort_unstable();
        let mut expected = expected.dispatched.clone();
        expected.sort_unstable();
        assert_eq!(
            keys, expected,
            "one nested map declaration per dispatched computing member"
        );
    }
    let state = control.state.lock().unwrap();

    assert_eq!(
        state.kernel_active, 0,
        "every partition kernel releases its in-flight witness"
    );
    // Certified cells replay the owner on a detached diagnostic worker
    // ledger. Those kernels may overlap ordinary sibling kernels, so this
    // shared fixture counter is their aggregate, not one request's width.
    // Uncertified cells retain the exact request bound, including nested work.
    if cell.dispatch == cases::UNSEEDED {
        assert_kernel_width(state.kernel_peak, cell.workers);
    }
    if cell.family == 7 && cell.workers > 1 && !expected.witnesses.is_empty() {
        assert!(
            state.kernel_peak >= 2,
            "nested parallel rows observe overlapping partition kernels"
        );
    }
}

fn assert_kernel_width(peak: usize, workers: usize) {
    assert!(
        !kernel_width_exceeded(peak, workers),
        "partition kernels in flight never exceed the named worker count"
    );
}
fn kernel_width_exceeded(peak: usize, workers: usize) -> bool {
    peak > workers
}

#[test]
fn four_inner_kernels_cannot_hide_in_one_member_window() {
    let world = super::super::seeded_world::world(super::super::Family::Empty, super::super::SEED);
    let control = control::Control::new(world, false, false, 0, true);
    control.state.lock().unwrap().armed = true;
    control.dispatched(8);
    let mut flights = Vec::new();
    for part in 0..4 {
        control.enter_compute(8, part, 1);
        flights.push(control::KernelFlight(&control));
    }
    let measured = {
        let state = control.state.lock().unwrap();
        (state.peak, state.kernel_peak)
    };
    assert_eq!(
        measured,
        (1, 4),
        "one member window contains four kernel lifetimes"
    );
    let peak = control.state.lock().unwrap().kernel_peak;
    assert!(
        kernel_width_exceeded(peak, 1),
        "four inner kernels exceed a one-worker lease despite one member window"
    );
    drop(flights);
    assert_eq!(
        control.state.lock().unwrap().kernel_active,
        0,
        "kernel lifetimes release individually"
    );
}
