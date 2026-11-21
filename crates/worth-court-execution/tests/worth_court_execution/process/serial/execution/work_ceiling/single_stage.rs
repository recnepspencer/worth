//! A request's declared single stage: its own narrowing-free nested stages join it.
use worth_execution::{
    CancellationSource, ChargedBytes, ExecutionMemoryReservation, ExecutionRequest,
    ExecutionScopeSetupCountsForTest as Setup, ExecutionScopeSetupObservationForTest,
    ExecutionWorkCeiling, MapKernelStop, ScopeStop, SerialRequest, WorkCeilingDenial,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

fn serial(memory: Option<u64>, work: Option<u64>) -> SerialRequest {
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, memory, work),
    );
    SerialRequest::from_policy(&policy, CancellationSource::new().token(), None)
}

fn stages(setups: usize) -> Setup {
    Setup {
        stage_setups: setups,
        limit_derivations: setups,
        admissions: setups,
    }
}

struct Kept(u64);
impl ChargedBytes for Kept {
    fn additional_charged_bytes(&self) -> u64 {
        self.0
    }
}

#[test]
fn nested_stages_of_the_request_join_its_single_stage_and_charge_it_once() {
    let backing = serial(Some(1 << 20), Some(100));
    let request = ExecutionRequest::serial(&backing);
    let observer = ExecutionScopeSetupObservationForTest::install_on_this_thread_for_test();
    let (work, _) = request
        .run_single_stage(|stage| {
            stage.checkpoint(3).unwrap();
            let ((), _) = request
                .run_native_with_checkpoint(|inner| inner.checkpoint(4).unwrap())
                .unwrap();
            request.in_scope(|_| ()).unwrap();
            let charged = || backing.memory().unwrap().charged_memory_bytes();
            let before = charged();
            let held = request.reserve_retained_memory(5).unwrap();
            assert_eq!(charged(), before + 5, "no stage framework beside the bytes");
            let mut memory = ExecutionMemoryReservation::reserve_in_scope(None, 0).unwrap();
            let outcome = request.run_measured_result::<_, Kept>(
                ExecutionWorkCeiling::new(request.work_ceiling()),
                0,
                &mut memory,
                |inner, result| {
                    inner.checkpoint(2).unwrap();
                    result.resize(9).unwrap();
                    Ok(Kept(7))
                },
            );
            assert_eq!(outcome.report.charged_work(), 2);
            assert!(outcome.result.is_ok());
            assert_eq!(memory.bytes(), 7, "the measured value keeps its heap");
            drop((held, memory));
            stage.charged_work()
        })
        .unwrap();
    assert_eq!(work, 9, "every joined charge lands on the one stage");
    assert_eq!(observer.counts_for_test(), stages(1));
    assert_eq!(backing.memory().unwrap().charged_memory_bytes(), 0);
}

#[test]
fn a_joined_charge_past_the_ceiling_stops_the_single_stage_with_its_own_cause() {
    let backing = serial(None, Some(5));
    let request = ExecutionRequest::serial(&backing);
    let outcome = request.run_single_stage(|stage| {
        stage.checkpoint(3).unwrap();
        let inner = request.run_native_with_checkpoint(|inner| inner.checkpoint(3));
        assert_eq!(
            inner.map(|(charged, _)| charged),
            Err(WorkCeilingDenial::Stopped(MapKernelStop::WorkCeiling))
        );
        stage.charged_work()
    });
    assert_eq!(
        outcome.map(|(work, _)| work),
        Err(WorkCeilingDenial::Stopped(MapKernelStop::WorkCeiling))
    );
}

#[test]
fn another_request_or_a_narrower_ceiling_keeps_its_own_stage() {
    let backing = serial(Some(1 << 20), Some(100));
    let other = serial(Some(1 << 20), Some(100));
    let request = ExecutionRequest::serial(&backing);
    let observer = ExecutionScopeSetupObservationForTest::install_on_this_thread_for_test();
    request
        .run_single_stage(|_| {
            ExecutionRequest::serial(&other)
                .run_native_with_checkpoint(|inner| inner.checkpoint(1).unwrap())
                .unwrap();
            let mut memory = ExecutionMemoryReservation::reserve_in_scope(None, 0).unwrap();
            let narrower = request.run_measured_result::<_, Kept>(
                ExecutionWorkCeiling::new(Some(1)),
                0,
                &mut memory,
                |_, _| Ok(Kept(0)),
            );
            assert!(narrower.result.is_ok());
        })
        .unwrap();
    assert_eq!(observer.counts_for_test(), stages(3));
}

#[test]
fn outside_a_single_stage_nested_stages_are_set_up_as_before() {
    let backing = serial(Some(1 << 20), Some(100));
    let request = ExecutionRequest::serial(&backing);
    let observer = ExecutionScopeSetupObservationForTest::install_on_this_thread_for_test();
    request
        .run_native_with_checkpoint(|_| {
            request
                .run_native_with_checkpoint(|inner| inner.checkpoint(1).unwrap())
                .unwrap();
        })
        .unwrap();
    assert_eq!(observer.counts_for_test(), stages(2));
}

