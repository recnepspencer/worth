pub use worth_query_execution::facade::domain_computation::bound_advancement_requests_on_this_thread_for_test as bound;
pub use worth_query_execution::facade::domain_computation::place_managed_computations_on_this_thread_for_test as place;
pub use worth_query_execution::facade::domain_computation::WorthQueryExecutionPlacementForTest as Placement;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::QueryRequestExecution;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::Resource;
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
    let policy = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::test_placement::test_policy(NonZeroUsize::MIN, None);
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
                            .checked_execution_request_for((*world.application.runtime_for_test()).product_runtime_for_test())
                            .expect("the issuer accepts its own phase"),
                        1
                    ),
                    MapOutcome::Complete { .. }
                ));
                assert_eq!(
                    with_bootstrap_advancement(
                        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::test_placement::test_policy(NonZeroUsize::MIN, None),
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
    let policy = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::test_placement::test_policy(NonZeroUsize::MIN, None);
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

mod completion_custody;

fn serial_policy(memory_bytes: Option<u64>) -> worth_foundational::ExecutionRequestPolicy {
    worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, memory_bytes, None),
    )
}

fn serial_available_bytes(caller: worth_execution::ExecutionRequest<'_, '_>) -> u64 {
    caller
        .run_completion(worth_execution::ExecutionWorkCeiling::new(None), |_| {
            worth_execution::ExecutionMemoryReservation::available_in_scope(None)
                .expect("bounded serial scope reports available bytes")
        })
        .expect("serial availability probe")
        .0
}

fn open_serial<'a>(
    policy: worth_foundational::ExecutionRequestPolicy,
    caller: worth_execution::ExecutionRequest<'a, 'a>,
) -> QueryRequestExecution<'a> {
    QueryRequestExecution::open_serial_request(
        worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(policy),
        caller,
        None,
        worth_execution::CancellationToken::new(),
        None,
    )
}

#[test]
fn serial_reserve_and_resize_succeed_inside_narrowed_advancement_without_nested_lease_misuse() {
    let caller_policy = serial_policy(Some(16 * 1024));
    let installed_policy = serial_policy(Some(12 * 1024));
    let serial = worth_execution::SerialRequest::from_policy(
        &caller_policy,
        worth_execution::CancellationToken::new(),
        None,
    );
    let caller = worth_execution::ExecutionRequest::serial(&serial);
    let baseline = serial_available_bytes(caller);
    let exec = open_serial(installed_policy, caller);

    let retained = exec
        .run_advancement(None, None, |_phase| {
            let mut held = exec
                .reserve(4096)
                .expect("serial retained reserve must succeed inside narrowed lease");
            assert_eq!(held.bytes(), 4096);

            held.resize(8192).expect("resize within both ceilings");
            assert_eq!(held.bytes(), 8192);

            // Exceeding tighter installed ceiling (12 KiB) must refuse with
            // MemoryLimit and preserve the 8 KiB hold cleanly.
            let over_installed = held.resize(14 * 1024);
            assert!(matches!(over_installed, Err(Resource::MemoryLimit { .. })));
            assert_eq!(held.bytes(), 8192);

            held
        })
        .expect("advancement completes");

    assert_eq!(baseline - serial_available_bytes(caller), 8 * 1024);
    drop(retained);
    assert_eq!(serial_available_bytes(caller), baseline);
}

