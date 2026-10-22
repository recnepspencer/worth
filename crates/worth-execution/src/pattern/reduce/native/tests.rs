//! Native storage uses actual canonical values, never a heap-size invoice.
use super::*;
use crate::{CancellationSource, ReductionDenial, SerialMemoryBudget, SerialRequest};
use std::collections::BTreeMap;

mod worker_only;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Text(String);
impl CanonicalBits for Text {
    fn canonical_len(&self) -> Option<usize> {
        Some(self.0.len())
    }
    fn visit_canonical_bits(&self, visit: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        visit(self.0.as_bytes())
    }
}
fn combine(a: &Text, b: &Text) -> Text {
    Text(format!("({},{})", a.0, b.0))
}
fn id(value: u64) -> PartitionIdentity {
    PartitionIdentity::new(value)
}
fn serial_request(cancellation: &CancellationSource) -> SerialRequest {
    SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None)
}
fn plan(ids: &[u64]) -> ReductionPlan {
    ReductionPlan::try_from_sorted_unique(ids.iter().copied().map(id).collect()).unwrap()
}

#[test]
fn native_canonical_map_keeps_real_cells_and_edits_match_fresh_association_without_quotes() {
    let cancellation = CancellationSource::new();
    let serial = serial_request(&cancellation);
    let request = ExecutionRequest::serial(&serial);
    let supplied = BTreeMap::from([(id(1), 1), (id(3), 3), (id(4), 4)]);
    let map = ExecutionMap::from_native_partitions(supplied).unwrap();
    let outcome = map.run_reduce_retained_native(
        request,
        ExecutionWorkCeiling::new(None),
        |value, context| {
            context.poll()?;
            Ok::<_, KernelFailure<std::convert::Infallible>>(Text(value.to_string().repeat(8192)))
        },
        Text("_".into()),
        combine as fn(&Text, &Text) -> Text,
    );
    let mut completed = outcome
        .result
        .unwrap_or_else(|cause| panic!("native completion: {cause:?}"));
    let run = completed
        .run
        .as_ref()
        .expect("actual keyless map completion");
    assert_eq!(run.partition_count(), 3);
    assert!(run.result(id(3)).unwrap().native_reuse_available());
    assert_eq!(
        completed.tree.leaf(id(3)),
        Some(run.result(id(3)).unwrap().value())
    );
    request
        .run_native(|lease| {
            completed
                .tree
                .update_native(id(3), Text("changed".into()), lease, || Ok::<_, ()>(()))
                .unwrap();
            completed
                .tree
                .insert_native(id(2), Text("inserted".into()), lease, || Ok::<_, ()>(()))
                .unwrap();
            completed
                .tree
                .delete_native(id(4), lease, || Ok::<_, ()>(()))
                .unwrap();
            let leaves = [1, 2, 3]
                .map(|key| (id(key), completed.tree.leaf(id(key)).unwrap().clone()))
                .to_vec();
            let (fresh, _) = ReductionTree::try_from_declared_native(
                plan(&[1, 2, 3]),
                leaves,
                Text("_".into()),
                combine,
                lease,
                || Ok::<_, ()>(()),
            )
            .unwrap();
            assert_eq!(completed.tree.result(), fresh.result());
            let unchanged = completed
                .tree
                .update_native(id(2), Text("inserted".into()), lease, || Ok::<_, ()>(()))
                .unwrap();
            assert_eq!(unchanged.accounted_metrics().unwrap().combine_calls, 0);
        })
        .unwrap();
}

#[test]
fn native_cancelled_edit_keeps_old_root_and_fresh_request_can_retry() {
    let cancellation = CancellationSource::new();
    let serial = serial_request(&cancellation);
    let request = ExecutionRequest::serial(&serial);
    let (mut tree, _) = request
        .run(ExecutionWorkCeiling::new(None), |lease| {
            ReductionTree::try_from_declared_native(
                plan(&[1]),
                vec![(id(1), Text("old".into()))],
                Text("_".into()),
                combine,
                lease,
                || Ok::<_, ()>(()),
            )
            .unwrap()
        })
        .unwrap()
        .0;
    let old = tree.result().clone();
    let stopped = request.run(ExecutionWorkCeiling::new(None), |lease| {
        let scope = crate::backend::run_scope_native_within(lease, true, None, None, |context| {
            let mut calls = 0;
            let result = tree.update_native(id(1), Text("new".into()), lease, || {
                calls += 1;
                if calls == 2 {
                    cancellation.cancel();
                }
                context.checkpoint(1)
            });
            assert_eq!(
                result.unwrap_err().reason,
                ReductionRunStop::Hook(KernelStop::Cancelled)
            );
            Ok::<_, KernelFailure<()>>(())
        });
        scope.result
    });
    assert!(matches!(
        stopped.unwrap().0,
        Err(crate::backend::ScopeStop::Failure(KernelFailure::Stop(
            KernelStop::Cancelled
        )))
    ));
    assert_eq!(tree.result(), &old);
    let fresh_cancellation = CancellationSource::new();
    let fresh_serial = serial_request(&fresh_cancellation);
    ExecutionRequest::serial(&fresh_serial)
        .run(ExecutionWorkCeiling::new(None), |lease| {
            tree.update_native(id(1), Text("new".into()), lease, || Ok::<_, ()>(()))
                .unwrap()
        })
        .unwrap();
    assert_ne!(tree.result(), &old);
}

