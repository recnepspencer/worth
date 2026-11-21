use crate::{
    CancellationSource, ChargedBytes, ExecutionMemoryReservation, ExecutionRequest,
    ExecutionScopeSetupCountsForTest as Setup, ExecutionScopeSetupObservationForTest,
    ExecutionWorkCeiling, MapKernelStop, SerialRequest, WorkCeilingDenial,
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