#[test]
fn preexisting_serial_caller_hold_narrows_inner_lease_and_rejects_over_budget_allocations() {
    let caller_policy = serial_policy(Some(16 * 1024));
    let installed_policy = serial_policy(Some(16 * 1024));
    let serial = worth_execution::SerialRequest::from_policy(
        &caller_policy,
        worth_execution::CancellationToken::new(),
        None,
    );
    let caller = worth_execution::ExecutionRequest::serial(&serial);
    let baseline = serial_available_bytes(caller);
    let preexisting = caller
        .reserve_retained_memory(12 * 1024)
        .expect("pre-existing caller hold")
        .expect("bounded serial returns reservation");
    assert_eq!(baseline - serial_available_bytes(caller), 12 * 1024);

    let exec = open_serial(installed_policy, caller);
    exec.run_advancement(None, None, |phase| {
        // Only ~4 KiB remains on the caller's serial budget: both ordinary
        // inner-lease allocations and retained reserves of 6 KiB must fail
        // cleanly (never NestedLeaseMisuse).
        assert!(reserve_phase_memory(&phase, 6 * 1024).is_err());
        assert!(matches!(
            exec.reserve(6 * 1024),
            Err(Resource::MemoryLimit { .. })
        ));

        let ok_retained = exec
            .reserve(3 * 1024)
            .expect("3 KiB fits within remaining serial room");
        assert_eq!(ok_retained.bytes(), 3 * 1024);
        assert!(matches!(
            exec.reserve(2 * 1024),
            Err(Resource::MemoryLimit { .. })
        ));
        drop(ok_retained);
        let again = exec.reserve(3 * 1024).expect("released hold restores room");
        drop(again);
    })
    .expect("advancement completes");

    assert_eq!(baseline - serial_available_bytes(caller), 12 * 1024);
    drop(preexisting);
    assert_eq!(serial_available_bytes(caller), baseline);
}

#[test]
fn two_advancements_sharing_cloned_serial_budget_and_overlapping_raw_holds_never_over_admit() {
    let shared_budget = worth_execution::SerialMemoryBudget::new(Some(16 * 1024));
    let req_a = worth_execution::SerialRequest::from_memory(
        shared_budget.clone(),
        worth_execution::CancellationToken::new(),
        None,
    );
    let req_b = worth_execution::SerialRequest::from_memory(
        shared_budget.clone(),
        worth_execution::CancellationToken::new(),
        None,
    );
    let caller_a = worth_execution::ExecutionRequest::serial(&req_a);
    let caller_b = worth_execution::ExecutionRequest::serial(&req_b);
    let installed_policy = serial_policy(Some(32 * 1024));
    let exec_a = open_serial(installed_policy, caller_a);
    let exec_b = open_serial(installed_policy, caller_b);

    exec_a
        .run_advancement(None, None, |phase_a| {
            // Raw inner phase reservation charges the shared 16 KiB SerialMemoryBudget.
            let raw_a = reserve_phase_memory(&phase_a, 10 * 1024)
                .expect("first 10 KiB raw phase hold fits in shared 16 KiB budget")
                .expect("leased inner phase returns reservation");

            // A concurrent advancement sharing the cloned SerialMemoryBudget
            // must NOT over-admit another 10 KiB via raw phase, Query reserve,
            // or a subsequent overlapping caller hold.
            std::thread::scope(|scope| {
                scope.spawn(|| {
                    assert!(caller_b.reserve_retained_memory(10 * 1024).is_err());
                    exec_b
                        .run_advancement(None, None, |phase_b| {
                            assert!(reserve_phase_memory(&phase_b, 10 * 1024).is_err());
                            assert!(matches!(
                                exec_b.reserve(10 * 1024),
                                Err(Resource::MemoryLimit { .. })
                            ));
                            let small = reserve_phase_memory(&phase_b, 2 * 1024)
                                .expect("2 KiB fits alongside live 10 KiB hold")
                                .expect("leased inner phase returns reservation");
                            drop(small);
                        })
                        .expect("second advancement completes cleanly");
                });
            });

            drop(raw_a);
            // Releasing raw_a restores full shared room for exec_b.
            std::thread::scope(|scope| {
                scope.spawn(|| {
                    exec_b
                        .run_advancement(None, None, |phase_b| {
                            let after = reserve_phase_memory(&phase_b, 10 * 1024)
                                .expect("10 KiB succeeds after sibling raw hold drops")
                                .expect("leased inner phase returns reservation");
                            drop(after);
                        })
                        .expect("second advancement completes cleanly");
                });
            });
        })
        .expect("first advancement completes cleanly");

    assert_eq!(shared_budget.charged_memory_bytes(), 0);
}

fn reserve_phase_memory(
    phase: &WorthQueryAdvancementPhase<'_>,
    bytes: u64,
) -> Result<Option<worth_execution::ExecutionMemoryReservation>, worth_execution::WorkCeilingDenial>
{
    phase
        .borrowed_request_for_test()
        .reserve_retained_memory(bytes)
}

mod nested_serial;