#[derive(Clone)]
struct Broken {
    length: usize,
}
impl CanonicalBits for Broken {
    fn canonical_len(&self) -> Option<usize> {
        Some(self.length)
    }
    fn visit_canonical_bits(&self, visit: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        visit(&[1])
    }
}

#[test]
fn native_storage_preserves_physical_layout_and_original_admission_causes() {
    let cancellation = CancellationSource::new();
    let serial = serial_request(&cancellation);
    let layout = ExecutionRequest::serial(&serial)
        .run_native(|lease| {
            ReductionTree::try_from_declared_native(
                plan(&[]),
                vec![],
                Broken { length: usize::MAX },
                |_, _| Broken { length: usize::MAX },
                lease,
                || Ok::<_, ()>(()),
            )
            .err()
            .unwrap()
        })
        .unwrap()
        .0;
    assert_eq!(
        layout.reason,
        ReductionRunStop::Denial(ReductionDenial::Allocation(
            crate::ExecutionPhysicalAllocationDenialKind::Layout
        ))
    );
    let admission = ReductionTree::try_from_declared_native(
        plan(&[]),
        vec![],
        Text("identity".into()),
        combine,
        None,
        || Ok::<_, ()>(()),
    )
    .err()
    .unwrap();
    assert_eq!(
        admission.reason,
        ReductionRunStop::Denial(ReductionDenial::Admission(
            LeaseDenial::NoActiveExecutionScope
        ))
    );
    let malformed = ExecutionRequest::serial(&serial)
        .run(ExecutionWorkCeiling::new(None), |lease| {
            ReductionTree::try_from_declared_native(
                plan(&[]),
                vec![],
                Broken { length: 2 },
                |_, _| Broken { length: 2 },
                lease,
                || Ok::<_, ()>(()),
            )
            .err()
            .unwrap()
        })
        .unwrap()
        .0;
    assert_eq!(
        malformed.reason,
        ReductionRunStop::Denial(ReductionDenial::InvalidCanonicalEncoding)
    );
}

#[test]
fn native_node_admission_refusal_keeps_root_and_releases_temporary_holds() {
    let cancellation = CancellationSource::new();
    let budget = SerialMemoryBudget::new(Some(16_384));
    let serial = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let request = ExecutionRequest::serial(&serial);
    let (mut tree, _) = request
        .run(ExecutionWorkCeiling::new(None), |lease| {
            ReductionTree::try_from_declared_native(
                plan(&[1]),
                vec![(id(1), Text("old".into()))],
                Text("_".into()),
                combine,
                lease,
                || Ok::<_, ()>(()),
            )
            .unwrap()
        })
        .unwrap()
        .0;
    let original = tree.result().clone();
    let held = budget.charged_memory_bytes();
    let refused = request.run(ExecutionWorkCeiling::new(None), |lease| {
        let _full = budget
            .reserve(budget.limit().unwrap() - budget.charged_memory_bytes())
            .unwrap();
        tree.update_native(id(1), Text("new".into()), lease, || Ok::<_, ()>(()))
    });
    assert!(matches!(
        refused.unwrap().0.unwrap_err().reason,
        ReductionRunStop::Denial(ReductionDenial::Admission(LeaseDenial::MemoryExhausted(
            crate::MemoryLimitDenial::Capacity { admitted: 0, .. }
        )))
    ));
    assert_eq!(tree.result(), &original);
    assert_eq!(budget.charged_memory_bytes(), held);
    request
        .run(ExecutionWorkCeiling::new(None), |lease| {
            tree.update_native(id(1), Text("new".into()), lease, || Ok::<_, ()>(()))
                .unwrap()
        })
        .unwrap();
    assert_ne!(tree.result(), &original);
    drop(tree);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
