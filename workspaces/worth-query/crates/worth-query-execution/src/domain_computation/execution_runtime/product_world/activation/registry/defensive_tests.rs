use super::*;
use crate::domain_computation::execution_runtime::product_world::test_product_world_resources;
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchCreationDenial as Creation;
use crate::facade::primary_graph::WorthQueryBranchAdoptionActivationDenial as Adoption;

#[test]
fn defensive_gate_overflow_and_allocation_refund_the_live_reservation() {
    let limit = test_product_world_resources()
        .budgets()
        .live_product_branches();
    for (reserved, expected) in [
        (
            usize::MAX,
            WorthQueryProductActivationDenial::OccupancyOverflow,
        ),
        (
            usize::MAX - 1,
            WorthQueryProductActivationDenial::AllocationRejected,
        ),
    ] {
        let registry = WorthQueryProductActivationRegistry::new(limit).unwrap();
        let maximum = limit.get().unwrap();
        let held = (1..maximum)
            .map(|_| registry.reserve().unwrap())
            .collect::<Vec<_>>();
        // These secondary counters are bounded by live capacity in consistent
        // operation. Inject desynchronization to reach their defensive producers.
        registry.state.lock().unwrap().reserved_gates = reserved;
        let produced = registry
            .reserve_for_source_program(None)
            .err()
            .expect("the actual registry reservation must be refused");
        assert_eq!(produced, expected);
        let creation = Creation::from(produced);
        assert!(
            matches!(
                (expected, &creation),
                (
                    WorthQueryProductActivationDenial::OccupancyOverflow,
                    Creation::CoordinationOccupancyOverflow
                ) | (
                    WorthQueryProductActivationDenial::AllocationRejected,
                    Creation::CoordinationAllocationRejected
                )
            ),
            "creation must preserve its producer: {creation:?}"
        );
        let adoption = Adoption::from(produced);
        assert!(
            matches!(
                (expected, &adoption),
                (
                    WorthQueryProductActivationDenial::OccupancyOverflow,
                    Adoption::OccupancyOverflow
                ) | (
                    WorthQueryProductActivationDenial::AllocationRejected,
                    Adoption::AllocationRejected
                )
            ),
            "adoption must preserve its producer: {adoption:?}"
        );
        assert_eq!(registry.state.lock().unwrap().reserved_gates, reserved);
        registry.state.lock().unwrap().reserved_gates = held.len();
        let last = registry
            .reserve_for_source_program(None)
            .expect("a defensive map refusal must refund the admitted live slot");
        assert!(
            matches!(registry.reserve(), Err(WorthQueryProductActivationDenial::CapacityExhausted { limit, occupancy }) if (limit, occupancy) == (maximum, maximum))
        );
        drop((last, held));
        assert_eq!(registry.state.lock().unwrap().reserved_gates, 0);
        assert!(registry.live_occurrences().unwrap().is_empty());
    }
}
