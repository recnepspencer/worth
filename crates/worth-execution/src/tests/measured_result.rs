mod drop_order;

use crate::{
    CancellationSource, CancellationToken, ChargedBytes, ExecutionRequest, ExecutionResourceLease,
    ExecutionWorkCeiling, LeaseDenial, LeaseRequest, MapKernelFailure, MapKernelStop,
    MemoryLimitDenial, ScopeStop, SerialRequest,
};
use std::num::NonZeroUsize;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

fn policy(memory: Option<u64>) -> ExecutionRequestPolicy {
    ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, memory, None),
    )
}

#[test]
fn measured_results_keep_the_original_serial_and_lease_charge_until_release() {
    let serial = SerialRequest::from_policy(&policy(None), CancellationToken::new(), None);
    let lease = ExecutionResourceLease::serial(LeaseRequest {
        policy: policy(None),
        cancellation: CancellationToken::new(),
        deadline: None,
    })
    .unwrap();
    for leased in [false, true] {
        let request = if leased {
            ExecutionRequest::leased(&lease)
        } else {
            ExecutionRequest::serial(&serial)
        };
        let mut memory = if leased {
            lease.reserve_memory(0)
        } else {
            serial
                .memory()
                .expect("explicit accounted request")
                .reserve(0)
        }
        .unwrap();
        let measured = request.run_measured_result(
            ExecutionWorkCeiling::new(None),
            0,
            &mut memory,
            |context, result_memory| {
                context.checkpoint(3)?;
                result_memory.resize(32_768).unwrap();
                Ok::<_, MapKernelFailure<()>>(vec![17_u8; 32_768])
            },
        );
        let value = measured.result.unwrap();
        assert!(value.iter().all(|byte| *byte == 17));
        assert_eq!(measured.report.charged_work(), 3);
        assert!(measured.report.physical().peak_charged_memory_bytes() > 32_768);
        assert_eq!(memory.bytes(), value.additional_charged_bytes());
        let charged = || {
            if leased {
                lease.charged_memory_bytes()
            } else {
                serial
                    .memory()
                    .expect("explicit accounted request")
                    .charged_memory_bytes()
            }
        };
        assert_eq!(charged(), 32_768);
        drop(value);
        assert_eq!(charged(), 32_768, "the actual reservation is still held");
        drop(memory);
        assert_eq!(charged(), 0);
    }
}

#[test]
fn measured_opaque_result_admission_keeps_a_finite_parent_and_previous_result() {
    let serial =
        SerialRequest::from_policy(&policy(Some(128 * 1024)), CancellationToken::new(), None);
    let request = ExecutionRequest::serial(&serial);
    let mut first = serial
        .memory()
        .expect("explicit accounted request")
        .reserve(0)
        .unwrap();
    let first_value = request
        .run_measured_result(ExecutionWorkCeiling::new(None), 0, &mut first, |_, _| {
            Ok::<_, MapKernelFailure<()>>(vec![5_u8; 96 * 1024])
        })
        .result
        .unwrap();
    let original_charge = serial
        .memory()
        .expect("explicit accounted request")
        .charged_memory_bytes();
    let mut second = serial
        .memory()
        .expect("explicit accounted request")
        .reserve(0)
        .unwrap();
    let rejected =
        request.run_measured_result(ExecutionWorkCeiling::new(None), 0, &mut second, |_, _| {
            Ok::<_, MapKernelFailure<()>>(vec![9_u8; 96 * 1024])
        });
    assert!(matches!(
        rejected.result,
        Err(ScopeStop::Admission(LeaseDenial::MemoryExhausted(
            MemoryLimitDenial::Capacity { .. }
        )))
    ));
    assert_eq!(second.bytes(), 0);
    assert_eq!(
        serial
            .memory()
            .expect("explicit accounted request")
            .charged_memory_bytes(),
        original_charge
    );
    assert_eq!(first_value, vec![5_u8; 96 * 1024]);
    drop(first_value);
    drop(first);
    assert_eq!(
        serial
            .memory()
            .expect("explicit accounted request")
            .charged_memory_bytes(),
        0
    );
}

#[test]
fn measured_domain_error_remains_funded_and_cancellation_releases_its_scope() {
    let cancellation = CancellationSource::new();
    let serial = SerialRequest::from_policy(&policy(None), cancellation.token(), None);
    let request = ExecutionRequest::serial(&serial);
    let mut memory = serial
        .memory()
        .expect("explicit accounted request")
        .reserve(0)
        .unwrap();
    let stopped =
        request.run_measured_result(ExecutionWorkCeiling::new(None), 0, &mut memory, |_, _| {
            Err::<(), _>(MapKernelFailure::Domain(vec![3_u8; 1_024]))
        });
    let Err(ScopeStop::Failure(MapKernelFailure::Domain(error))) = stopped.result else {
        panic!("original domain error")
    };
    assert_eq!(memory.bytes(), error.additional_charged_bytes());
    assert_eq!(error, vec![3_u8; 1_024]);
    drop(error);
    memory.resize(0).unwrap();
    let stopped = request.run_measured_result(
        ExecutionWorkCeiling::new(None),
        0,
        &mut memory,
        |context, _| {
            cancellation.cancel();
            context.checkpoint(0)?;
            Ok::<_, MapKernelFailure<()>>([7_u8])
        },
    );
    assert!(matches!(
        stopped.result,
        Err(ScopeStop::Failure(MapKernelFailure::Stop(
            MapKernelStop::Cancelled
        )))
    ));
    assert_eq!(memory.bytes(), 0);
    assert_eq!(
        serial
            .memory()
            .expect("explicit accounted request")
            .charged_memory_bytes(),
        0
    );
}

#[test]
fn measured_scope_cannot_return_success_after_a_swallowed_byte_counter_stop() {
    let serial = SerialRequest::from_policy(&policy(None), CancellationToken::new(), None);
    let mut memory = serial
        .memory()
        .expect("explicit accounted request")
        .reserve(0)
        .unwrap();
    let stopped = ExecutionRequest::serial(&serial).run_measured_result(
        ExecutionWorkCeiling::new(None),
        0,
        &mut memory,
        |_, result_memory| {
            assert!(matches!(
                result_memory.resize(u64::MAX),
                Err(MemoryLimitDenial::CounterOverflow { .. })
            ));
            Ok::<_, MapKernelFailure<()>>(())
        },
    );
    assert!(matches!(
        stopped.result,
        Err(ScopeStop::Failure(MapKernelFailure::Stop(
            MapKernelStop::NestedStopped
        )))
    ));
    assert_eq!(memory.bytes(), 0);
    assert_eq!(
        serial
            .memory()
            .expect("explicit accounted request")
            .charged_memory_bytes(),
        0
    );
}
