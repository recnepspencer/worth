use tempfile::tempdir;
use worth_foundational::FoundationalPerformanceWorkClass;
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalEffectObligation, PhysicalSchedulerDemand, PhysicalSchedulerDenial,
    PhysicalWorkOperationFamily, PhysicalWorkPreEffectDenial, PhysicalWorkReadiness,
};
use worth_store_io_scheduler::{
    admit_secure_io_scope_for_scheduler, admit_security_scope_for_scheduler,
    foreground_reservation::ForegroundIoLaneKind, IoSchedulerBackendCapabilityAdmission,
    QueueDurabilityClass, QueueExecutionAdmissionDenial, SecureIoOperation,
    SecureIoPreservationRequest,
};

use super::fixture::{
    foreground_saturation_fixture, serving_from_initialization_with_work_profile, work_fixture,
};

mod locality;
mod policy_evidence;

use policy_evidence::mismatched_policy_receipt;
pub(crate) use policy_evidence::{exhausted_policy_receipt, policy_receipt, policy_receipt_for};

#[test]
fn ready_work_lowers_exact_budget_and_admits_without_effects() {
    let root = tempdir().unwrap();
    let (profile, _, mutation_request) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let before = serving.media_counters();
    let ready = ready_work(&serving, mutation_request);
    let demand =
        PhysicalSchedulerDemand::foreground(ready, super::reserved_page_write(&serving), None)
            .unwrap();
    let work = demand.queue_work();
    assert_eq!(work.durability_class(), QueueDurabilityClass::BufferedWrite);
    assert_eq!(work.requested_budget().queue_slots(), 1);
    assert_eq!(work.requested_budget().bandwidth_tokens(), 8);
    let backend_requirement = work.backend_requirement();
    let requested_budget = work.requested_budget();
    let backend = serving
        .admit_physical_scheduler_capability(backend_requirement)
        .unwrap();
    let demand = secure_demand(demand, &backend);
    let admitted = serving
        .admit_physical_scheduler_demand(demand, &backend, policy_receipt(requested_budget))
        .unwrap();
    assert_eq!(admitted.queue_plan().admitted_budget(), requested_budget);
    let grouping = admitted.queue_plan().grouping_basis();
    assert_eq!(
        grouping.security_scope_identity(),
        admitted.intent().security()
    );
    assert_eq!(
        grouping.durability_class(),
        QueueDurabilityClass::BufferedWrite
    );
    assert_eq!(grouping.flush_epoch(), 0);
    assert!(grouping.locality().is_some());
    assert_eq!(serving.media_counters(), before);
    serving.close();
}

#[test]
fn budget_mismatch_preserves_the_scheduler_denial() {
    let root = tempdir().unwrap();
    let (profile, _, mutation_request) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let ready = ready_work(&serving, mutation_request);
    let demand =
        PhysicalSchedulerDemand::foreground(ready, super::reserved_page_write(&serving), None)
            .unwrap();
    let work = demand.queue_work();
    let backend_requirement = work.backend_requirement();
    let requested_budget = work.requested_budget();
    let backend = serving
        .admit_physical_scheduler_capability(backend_requirement)
        .unwrap();
    let demand = secure_demand(demand, &backend);
    assert!(matches!(
        serving.admit_physical_scheduler_demand(
            demand,
            &backend,
            mismatched_policy_receipt(requested_budget)
        ),
        Err(PhysicalSchedulerDenial::Queue(
            QueueExecutionAdmissionDenial::PolicyReceiptBudgetMismatch { .. }
        ))
    ));
    serving.close();
}

#[test]
fn policy_receipt_for_planning_cannot_admit_authoritative_physical_io() {
    let root = tempdir().unwrap();
    let (profile, _, mutation_request) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let demand = write_demand(&serving, ready_work(&serving, mutation_request));
    let work = demand.queue_work();
    let backend_requirement = work.backend_requirement();
    let requested_budget = work.requested_budget();
    let backend = serving
        .admit_physical_scheduler_capability(backend_requirement)
        .unwrap();
    let demand = secure_demand(demand, &backend);

    assert!(matches!(
        serving.admit_physical_scheduler_demand(
            demand,
            &backend,
            policy_receipt_for(
                requested_budget,
                0,
                FoundationalPerformanceWorkClass::ValidationPlanning,
            ),
        ),
        Err(PhysicalSchedulerDenial::Queue(
            QueueExecutionAdmissionDenial::PolicyReceiptContextMismatch {
                expected_work: FoundationalPerformanceWorkClass::AuthoritativeMutation,
            },
        ))
    ));
    serving.close();
}

