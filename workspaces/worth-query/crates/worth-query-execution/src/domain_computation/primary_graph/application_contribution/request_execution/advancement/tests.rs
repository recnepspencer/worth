use super::super::test_placement::{
    bound_advancement_requests_on_this_thread_for_test as bound,
    place_managed_computations_on_this_thread_for_test as place,
    WorthQueryExecutionPlacementForTest as Placement,
};
use super::*;
use std::{collections::BTreeMap, num::NonZeroUsize};
use worth_execution::{ExecutionMap, KeylessPartition, MapKernelFailure, MapOutcome, MapStop};
use worth_foundational::{ExecutionBudget, PartitionIdentity};

struct Restore(Placement, Option<ExecutionBudget>);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}

fn request() -> QueryRequestExecution<'static> {
    let policy = super::super::test_placement::test_policy(NonZeroUsize::MIN, 1 << 20);
    QueryRequestExecution::open_control(
        worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(policy),
        worth_execution::CancellationToken::new(),
        None,
    )
}

fn charged_request(
    request: worth_execution::ExecutionRequest<'_, '_>,
    units: u64,
) -> MapOutcome<(), ()> {
    let map = ExecutionMap::<(), u64>::from_keyless_partitions(BTreeMap::from([(
        PartitionIdentity::new(1),
        KeylessPartition {
            value: (),
            kernel_scratch_bytes: 0,
            max_result_bytes: 0,
        },
    )]))
    .unwrap();
    request
        .in_scope(|lease| {
            map.run(lease, |_, context| {
                context.checkpoint(units).map_err(MapKernelFailure::Stop)?;
                Ok(())
            })
        })
        .unwrap()
}

#[test]
fn nested_execution_phases_share_one_work_ceiling() {
    const UNITS: u64 = 3;
    let _restore = Restore(
        place(Placement::Leased(NonZeroUsize::MIN)),
        bound(Some(ExecutionBudget::new(
            NonZeroUsize::MIN,
            1 << 20,
            2 * UNITS,
        ))),
    );
    observation::advancement_requests_on_this_thread_for_test();
    let request = request();
    let stop = request
        .run_advancement(None, None, |phase| {
            assert!(matches!(
                charged_request(
                    phase.request_for_source(0).expect("standalone fixture"),
                    UNITS
                ),
                MapOutcome::Complete { .. }
            ));
            assert!(matches!(
                charged_request(
                    phase.request_for_source(0).expect("standalone fixture"),
                    UNITS
                ),
                MapOutcome::Complete { .. }
            ));
            // This local law probes nested scopes; the real caller retry
            // is covered by the topology-entry advancement_custody fixture.
            charged_request(
                phase.request_for_source(0).expect("standalone fixture"),
                UNITS,
            )
        })
        .unwrap();
    assert!(matches!(stop, MapOutcome::Stopped {
        reason: MapStop::WorkExhausted { identity }, ..
    } if identity == PartitionIdentity::new(1)));
    let reports = observation::advancement_requests_on_this_thread_for_test();
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].as_ref().unwrap().charged_work(), 2 * UNITS);
}

#[test]
fn zero_work_refuses_before_the_advancement_body() {
    let _restore = Restore(
        place(Placement::Leased(NonZeroUsize::MIN)),
        bound(Some(ExecutionBudget::new(NonZeroUsize::MIN, 1 << 20, 0))),
    );
    let result = request().run_advancement(None, None, |_| {
        panic!("a refused request cannot reach its reader")
    });
    assert_eq!(
        result,
        Err(WorthQueryAdvancementDenial::Resource(
            Resource::WorkExhausted
        ))
    );
    observation::advancement_requests_on_this_thread_for_test();
}

#[test]
fn reentrant_host_open_is_typed_at_serial_and_leased_placements() {
    use crate::domain_computation::primary_graph::tests::fixture::{
        installed_authorization_world, live_scope,
    };
    let world = installed_authorization_world(true);
    let scope = live_scope();
    let _restore = Restore(place(Placement::World), bound(None));
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        place(placement);
        observation::advancement_requests_on_this_thread_for_test();
        world
            .application
            .with_application_advancement(&scope, |phase| {
                // A host callback may reenter, but cannot acquire another budget.
                let refusal = world.application.with_application_advancement(&scope, |_| {
                    panic!("a nested opening must not enter its reader")
                });
                assert_eq!(refusal, Err(WorthQueryAdvancementDenial::NestedOpening));
                assert!(matches!(
                    charged_request(
                        phase
                            .execution_request_for(&world.application.product_runtime)
                            .expect("the issuer accepts its own phase"),
                        1
                    ),
                    MapOutcome::Complete { .. }
                ));
                assert_eq!(
                    with_bootstrap_advancement(
                        super::super::test_placement::test_policy(NonZeroUsize::MIN, 1 << 20),
                        |_| ()
                    ),
                    Err(WorthQueryAdvancementDenial::NestedOpening)
                );
            })
            .unwrap();
        let reports = observation::advancement_requests_on_this_thread_for_test();
        assert_eq!(reports.len(), 3);
        assert_eq!(reports.iter().filter(|report| report.is_ok()).count(), 1);
    }
    // The guard is released at the public call's end.
    world
        .application
        .with_application_advancement(&scope, |_| ())
        .unwrap();
    observation::advancement_requests_on_this_thread_for_test();
}

#[test]
fn unwinding_a_host_callback_releases_opening_custody() {
    let policy = super::super::test_placement::test_policy(NonZeroUsize::MIN, 1 << 20);
    assert!(std::panic::catch_unwind(|| {
        let _ = with_bootstrap_advancement(policy, |_| panic!("host callback"));
    })
    .is_err());
    assert_eq!(with_bootstrap_advancement(policy, |_| ()), Ok(()));
    observation::advancement_requests_on_this_thread_for_test();
}

#[path = "seam_cancellation.rs"]
mod seam_cancellation;
