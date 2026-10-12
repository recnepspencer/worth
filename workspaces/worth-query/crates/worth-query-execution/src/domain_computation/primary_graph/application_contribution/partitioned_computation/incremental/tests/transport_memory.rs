//! Incremental transport reserves the caller-thread build it can perform.
use super::*;
use worth_execution::{ReductionTree, SerialMemoryBudget};

#[test]
fn an_incremental_transport_fits_its_serial_tree_capacity_exactly() {
    let _observation = QueryObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let installed = installed(StatusRead::Gather(1), sum);
    let first = first_run(&world, &installed);
    let request = live_scope();
    let mut quoted = None;
    world.application.with_application_advancement(&request, |phase| {
        let admission = projected_attempt::admit(&world, &request, &phase);
        let custody = phase.test_host_request();
        ComputationPrior::hand_in_test(Some(prior_of(first, false)));
        world.invariant.project_admitted_operation(
            &custody.phase_for_application(&world.application), &admission,
            |reader, root| {
                let execution = phase.execution_for(&world.application.product_runtime).unwrap();
                let prepared = installed.prepare_through(reader, execution, root).unwrap();
                let super::super::super::compute::PreparedRun::Incremental(incremental) = &prepared.run else {
                    panic!("unchanged producer state prepares an incremental run");
                };
                let declared = Computation::RESOURCES.maximum_retained_bytes() as u64;
                let tree = ReductionTree::<u64, fn(&u64, &u64) -> u64>::checked_serial_build_memory_bound(
                    incremental.partition_count(), declared).unwrap();
                let expected = prepared.transport_bytes() + tree + declared;
                let capacity = prepared.result_capacity();
                quoted = Some((expected, capacity));
            }, worth_execution::ExecutionAllocationPolicy::SystemAllocation).unwrap();
        ComputationPrior::hand_in_test(None);
    }).unwrap();
    let (expected, capacity) = quoted.expect("the actual prepared transport was observed");
    for short in [0, 1] {
        let budget = SerialMemoryBudget::new(expected - short);
        let reservation = budget.reserve(capacity);
        if short == 0 {
            drop(reservation.expect("incremental serial tree capacity fits exactly"));
        } else {
            let denial = reservation.expect_err("one byte under the serial capacity refuses");
            assert_eq!(denial.requested, expected);
            assert_eq!(denial.admitted, expected - 1);
        }
        assert_eq!(budget.charged_memory_bytes(), 0);
    }
}
