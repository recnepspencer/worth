use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};

use worth_query_declaration::facade::domain_computation::{
    WorthQueryCancellationSafePointFamily, WorthQueryExecutionMode,
    WorthQueryExecutionResourceRequest,
};
use worth_query_installation::facade::{
    WorthQueryExecutionAccessProductFamily, WorthQueryExecutionAllocatorFamily,
    WorthQueryExecutionProviderFamily, WorthQueryExecutionProviderRequirements,
    WorthQueryExecutionResourceContract, WorthQueryExecutionResourceEnvelope,
    WorthQueryExecutionStrategyContract, WorthQueryExecutionStrategyName,
};

use super::*;

#[test]
fn later_subject_failure_rolls_back_every_prior_reservation() {
    let first = Arc::new(ObservedCapacity::new("first", 1));
    let second = Arc::new(ObservedCapacity::new("second", 1));
    let blocker = second.try_reserve().unwrap();
    let plan = admitted(snapshot(first.clone(), second.clone()));

    assert!(reserve_execution_resource_plan(plan).is_none());
    assert_eq!(first.active(), 0);
    assert_eq!(second.active(), 1);

    drop(blocker);
    let reserved =
        reserve_execution_resource_plan(admitted(snapshot(first.clone(), second.clone()))).unwrap();
    assert_eq!((first.active(), second.active()), (1, 1));
    let plan_identity = reserved.resources().identity().to_owned();
    let receipt = reserved.release();
    assert_eq!(receipt.resource_plan_identity(), plan_identity);
    assert_eq!(
        receipt.scope(),
        WorthQueryExecutionCapacityReservationScope::Direct
    );
    assert_eq!(receipt.released_reservation_count(), 2);
    assert_eq!((first.active(), second.active()), (0, 0));
}

#[test]
fn workflow_release_disposes_each_exact_capacity_authority_once() {
    let capacity = Arc::new(ObservedCapacity::new("workflow", 1));
    let first = workflow_plan(capacity.clone());
    let blocked = workflow_plan(capacity.clone());
    let retry = workflow_plan(capacity.clone());

    let reserved = reserve_workflow_resource_plan(first).expect("workflow should reserve");
    assert!(reserve_workflow_resource_plan(blocked).is_none());

    let plan_identity = reserved.resources().identity().to_owned();
    let receipt = reserved.release();
    assert_eq!(receipt.resource_plan_identity(), plan_identity);
    assert_eq!(
        receipt.scope(),
        WorthQueryExecutionCapacityReservationScope::Workflow
    );
    assert_eq!(receipt.released_reservation_count(), 1);
    assert_eq!(capacity.active(), 0);
    assert!(reserve_workflow_resource_plan(retry).is_some());
}

#[test]
fn racing_reservations_never_exceed_the_physical_limit() {
    const THREADS: usize = 8;
    let capacity = Arc::new(ObservedCapacity::new("raced", 1));
    let plans = (0..THREADS)
        .map(|_| admitted(single_snapshot(capacity.clone())))
        .collect::<Vec<_>>();
    let start = Arc::new(Barrier::new(THREADS + 1));
    let held = Arc::new(Barrier::new(THREADS + 1));
    let successes = Arc::new(AtomicUsize::new(0));
    let workers = plans
        .into_iter()
        .map(|plan| {
            let start = Arc::clone(&start);
            let held = Arc::clone(&held);
            let successes = Arc::clone(&successes);
            std::thread::spawn(move || {
                start.wait();
                let reservation = reserve_execution_resource_plan(plan);
                if reservation.is_some() {
                    successes.fetch_add(1, Ordering::AcqRel);
                }
                held.wait();
                drop(reservation);
            })
        })
        .collect::<Vec<_>>();

    start.wait();
    held.wait();
    for worker in workers {
        worker.join().unwrap();
    }
    assert_eq!(successes.load(Ordering::Acquire), 1);
    assert_eq!(capacity.active(), 0);
}

#[test]
fn graph_reservation_admission_precedes_capacity_claim_and_release() {
    let capacity = Arc::new(ObservedCapacity::new("graph-admitted", 1));
    let support = support(capacity.clone());
    let refused = reserve_graph_provider_capacity_admitted(&support, &mut |_, _| Err("denied"));
    assert!(matches!(
        refused,
        Err(WorthQueryCapacityReservationAdmissionStop::Admission(
            "denied"
        ))
    ));
    assert_eq!(capacity.active(), 0);

    let mut work = 0_u64;
    let mut bytes = 0_u64;
    let reserved =
        reserve_graph_provider_capacity_admitted(&support, &mut |next_work, next_bytes| {
            work = work.checked_add(next_work).unwrap();
            bytes = bytes.checked_add(next_bytes).unwrap();
            Ok::<(), ()>(())
        })
        .unwrap()
        .expect("installed graph capacity admits one reservation");
    assert!(work > 0);
    assert!(bytes >= std::mem::size_of::<ObservedReservation>() as u64);
    assert_eq!(capacity.active(), 1);
    drop(reserved);
    assert_eq!(capacity.active(), 0);
}

