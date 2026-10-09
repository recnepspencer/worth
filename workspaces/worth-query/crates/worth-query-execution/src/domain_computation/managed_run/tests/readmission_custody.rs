//! A bounded standalone provider fixture gates its real checkpoint restore at both placements.
//! The fixture has no product World, so it does not borrow an installed runtime's phase.
use super::*;
use crate::domain_computation::primary_graph::{
    bound_advancement_requests_on_this_thread_for_test as bound,
    place_managed_computations_on_this_thread_for_test as place, AdvancementHost,
    WorthQueryAdvancementDenial as Denial, WorthQueryExecutionPlacementForTest as Placement,
    WorthQueryManagedComputationResourceDenial as Resource,
};
thread_local! { static RESTORES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
pub(super) fn record_restore() {
    RESTORES.with(|n| n.set(n.get() + 1));
}
fn restores() -> usize {
    RESTORES.with(std::cell::Cell::get)
}
struct Restore(Placement, Option<worth_foundational::ExecutionBudget>);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}
#[test]
fn bounded_provider_fixture_opener_refuses_before_direct_and_workflow_readmission() {
    use std::num::NonZeroUsize;
    bound(None);
    let host = AdvancementHost::install();
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _restore = Restore(place(placement), bound(None));
        let (yielded, bridge, runtime) = host
            .run_provider_fixture(|phase| readmission_direct::yielded_direct(&phase))
            .unwrap();
        let mut direct = Some(yielded);
        let (yielded, workflow_bridge, workflow_runtime, _) = host
            .run_provider_fixture(|phase| {
                readmission_workflow::yielded_workflow(
                    &phase,
                    yield_fixture::YieldProvider::installed(5),
                )
            })
            .unwrap();
        let mut workflow = Some(yielded);
        for memory in [false, true] {
            bound(Some(worth_foundational::ExecutionBudget::new(
                NonZeroUsize::MIN,
                if memory { 0 } else { 64 * 1024 * 1024 },
                if memory { 8_000_000 } else { 0 },
            )));
            let before = restores();
            let direct_stop = host
                .run_provider_fixture(|phase| {
                    direct
                        .take()
                        .unwrap()
                        .readmit_same_runtime(&phase, &runtime, &bridge)
                })
                .err()
                .expect("zero request refuses before direct restore");
            let workflow_stop = host
                .run_provider_fixture(|phase| {
                    workflow.take().unwrap().readmit_same_runtime(
                        &phase,
                        &workflow_runtime,
                        &workflow_bridge,
                    )
                })
                .err()
                .expect("zero request refuses before workflow restore");
            assert_eq!(direct_stop, workflow_stop);
            if !memory {
                assert_eq!(direct_stop, Denial::Resource(Resource::WorkExhausted));
            } else if placement == Placement::Serial {
                assert_eq!(direct_stop, Denial::Resource(Resource::PolicyMemoryLimit));
            } else {
                let Denial::Resource(Resource::MemoryLimit {
                    level,
                    requested,
                    admitted,
                }) = direct_stop
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
            assert_eq!(restores(), before);
        }
        bound(None);
        let before = restores();
        let direct = host
            .run_provider_fixture(|phase| {
                direct
                    .take()
                    .unwrap()
                    .readmit_same_runtime(&phase, &runtime, &bridge)
            })
            .unwrap();
        assert!(
            restores() > before,
            "direct control reaches its actual checkpoint reader"
        );
        let crate::domain_computation::WorthQueryDirectReadmissionOutcome::Readmitted(direct) =
            direct
        else {
            panic!("admitted direct restore")
        };
        let _ = direct.into_active().abandon();
        let before = restores();
        let workflow = host
            .run_provider_fixture(|phase| {
                workflow.take().unwrap().readmit_same_runtime(
                    &phase,
                    &workflow_runtime,
                    &workflow_bridge,
                )
            })
            .unwrap();
        assert!(
            restores() > before,
            "workflow control reaches its actual checkpoint reader"
        );
        let crate::domain_computation::WorthQueryWorkflowReadmissionOutcome::Readmitted(workflow) =
            workflow
        else {
            panic!("admitted workflow restore")
        };
        let _ = workflow.into_active().abandon();
    }
}