#[test]
fn operation_family_cannot_be_laundered_into_an_incompatible_lane() {
    let root = tempdir().unwrap();
    let (profile, read_request, _) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let ready = ready_read_work(&serving, read_request);

    assert!(matches!(
        PhysicalSchedulerDemand::foreground(ready, super::reserved_page_write(&serving), None,),
        Err(PhysicalSchedulerDenial::ForegroundLaneMismatch {
            operation: PhysicalWorkOperationFamily::ArtifactRangeRead,
            lane: ForegroundIoLaneKind::OrdinaryPageWrite,
        })
    ));
    serving.close();
}

#[test]
fn cancelled_ready_work_is_denied_before_scheduler_demand_admission() {
    let root = tempdir().unwrap();
    let (profile, _, mutation_request) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let ready = ready_work(&serving, mutation_request);
    let consumer = ready.consumer_handle();
    let capacity_before = serving.physical_scheduler_capacity();
    serving
        .advance_physical_signal_clock(
            consumer,
            worth_signal::facade::ClockAdvanceRequest::new(
                worth_signal::facade::ClockDomain::MonotonicExecution,
                worth_signal::facade::ClockTick::new(1_000),
            ),
        )
        .unwrap();
    let timeout = serving.timeout_physical_work(consumer).unwrap();

    assert_eq!(
        timeout.obligation(),
        PhysicalEffectObligation::NotDispatched
    );
    assert!(matches!(
        PhysicalSchedulerDemand::foreground(ready, super::reserved_page_write(&serving), None,),
        Err(PhysicalSchedulerDenial::PreEffect(
            PhysicalWorkPreEffectDenial::ConsumerCancelled
        ))
    ));
    let capacity_after = serving.physical_scheduler_capacity();
    assert_eq!(capacity_after.active_reservations(), 0);
    assert_eq!(capacity_after.available(), capacity_before.available());
    assert_eq!(
        capacity_after.admitted_reservations(),
        capacity_before.admitted_reservations() + 1
    );
    assert_eq!(
        capacity_after.released_reservations(),
        capacity_before.released_reservations() + 1
    );
    serving.close();
}

#[test]
fn disjoint_ready_work_admits_independently_and_a_denial_does_not_mutate_admitted_plans() {
    let root = tempdir().unwrap();
    let (profile, requests) = foreground_saturation_fixture();
    let [first_request, second_request, third_request, _] = requests;
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let first = ready_work(&serving, first_request);
    let second = ready_work(&serving, second_request);
    let third = ready_work(&serving, third_request);
    let before_media = serving.media_counters();
    let first_demand = write_demand(&serving, first);
    let second_demand = write_demand(&serving, second);
    let third_demand = write_demand(&serving, third);
    let budget = first_demand.queue_work().requested_budget();
    assert_eq!(second_demand.queue_work().requested_budget(), budget);
    assert_eq!(third_demand.queue_work().requested_budget(), budget);
    let backend = serving
        .admit_physical_scheduler_capability(first_demand.queue_work().backend_requirement())
        .unwrap();
    let first_demand = secure_demand(first_demand, &backend);
    let second_demand = secure_demand(second_demand, &backend);
    let third_demand = secure_demand(third_demand, &backend);

    let start = std::sync::Barrier::new(3);
    let (first, second) = std::thread::scope(|scope| {
        let first_start = &start;
        let second_start = &start;
        let first_serving = &serving;
        let second_serving = &serving;
        let first_backend = &backend;
        let second_backend = &backend;
        let first = scope.spawn(move || {
            first_start.wait();
            first_serving.admit_physical_scheduler_demand(
                first_demand,
                first_backend,
                policy_receipt(budget),
            )
        });
        let second = scope.spawn(move || {
            second_start.wait();
            second_serving.admit_physical_scheduler_demand(
                second_demand,
                second_backend,
                policy_receipt(budget),
            )
        });
        start.wait();
        (
            first.join().unwrap().unwrap(),
            second.join().unwrap().unwrap(),
        )
    });
    let first_identity = first.intent().identity();
    let second_identity = second.intent().identity();
    assert_ne!(first_identity, second_identity);
    assert!(matches!(
        serving.admit_physical_scheduler_demand(
            third_demand,
            &backend,
            mismatched_policy_receipt(budget),
        ),
        Err(PhysicalSchedulerDenial::Queue(
            QueueExecutionAdmissionDenial::PolicyReceiptBudgetMismatch { .. }
        ))
    ));
    assert_eq!(first.intent().identity(), first_identity);
    assert_eq!(second.intent().identity(), second_identity);
    assert_eq!(first.queue_plan().admitted_budget(), budget);
    assert_eq!(second.queue_plan().admitted_budget(), budget);
    assert_ne!(
        first.queue_plan().grouping_basis().locality(),
        second.queue_plan().grouping_basis().locality(),
        "disjoint physical scopes must remain distinct scheduler locality"
    );
    assert_eq!(
        first
            .queue_plan()
            .grouping_basis()
            .locality()
            .unwrap()
            .relation(second.queue_plan().grouping_basis().locality().unwrap()),
        worth_store_io_scheduler::QueueLocalityRelation::Adjacent
    );
    assert_eq!(serving.media_counters(), before_media);
    serving.close();
}