fn lease_request(policy: ExecutionRequestPolicy) -> worth_execution::LeaseRequest {
    worth_execution::LeaseRequest {
        policy,
        deadline: None,
        cancellation: worth_execution::CancellationToken::new(),
    }
}

#[test]
fn a_leased_single_stage_joins_its_own_nested_stages() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(1 << 16), Some(100)),
    );
    let lease = crate::work_ceiling::ordinary_context::authority()
        .request_lease(lease_request(policy))
        .unwrap();
    let request = ExecutionRequest::leased(&lease);
    let observer = ExecutionScopeSetupObservationForTest::install_on_this_thread_for_test();
    let (work, _) = request
        .run_single_stage(|stage| {
            stage.checkpoint(3).unwrap();
            request
                .run_native_with_checkpoint(|inner| inner.checkpoint(4).unwrap())
                .unwrap();
            let held = request.reserve_retained_memory(5).unwrap();
            let mut memory = ExecutionMemoryReservation::reserve_in_scope(Some(&lease), 0).unwrap();
            let outcome = request.run_measured_result::<_, Kept>(
                ExecutionWorkCeiling::new(request.work_ceiling()),
                0,
                &mut memory,
                |inner, _| {
                    inner.checkpoint(2).unwrap();
                    Ok(Kept(7))
                },
            );
            assert!(outcome.result.is_ok());
            assert_eq!(memory.bytes(), 7);
            drop((held, memory));
            stage.charged_work()
        })
        .unwrap();
    assert_eq!(work, 9);
    assert_eq!(observer.counts_for_test(), stages(1));
    assert_eq!(lease.charged_memory_bytes(), 0);
}

#[test]
fn a_foreign_reservation_is_refused_and_stops_the_single_stage() {
    let backing = serial(Some(1 << 20), Some(100));
    let request = ExecutionRequest::serial(&backing);
    let mut ran = false;
    let outcome = request.run_single_stage(|_| {
        let mut foreign = worth_execution::SerialMemoryBudget::new(None)
            .reserve(0)
            .unwrap();
        let refused = request.run_measured_result::<_, Kept>(
            ExecutionWorkCeiling::new(None),
            0,
            &mut foreign,
            |_, _| {
                ran = true;
                Ok(Kept(0))
            },
        );
        assert!(matches!(refused.result, Err(ScopeStop::Admission(_))));
    });
    assert!(!ran, "a refused reservation never runs its kernel");
    assert!(
        outcome.is_err(),
        "a swallowed refusal still stops the stage"
    );
}

#[test]
fn a_refused_payload_releases_its_reservation_and_stops_the_single_stage() {
    let backing = serial(Some(1 << 12), Some(100));
    let request = ExecutionRequest::serial(&backing);
    let outcome = request.run_single_stage(|_| {
        let mut memory = ExecutionMemoryReservation::reserve_in_scope(None, 0).unwrap();
        let refused = request.run_measured_result::<_, Kept>(
            ExecutionWorkCeiling::new(None),
            0,
            &mut memory,
            |_, _| Ok(Kept(1 << 20)),
        );
        assert!(matches!(refused.result, Err(ScopeStop::Admission(_))));
        assert_eq!(memory.bytes(), 0);
    });
    assert!(outcome.is_err());
    assert_eq!(backing.memory().unwrap().charged_memory_bytes(), 0);
}

#[test]
fn a_joined_panic_is_contained_and_releases_its_result_memory() {
    let backing = serial(Some(1 << 20), Some(100));
    let request = ExecutionRequest::serial(&backing);
    let outcome = request.run_single_stage(|_| {
        let native = request.run_native_with_checkpoint(|_| panic!("joined native kernel"));
        assert_eq!(native.err(), Some(WorkCeilingDenial::Panicked));
        let mut memory = ExecutionMemoryReservation::reserve_in_scope(None, 0).unwrap();
        let measured = request.run_measured_result::<Kept, Kept>(
            ExecutionWorkCeiling::new(None),
            0,
            &mut memory,
            |_, result| {
                result.resize(100).unwrap();
                panic!("joined measured kernel")
            },
        );
        assert!(matches!(
            measured.result,
            Err(ScopeStop::Failure(worth_execution::MapKernelFailure::Panic))
        ));
        assert_eq!(memory.bytes(), 0);
    });
    assert!(outcome.is_err(), "a contained panic stops the stage");
    assert_eq!(backing.memory().unwrap().charged_memory_bytes(), 0);
}

#[test]
fn a_latched_stop_refuses_a_joined_stage_before_its_effect() {
    let backing = serial(None, Some(5));
    let request = ExecutionRequest::serial(&backing);
    let mut ran = false;
    let outcome = request.run_single_stage(|stage| {
        assert_eq!(stage.checkpoint(6), Err(MapKernelStop::WorkCeiling));
        let refused = request.run_native(|_| ran = true);
        assert_eq!(
            refused.err(),
            Some(WorkCeilingDenial::Stopped(MapKernelStop::WorkCeiling))
        );
    });
    assert!(!ran, "a stopped stage runs no joined effect");
    assert!(outcome.is_err());
}
