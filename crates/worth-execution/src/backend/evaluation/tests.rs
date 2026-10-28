use super::*;
use crate::{CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig, LeaseRequest};
use std::{num::NonZeroUsize, sync::Mutex};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
struct Value {
    bytes: Vec<u8>,
    panic: bool,
}
impl ChargedBytes for Value {
    fn additional_charged_bytes(&self) -> u64 {
        self.bytes.capacity() as u64
    }
}
impl Drop for Value {
    fn drop(&mut self) {
        assert!(
            !self.panic,
            "destructor failure after actual memory refusal"
        );
    }
}
#[test]
fn measured_backing_refusal_and_panicking_cleanup_keep_request_ownership() {
    if crate::tests::fresh_process::ran_in_fresh_child(
        module_path!(),
        "measured_backing_refusal_and_panicking_cleanup_keep_request_ownership",
    ) {
        return;
    }
    let authority = ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
        max_workers: NonZeroUsize::new(1).unwrap(),
        charged_memory_bytes: Some(0),
    })
    .unwrap();
    for panic in [false, true] {
        let lease = authority
            .request_lease(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    ExecutionPosture::Serial,
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), Some(0), None),
                ),
                deadline: None,
                cancellation: CancellationToken::new(),
            })
            .unwrap();
        let limits = RunLimits::for_run(Some(&lease), true);
        let holds = Mutex::new(Vec::new());
        let (outcome, receipt, _) = evaluate(
            (),
            Some(&lease),
            &limits,
            None,
            Some(&holds),
            None,
            &|_, _: &mut KernelContext<'_, '_>| {
                Ok::<_, KernelFailure<()>>(Value {
                    bytes: vec![1; 8],
                    panic,
                })
            },
        );
        let (result, owner) = outcome.result.into_parts(&limits);
        assert_eq!(owner, Some(super::super::meter::NestedStopOwner::Request));
        if panic {
            assert!(matches!(result, Err(KernelFailure::Panic)));
            assert!(outcome.allocation_denial.is_none());
        } else {
            assert!(matches!(result, Err(KernelFailure::ResultCapacityExceeded)));
            assert!(outcome.allocation_denial.is_some());
        }
        assert!(receipt.is_none());
        assert!(holds.lock().unwrap().is_empty());
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
}

use crate::{ExecutionMap, MapKernelFailure, MapOutcome};

#[test]
fn keyless_values_materialize_variable_results_without_result_estimates() {
    if crate::tests::fresh_process::ran_in_fresh_child(
        module_path!(),
        "keyless_values_materialize_variable_results_without_result_estimates",
    ) {
        return;
    }
    use std::collections::BTreeMap;
    use worth_foundational::PartitionIdentity;
    let authority = ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
        max_workers: NonZeroUsize::new(2).unwrap(),
        charged_memory_bytes: Some(65_536),
    })
    .unwrap();
    let cancellation = crate::CancellationSource::new();
    let lease = authority
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(2).unwrap(), Some(65_536), None),
            ),
            deadline: None,
            cancellation: cancellation.token(),
        })
        .unwrap();
    let map = ExecutionMap::<_, ()>::from_keyless_values(BTreeMap::from([
        (PartitionIdentity::new(1), 3_u64),
        (PartitionIdentity::new(2), 8192_u64),
    ]))
    .unwrap();
    match map.run(Some(&lease), |count, _| {
        Ok::<_, MapKernelFailure<()>>("v".repeat(*count as usize))
    }) {
        MapOutcome::Complete { values, .. } => {
            assert_eq!(values, vec!["v".repeat(3), "v".repeat(8192)]);
        }
        MapOutcome::Stopped { reason, .. } => {
            panic!("actual variable-width native result was refused: {reason:?}")
        }
    }
    cancellation.cancel();
    let entered = std::sync::atomic::AtomicUsize::new(0);
    let stopped = map.run(Some(&lease), |_, _| {
        entered.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok::<_, MapKernelFailure<()>>(String::new())
    });
    assert!(
        matches!(stopped, MapOutcome::Stopped { completed_prefix, reason: crate::MapStop::Failure { cause: MapKernelFailure::Stop(crate::MapKernelStop::Cancelled), .. }, .. } if completed_prefix.is_empty())
    );
    assert_eq!(entered.load(std::sync::atomic::Ordering::SeqCst), 0);
    assert_eq!(lease.charged_memory_bytes(), 0);
}
