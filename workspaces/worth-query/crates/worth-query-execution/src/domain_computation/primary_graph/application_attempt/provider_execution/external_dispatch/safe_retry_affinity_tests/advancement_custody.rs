//! Redispatch is observed at its real outbox reader and transport, never a query marker.
use super::*;
use crate::domain_computation::primary_graph::{
    bound_advancement_requests_on_this_thread_for_test as bound,
    installed_source_reads_on_this_thread_for_test as reads,
    place_managed_computations_on_this_thread_for_test as place,
    WorthQueryExecutionPlacementForTest as Placement,
    WorthQueryManagedComputationResourceDenial as Resource,
};
use std::num::NonZeroUsize;
struct Restore(Placement, Option<worth_foundational::ExecutionBudget>);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}
#[test]
fn redispatch_refuses_before_outbox_reads_and_transport_at_both_placements() {
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _restore = Restore(place(placement), bound(None));
        let (world, _, handle, admission) = real_handle(221, "redispatch-custody");
        let authority = authority(&world, &handle, &admission);
        let transport = Arc::new(CompletingTransport(AtomicUsize::new(0)));
        world
            .application
            .install_external_effect_transport(transport.clone())
            .unwrap();
        for memory in [false, true] {
            bound(Some(worth_foundational::ExecutionBudget::new(
                NonZeroUsize::MIN,
                if memory { 0 } else { 64 * 1024 * 1024 },
                if memory { 8_000_000 } else { 0 },
            )));
            let before = reads();
            let Err(WorthQueryExternalRedispatchDenial::CompletionExecutionDenied {
                kind:
                    crate::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionResource {
                        denial: cause,
                        ..
                    },
                ..
            }) = world
                .application
                .redispatch_admitted_external_effect(&handle, &authority, &admission)
            else {
                panic!("redispatch opening retains cause")
            };
            if !memory {
                assert_eq!(cause, Resource::WorkExhausted);
            } else if placement == Placement::Serial {
                assert_eq!(cause, Resource::PolicyMemoryLimit);
            } else {
                let Resource::MemoryLimit {
                    level,
                    requested,
                    admitted,
                } = cause
                else {
                    panic!("lease bytes")
                };
                assert_eq!(
                    level,
                    crate::domain_computation::primary_graph::WorthQueryMemoryLimitLevel::Policy
                );
                assert!(requested > 0);
                assert_eq!(admitted, 0);
            }
            assert_eq!(reads(), before);
            assert_eq!(transport.0.load(Ordering::Relaxed), 0);
        }
        bound(None);
        let before = reads();
        world
            .application
            .redispatch_admitted_external_effect(&handle, &authority, &admission)
            .unwrap();
        assert!(
            reads() > before,
            "admitted redispatch contacts its outbox reader"
        );
        assert_eq!(transport.0.load(Ordering::Relaxed), 1);
    }
}
