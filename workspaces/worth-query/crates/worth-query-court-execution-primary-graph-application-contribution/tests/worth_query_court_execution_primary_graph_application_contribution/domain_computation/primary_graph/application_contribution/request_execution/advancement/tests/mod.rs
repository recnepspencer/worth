pub use worth_query_execution::facade::domain_computation::bound_advancement_requests_on_this_thread_for_test as bound;
pub use worth_query_execution::facade::domain_computation::place_managed_computations_on_this_thread_for_test as place;
pub use worth_query_execution::facade::domain_computation::WorthQueryExecutionPlacementForTest as Placement;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::QueryRequestExecution;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::Resource;
pub use worth_execution::WorkCeilingDenial;
pub use worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial;
pub use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase;
pub use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationInterruption;
pub use worth_query_execution::facade::application_contribution::with_bootstrap_advancement;
pub use std::collections::BTreeMap;
pub use std::num::NonZeroUsize;
pub use worth_execution::ExecutionMap;
pub use worth_execution::KeylessPartition;
pub use worth_execution::MapKernelFailure;
pub use worth_execution::MapOutcome;
pub use worth_execution::MapStop;
pub use worth_foundational::ExecutionBudget;
pub use worth_foundational::PartitionIdentity;

pub struct Restore(Placement, Option<ExecutionBudget>);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}

pub fn request() -> QueryRequestExecution<'static> {
    let policy = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::test_placement::test_policy(NonZeroUsize::MIN, None);
    QueryRequestExecution::open_control_for_test(
        worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(policy),
        worth_execution::CancellationToken::new(),
        None,
    )
}

pub fn charged_request(
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
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    const UNITS: u64 = 3;
    let _restore = Restore(
        place(Placement::Leased(NonZeroUsize::MIN)),
        bound(Some(ExecutionBudget::new(
            NonZeroUsize::MIN,
            None,
            Some(2 * UNITS),
        ))),
    );
    worth_query_execution::facade::domain_computation::advancement_requests_on_this_thread_for_test(
    );
    let request = request();
    let stop = request
        .run_advancement_for_test(None, None, |phase| {
            assert!(matches!(
                charged_request(phase.borrowed_request_for_test(), UNITS),
                MapOutcome::Complete { .. }
            ));
            assert!(matches!(
                charged_request(phase.borrowed_request_for_test(), UNITS),
                MapOutcome::Complete { .. }
            ));
            // This local law probes nested scopes; the real caller retry
            // is covered by the topology-entry advancement_custody fixture.
            charged_request(phase.borrowed_request_for_test(), UNITS)
        })
        .unwrap();
    assert!(matches!(stop, MapOutcome::Stopped {
        reason: MapStop::WorkExhausted { identity }, ..
    } if identity == PartitionIdentity::new(1)));
    let reports = worth_query_execution::facade::domain_computation::advancement_requests_on_this_thread_for_test();
    assert_eq!(reports.len(), 1);
}

#[test]
fn zero_work_refuses_before_the_advancement_body() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let _restore = Restore(
        place(Placement::Leased(NonZeroUsize::MIN)),
        bound(Some(ExecutionBudget::new(NonZeroUsize::MIN, None, Some(0)))),
    );
    let result = request().run_advancement_for_test(None, None, |_| {
        panic!("a refused request cannot reach its reader")
    });
    assert_eq!(
        result,
        Err(WorthQueryAdvancementDenial::Resource(
            Resource::WorkExhausted
        ))
    );
    worth_query_execution::facade::domain_computation::advancement_requests_on_this_thread_for_test(
    );
}

#[test]
fn reentrant_host_open_is_typed_at_serial_and_leased_placements() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let scope = live_scope();
    let _restore = Restore(place(Placement::World), bound(None));
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        place(placement);
        worth_query_execution::facade::domain_computation::advancement_requests_on_this_thread_for_test();
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
                            .execution_request_for_for_test(&(*(*world.application.runtime_for_test()).product_runtime_for_test()))
                            .expect("the issuer accepts its own phase"),
                        1
                    ),
                    MapOutcome::Complete { .. }
                ));
                assert_eq!(
                    with_bootstrap_advancement(
                        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::test_placement::test_policy(NonZeroUsize::MIN, None),
                        |_| ()
                    ),
                    Err(WorthQueryAdvancementDenial::NestedOpening)
                );
            })
            .unwrap();
        let reports = worth_query_execution::facade::domain_computation::advancement_requests_on_this_thread_for_test();
        // A refused nested opening owns no advancement to report.
        assert_eq!(reports.len(), 1);
        assert_eq!(reports.iter().filter(|report| report.is_ok()).count(), 1);
    }
    // The guard is released at the public call's end.
    world
        .application
        .with_application_advancement(&scope, |_| ())
        .unwrap();
    worth_query_execution::facade::domain_computation::advancement_requests_on_this_thread_for_test(
    );
}

#[test]
fn unwinding_a_host_callback_releases_opening_custody() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let policy = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::test_placement::test_policy(NonZeroUsize::MIN, None);
    assert!(std::panic::catch_unwind(|| {
        let _ = with_bootstrap_advancement(policy, |_| panic!("host callback"));
    })
    .is_err());
    assert_eq!(with_bootstrap_advancement(policy, |_| ()), Ok(()));
    worth_query_execution::facade::domain_computation::advancement_requests_on_this_thread_for_test(
    );
}

#[path = "member_lineage/mod.rs"]
pub mod member_lineage;
#[path = "observations/mod.rs"]
pub mod observations;
#[path = "panic_custody/mod.rs"]
pub mod panic_custody;
#[path = "seam_cancellation/mod.rs"]
pub mod seam_cancellation;
