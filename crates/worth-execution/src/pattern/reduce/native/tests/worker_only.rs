//! The ordinary reducer has controls and canonical values, without a cost ledger.
use super::*;
use crate::{ExecutionAuthority, ExecutionAuthorityConfig, ExecutionResourceLease, LeaseRequest};
use std::num::NonZeroUsize;
use worth_foundational::{DeterminismContract, ExecutionPosture, ExecutionRequestPolicy};

fn workers() -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            NonZeroUsize::new(2).unwrap(),
        ),
        cancellation: crate::CancellationToken::new(),
        deadline: None,
    }
}

fn worker_serial(cancellation: &CancellationSource) -> SerialRequest {
    SerialRequest::from_policy(&workers().policy, cancellation.token(), None)
}

// This physical-authority case runs in its own process, like the existing
// canonical cancellation fixtures: process construction is genuinely unique.
#[test]
fn worker_only_native_reduction_certifies_shared_scheduled_engine_without_accounting() {
    let authority = ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
        max_workers: NonZeroUsize::new(2).unwrap(),
        charged_memory_bytes: None,
    })
    .unwrap();
    let lease = authority.request_lease(workers()).unwrap();
    let request = ExecutionRequest::leased(&lease);
    let map = ExecutionMap::<_, ()>::from_native_partitions(BTreeMap::from([
        (id(1), 1),
        (id(3), 3),
        (id(4), 4),
    ]))
    .unwrap();
    request
        .run_native(|active| {
            let (tree, report, metrics) = map
                .certify_reduce_native(
                    active.unwrap(),
                    17,
                    |value, context| {
                        assert!(!context.has_accounting());
                        context.poll()?;
                        Ok::<_, KernelFailure<()>>(Text(value.to_string()))
                    },
                    Text("_".into()),
                    combine,
                )
                .unwrap();
            assert!(!tree.result().0.is_empty());
            assert_eq!(tree.checked_full_build_cost(), None);
            assert_eq!(report.accounted_cost(), None);
            assert_eq!(metrics.accounted_metrics(), None);
            assert_eq!(report.physical().peak_charged_memory_bytes(), 0);
            assert!(report.physical().active_workers_high_watermark() >= 1);
        })
        .unwrap();
    assert_eq!(lease.charged_memory_bytes(), 0);
    assert!(
        lease.try_reserve_workers(2).is_ok(),
        "original worker claims release"
    );
}

#[test]
fn worker_only_native_edits_keep_absent_cost_and_original_cancelled_root() {
    let cancellation = CancellationSource::new();
    let serial = worker_serial(&cancellation);
    let request = ExecutionRequest::serial(&serial);
    let map = ExecutionMap::<_, ()>::from_native_partitions(BTreeMap::from([(id(1), 1)])).unwrap();
    let mut completed = map
        .run_reduce_retained_native(
            request,
            ExecutionWorkCeiling::new(None),
            |_, context| {
                context.poll()?;
                Ok::<_, KernelFailure<()>>(Text("old".into()))
            },
            Text("_".into()),
            combine,
        )
        .result
        .unwrap();
    assert_eq!(completed.report.accounted_cost(), None);
    assert_eq!(completed.metrics.accounted_metrics(), None);
    assert_eq!(completed.run.as_ref().unwrap().accounted_cost(), None);
    let old = completed.tree.result().clone();
    let stopped = request.run_native_with_checkpoint(|context| {
        let mut calls = 0;
        let failure = completed
            .tree
            .update_native(id(1), Text("new".into()), None, || {
                calls += 1;
                if calls == 2 {
                    cancellation.cancel();
                }
                context.poll()
            })
            .unwrap_err();
        assert_eq!(
            failure.reason,
            ReductionRunStop::Hook(KernelStop::Cancelled)
        );
        assert_eq!(failure.metrics.accounted_metrics(), None);
    });
    assert_eq!(
        stopped.unwrap_err(),
        crate::WorkCeilingDenial::Stopped(KernelStop::Cancelled)
    );
    assert_eq!(completed.tree.result(), &old);
    let fresh_cancel = CancellationSource::new();
    let fresh = worker_serial(&fresh_cancel);
    ExecutionRequest::serial(&fresh)
        .run_native_with_checkpoint(|context| {
            let metrics = completed
                .tree
                .update_native(id(1), Text("new".into()), None, || context.poll())
                .unwrap();
            assert_eq!(metrics.accounted_metrics(), None);
            assert_eq!(completed.tree.checked_full_build_cost(), None);
            let (oracle, _) = ReductionTree::try_from_declared_native(
                plan(&[1]),
                vec![(id(1), Text("new".into()))],
                Text("_".into()),
                combine,
                None,
                || context.poll(),
            )
            .unwrap();
            assert_eq!(completed.tree.result(), oracle.result());
        })
        .unwrap();
}

#[derive(Clone)]
struct InvalidLayout(bool);
impl CanonicalBits for InvalidLayout {
    fn canonical_len(&self) -> Option<usize> {
        Some(if self.0 { usize::MAX } else { 0 })
    }
    fn visit_canonical_bits(&self, _: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        !self.0
    }
}

#[test]
fn worker_only_native_reduction_keeps_original_physical_layout_refusal() {
    let lease = ExecutionResourceLease::serial(workers()).unwrap();
    let failure = ExecutionRequest::leased(&lease)
        .run_native(|active| {
            ReductionTree::try_from_declared_native(
                plan(&[1]),
                vec![(id(1), InvalidLayout(true))],
                InvalidLayout(false),
                |a: &InvalidLayout, _: &InvalidLayout| a.clone(),
                active,
                || Ok::<_, ()>(()),
            )
            .err()
            .expect("canonical layout cannot be allocated")
        })
        .unwrap()
        .0;
    assert_eq!(
        failure.reason,
        ReductionRunStop::Denial(ReductionDenial::Allocation(
            crate::ExecutionPhysicalAllocationDenialKind::Layout,
        ))
    );
    assert_eq!(failure.metrics.accounted_metrics(), None);
    assert_eq!(lease.charged_memory_bytes(), 0);
    assert!(lease.try_reserve_workers(1).is_ok());
}
