//! External interruption belongs to the caller, never the admitted Query scope.
use super::*;
use crate::facade::primary_graph::WorthQueryApplicationOneShotDenialKind;
#[test]
fn caller_cancellation_and_deadline_at_each_root_preserve_a_canonical_boundary() {
    for deadline in [false, true] {
        for index in 0..3 {
            for workers in [None, Some(1), Some(2), Some(4)] {
                let observed = observe_case(
                    workers,
                    3,
                    false,
                    false,
                    &[],
                    Case {
                        interrupt: Some((index, deadline)),
                        ..Case::default()
                    },
                );
                assert_eq!(
                    observed.denial,
                    Some(if deadline {
                        WorthQueryManagedDerivedViewDenial::DeadlineElapsed
                    } else {
                        WorthQueryManagedDerivedViewDenial::Cancelled
                    })
                );
                let boundary = observed.map.1;
                let point = observed.interruption_point.as_ref().unwrap();
                assert_eq!(point.root, observed.roots[index]);
                assert!(point.fresh);
                assert!(observed.roots[..index]
                    .iter()
                    .all(|root| point.completed.contains(root)));
                // The witness waits for whole predecessor kernels, so even a
                // wider dispatch cannot stop before this constructed boundary.
                assert_eq!(
                    boundary, index,
                    "the selected kernel determines the computed prefix"
                );
                // The owner rechecks the interrupted request before projection.
                assert!(observed.values.is_empty());
                assert!(observed.dependencies.is_empty());
                // Independent ledger of actually accepted kernel checkpoints.
                // Canonical settlement charges earlier roots and the stopping
                // root's admitted units; speculative later roots are excluded.
                let admitted = observed
                    .roots
                    .iter()
                    .take(boundary + 1)
                    .map(|root| observed.map.2.get(root).copied().unwrap_or(0))
                    .sum::<u64>();
                assert_eq!(
                    observed.map.0, admitted,
                    "interruption settlement must retain exactly the admitted canonical work"
                );
            }
        }
    }
}
#[test]
fn query_scope_interruption_at_finalization_does_not_interrupt_worker_reads() {
    for (count, interruption) in [
        (1, ScopeInterruption::Cancel),
        (3, ScopeInterruption::Cancel),
        (3, ScopeInterruption::Deadline),
    ] {
        for workers in [None, Some(1), Some(2), Some(4)] {
            let serial = observe(workers, count, false, false, &[]);
            let ignored_scope = observe_case(
                workers,
                count,
                false,
                false,
                &[],
                Case {
                    query_interruption: Some(interruption),
                    ..Case::default()
                },
            );
            assert_eq!(
                ignored_scope.map, serial.map,
                "a prepared worker has no Query scope safe point or clock"
            );
            assert_eq!(ignored_scope.map.1, count);
            assert!(ignored_scope.interruption_point.as_ref().unwrap().fresh);
            assert!(ignored_scope.values.is_empty());
            // Owner finalization validates its admitted Query scope, separately
            // from interruption of the caller's worker request.
            match ignored_scope.denial {
                Some(WorthQueryManagedDerivedViewDenial::ReadDenied { denial, .. }) => {
                    assert_eq!(
                        denial.kind(),
                        match interruption {
                            ScopeInterruption::Cancel =>
                                WorthQueryApplicationOneShotDenialKind::Cancelled,
                            ScopeInterruption::Deadline =>
                                WorthQueryApplicationOneShotDenialKind::DeadlineExceeded,
                        }
                    );
                }
                cause => panic!("owner finalization must report its scope refusal: {cause:?}"),
            }
        }
    }
}
