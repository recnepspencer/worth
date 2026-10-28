//! Swallowing a real original-request stop cannot publish a component rule.
use super::super::*;
use crate::facade::application_invariants::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
#[test]
fn component_reads_preserve_original_cancellation_and_live_twin() {
    let _isolation = super::super::super::super::fixture::isolated_request_owner();
    for cancel in [false, true] {
        let source = WorthQueryCancellationSource::new();
        let cancellation = source.clone();
        let calls = Arc::new(AtomicUsize::new(0));
        let seen = calls.clone();
        let observer = probe::Probe::<false>::new(move |context, scope, rule| {
            let before = context.reads();
            if cancel {
                cancellation.cancel();
            }
            let view = context.proposed();
            let of = view.component_of(&rule.shells, scope.node(0));
            let count = view.component_count(&rule.shells);
            if cancel {
                for refusal in [of.err().unwrap(), count.err().unwrap()] {
                    assert_eq!(
                        refusal.kind(),
                        WorthQueryInvariantAccessDenialKind::Interrupted(
                            CustomInvariantInterruption::Cancelled
                        )
                    );
                }
                assert_eq!(
                    context.reads(),
                    before,
                    "stop precedes business acquisition"
                );
            } else {
                assert_eq!(of.unwrap().len(), 3);
                assert_eq!(count.unwrap(), 3);
            }
            seen.fetch_add(1, Ordering::SeqCst);
            // Intentionally swallow the read stop; the original owner must veto publication.
        })
        .complete_components();
        let world = world::install_with_probe(super::census::graph(), observer.clone());
        let request = WorthQueryRequestScope::new(
            Some(std::time::Instant::now() + std::time::Duration::from_secs(60)),
            source.token(),
        );
        let operation = program::swap_successors(&world, &request, 0, 1);
        let before = world.committed_version();
        observer.arm();
        let outcome = commit(&world, &request, operation, 196 + cancel as u8);
        if cancel {
            let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
                panic!("original cancellation must deny publication")
            };
            assert_eq!(
                denial.execution_denial_cause(),
                Some(Err(
                    crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled
                ))
            );
            assert_eq!(world.committed_version(), before);
        } else {
            assert!(matches!(
                outcome,
                WorthQueryApplicationCommitOutcome::Committed(_)
            ));
        }
        assert!(calls.load(Ordering::SeqCst) > 0);
    }
}
