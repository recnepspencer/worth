//! The wake pump is also the executing host door for conditional reentry.
use super::{primary_graph, world::CourtroomWorld};
use std::num::NonZeroUsize;
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial as Denial,
        WorthQueryManagedComputationResourceDenial as Resource,
    },
    primary_graph::{
        advancement_requests_on_this_thread_for_test as reports,
        bound_advancement_requests_on_this_thread_for_test as bound,
        installed_source_reads_on_this_thread_for_test as reads,
        place_managed_computations_on_this_thread_for_test as place,
        WorthQueryExecutionPlacementForTest as Placement,
    },
};
struct Restore(Placement, Option<worth_foundational::ExecutionBudget>);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}
#[test]
fn wake_pump_refuses_before_clock_truth_reads_or_operation_reentry() {
    let world = CourtroomWorld::publish("ready");
    let mut port = world.conditional_clock();
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _placement = Restore(place(placement), bound(None));
        let before = world.clock_control.observations();
        let _ = port.observe();
        assert!(
            world.clock_control.observations() > before,
            "admitted wake pump contacts its actual clock reader"
        );
        for zero_memory in [false, true] {
            let _restore = Restore(
                place(placement),
                bound(Some(worth_foundational::ExecutionBudget::new(
                    NonZeroUsize::MIN,
                    if zero_memory { 0 } else { 64 * 1_024 * 1_024 },
                    if zero_memory { 8_000_000 } else { 0 },
                ))),
            );
            let before = (
                reads(),
                world.clock_control.observations(),
                world.contacts.snapshot(),
            );
            reports();
            let primary_graph::WorthQueryConditionalClockObservationOutcome::Failed(failure) =
                port.observe()
            else {
                panic!("the host policy refuses before the wake pump begins");
            };
            let primary_graph::WorthQueryConditionalClockObservationFailureKind::ExecutionRequest(
                cause,
            ) = failure.kind()
            else {
                panic!("the pump retains request admission");
            };
            if !zero_memory {
                assert_eq!(cause, Denial::Resource(Resource::WorkExhausted));
            } else {
                match placement {
                    Placement::Serial => {
                        assert_eq!(cause, Denial::Resource(Resource::PolicyMemoryLimit))
                    }
                    Placement::Leased(_) => {
                        let Denial::Resource(Resource::MemoryLimit {
                            level,
                            requested,
                            admitted,
                        }) = cause
                        else {
                            panic!("the exact lease bytes are retained");
                        };
                        assert_eq!(level, worth_query_host::facade::application_contribution::WorthQueryMemoryLimitLevel::Policy);
                        assert!(requested > 0);
                        assert_eq!(admitted, 0);
                    }
                    Placement::World | Placement::Certified { .. } => {
                        unreachable!("this probe declares placement")
                    }
                }
            }
            assert_eq!(
                (
                    reads(),
                    world.clock_control.observations(),
                    world.contacts.snapshot()
                ),
                before
            );
            assert_eq!(reports(), vec![Err(cause)]);
        }
    }
}

#[path = "advancement_custody/conditional_delivery.rs"]
mod conditional_delivery;

#[path = "advancement_custody/width_differential.rs"]
mod width_differential;