#[test]
fn a_scheduler_demand_cannot_cross_store_owners() {
    let first_root = tempdir().unwrap();
    let second_root = tempdir().unwrap();
    let (profile, _, mutation_request) = work_fixture();
    let first = serving_from_initialization_with_work_profile(first_root.path(), profile.clone());
    let second = serving_from_initialization_with_work_profile(second_root.path(), profile);
    let demand = write_demand(&first, ready_work(&first, mutation_request));
    let work = demand.queue_work();
    let backend_requirement = work.backend_requirement();
    let requested_budget = work.requested_budget();
    let backend = second
        .admit_physical_scheduler_capability(backend_requirement)
        .unwrap();
    let before = second.media_counters();

    assert!(matches!(
        second.admit_physical_scheduler_demand(demand, &backend, policy_receipt(requested_budget)),
        Err(PhysicalSchedulerDenial::PreEffect(
            worth_store::physical_runtime::PhysicalWorkPreEffectDenial::ForeignStore
        ))
    ));
    assert_eq!(second.media_counters(), before);
    first.close();
    second.close();
}

pub(super) fn write_demand(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    ready: worth_store::physical_runtime::ReadyPhysicalWork,
) -> PhysicalSchedulerDemand {
    PhysicalSchedulerDemand::foreground(ready, super::reserved_page_write(serving), None).unwrap()
}

pub(super) fn secure_demand(
    demand: PhysicalSchedulerDemand,
    backend: &IoSchedulerBackendCapabilityAdmission,
) -> PhysicalSchedulerDemand {
    let work = demand.queue_work();
    let admitted_scope = worth_store_security::admitted_security_scope_for_identity_for_test(
        work.security_scope_identity(),
    );
    let scheduler_security = admit_security_scope_for_scheduler(&admitted_scope)
        .expect("the physical-work fixture uses the scheduler's Store-internal security scope");
    let budget = work.requested_budget();
    let operation = if budget.read_ahead_window() > 0 {
        SecureIoOperation::ReadAhead
    } else if budget.write_back_window() > 0 {
        SecureIoOperation::WriteBack
    } else {
        SecureIoOperation::BatchedWrite
    };
    let secure_io = admit_secure_io_scope_for_scheduler(SecureIoPreservationRequest::new(
        operation,
        &scheduler_security,
        backend,
    ))
    .expect("the physical-work fixture binds secure I/O to its exact scope and backend");
    demand.with_secure_io(secure_io)
}

pub(crate) fn ready_work(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    request: worth_store::physical_runtime::PhysicalMutationWorkRequest,
) -> worth_store::physical_runtime::ReadyPhysicalWork {
    let receipt = match serving
        .physical_mutation_submission()
        .submit(request)
        .into_raw()
    {
        TransitionOutcome::Success(receipt) => receipt,
        outcome => panic!("physical work should declare: {outcome:?}"),
    };
    let admitted = serving.admit_physical_work(receipt).unwrap();
    match serving.request_physical_work(admitted).unwrap() {
        PhysicalWorkReadiness::Ready(ready) => ready,
        PhysicalWorkReadiness::Blocked(blocked) => {
            panic!(
                "physical work unexpectedly blocked: {:?}",
                blocked.condition()
            )
        }
    }
}

pub(super) fn ready_read_work(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    request: worth_store::physical_runtime::PhysicalReadWorkRequest,
) -> worth_store::physical_runtime::ReadyPhysicalWork {
    let receipt = match serving
        .physical_read_submission()
        .submit(request)
        .into_raw()
    {
        TransitionOutcome::Success(receipt) => receipt,
        outcome => panic!("physical work should declare: {outcome:?}"),
    };
    let admitted = serving.admit_physical_work(receipt).unwrap();
    match serving.request_physical_work(admitted).unwrap() {
        PhysicalWorkReadiness::Ready(ready) => ready,
        PhysicalWorkReadiness::Blocked(blocked) => {
            panic!(
                "physical work unexpectedly blocked: {:?}",
                blocked.condition()
            )
        }
    }
}
