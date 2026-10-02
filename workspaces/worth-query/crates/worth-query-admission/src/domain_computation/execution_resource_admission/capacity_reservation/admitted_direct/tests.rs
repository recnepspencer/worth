use std::sync::Arc;

use super::*;
use crate::domain_computation::execution_resource_admission::{
    admit_execution_resource_plan, tests, WorthQueryExecutionResourceSupportSnapshot,
    WorthQueryFixedExecutionCapacity,
};

#[test]
fn late_parent_refusal_releases_every_already_acquired_provider_capacity() {
    let capacities = ["executor", "graph", "commit"]
        .map(|identity| Arc::new(WorthQueryFixedExecutionCapacity::new(identity, 1).unwrap()));
    let plan = resources(&capacities);
    let marker = Arc::new(());
    let mut reached_live_reservation = false;
    let result = reserve_execution_resource_plan_admitted(plan, &mut |_, _| {
        if capacities[0].active_attempts() == 1 {
            reached_live_reservation = true;
            return Err(Arc::clone(&marker));
        }
        Ok(())
    });
    match result {
        Err(WorthQueryCapacityReservationAdmissionStop::Admission(original)) => {
            assert!(Arc::ptr_eq(&marker, &original))
        }
        _ => panic!("the original parent refusal must survive partial reservation"),
    }
    assert!(reached_live_reservation);
    assert!(capacities
        .iter()
        .all(|capacity| capacity.active_attempts() == 0));
    let retried = reserve_execution_resource_plan_admitted(resources(&capacities), &mut |_, _| {
        Ok::<(), ()>(())
    })
    .unwrap()
    .expect("refused custody has released the actual pools");
    assert_eq!(retried.reservation_count(), 3);
    assert!(capacities
        .iter()
        .all(|capacity| capacity.active_attempts() == 1));
    let identity = retried.resources().identity().to_owned();
    let released = retried.release();
    assert_eq!(released.resource_plan_identity(), identity);
    assert_eq!(released.released_reservation_count(), 3);
    assert!(capacities
        .iter()
        .all(|capacity| capacity.active_attempts() == 0));
}

fn resources(
    capacities: &[Arc<WorthQueryFixedExecutionCapacity>; 3],
) -> WorthQueryAdmittedExecutionResourcePlan {
    let [executor, graph, commit] = capacities.each_ref().map(|capacity| {
        WorthQueryExecutionResourceSupport::new(
            tests::provider(),
            tests::access(),
            tests::allocator(),
            tests::envelope(8),
            Arc::clone(capacity) as Arc<dyn super::super::super::WorthQueryExecutionCapacityPort>,
        )
    });
    let support = WorthQueryExecutionResourceSupportSnapshot::new(
        executor,
        Vec::new(),
        vec![("primary".to_owned(), graph)],
        vec![("primary".to_owned(), commit)],
        None,
    );
    admit_execution_resource_plan(
        "invocation",
        &tests::contract(8),
        &tests::request(8),
        support,
        Default::default(),
    )
    .unwrap()
}