struct ObservedCapacity {
    identity: String,
    limit: usize,
    active: Arc<AtomicUsize>,
}

impl ObservedCapacity {
    fn new(identity: &str, limit: usize) -> Self {
        Self {
            identity: identity.into(),
            limit,
            active: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn active(&self) -> usize {
        self.active.load(Ordering::Acquire)
    }
}

impl WorthQueryExecutionCapacityPort for ObservedCapacity {
    fn capacity_subject_identity(&self) -> &str {
        &self.identity
    }

    fn reservation_preflight_cost(&self) -> Option<(u64, u64)> {
        let backing = u64::try_from(std::mem::size_of::<ObservedReservation>()).ok()?;
        Some((backing.checked_add(2)?, backing))
    }

    fn try_reserve(&self) -> Option<Box<dyn WorthQueryExecutionCapacityReservation>> {
        self.active
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |active| {
                (active < self.limit).then_some(active + 1)
            })
            .ok()?;
        Some(Box::new(ObservedReservation(Arc::clone(&self.active))))
    }
}

struct ObservedReservation(Arc<AtomicUsize>);

impl Drop for ObservedReservation {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

fn admitted(
    support: WorthQueryExecutionResourceSupportSnapshot,
) -> WorthQueryAdmittedExecutionResourcePlan {
    admit_execution_resource_plan(
        "capacity-atomicity",
        &contract(),
        &WorthQueryExecutionResourceRequest::bounded(8, 8, safe_point()),
        support,
        WorthQueryExecutionResourceAdmissionCounters::default(),
    )
    .unwrap()
}

fn contract() -> WorthQueryExecutionResourceContract {
    WorthQueryExecutionResourceContract::declared([WorthQueryExecutionStrategyContract::new(
        WorthQueryExecutionStrategyName::new("capacity").unwrap(),
        envelope(),
        requirements(),
    )])
    .unwrap()
}

fn snapshot(
    first: Arc<ObservedCapacity>,
    second: Arc<ObservedCapacity>,
) -> WorthQueryExecutionResourceSupportSnapshot {
    WorthQueryExecutionResourceSupportSnapshot::new(
        support(first),
        Vec::new(),
        vec![("graph".into(), support(second))],
        Vec::new(),
        None,
    )
}

fn single_snapshot(capacity: Arc<ObservedCapacity>) -> WorthQueryExecutionResourceSupportSnapshot {
    WorthQueryExecutionResourceSupportSnapshot::new(
        support(capacity),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        None,
    )
}

fn workflow_plan(capacity: Arc<ObservedCapacity>) -> WorthQueryAdmittedWorkflowResourcePlan {
    let operation = admitted(single_snapshot(capacity.clone()));
    let stage = admitted(single_snapshot(capacity));
    WorthQueryAdmittedWorkflowResourcePlan::assemble(
        operation,
        [("stage".to_owned(), stage)].into_iter().collect(),
    )
}

fn support(capacity: Arc<ObservedCapacity>) -> WorthQueryExecutionResourceSupport {
    WorthQueryExecutionResourceSupport::new(
        requirements().provider().clone(),
        requirements().access_product().clone(),
        requirements().allocator().clone(),
        envelope(),
        capacity,
    )
}

fn requirements() -> WorthQueryExecutionProviderRequirements {
    WorthQueryExecutionProviderRequirements::new(
        WorthQueryExecutionProviderFamily::new("provider").unwrap(),
        WorthQueryExecutionAccessProductFamily::new("access").unwrap(),
        WorthQueryExecutionAllocatorFamily::new("allocator").unwrap(),
    )
}

fn envelope() -> WorthQueryExecutionResourceEnvelope {
    WorthQueryExecutionResourceEnvelope::bounded(
        8,
        8,
        WorthQueryExecutionMode::Synchronous,
        safe_point(),
    )
}

fn safe_point() -> WorthQueryCancellationSafePointFamily {
    WorthQueryCancellationSafePointFamily::new("capacity-chunk").unwrap()
}
