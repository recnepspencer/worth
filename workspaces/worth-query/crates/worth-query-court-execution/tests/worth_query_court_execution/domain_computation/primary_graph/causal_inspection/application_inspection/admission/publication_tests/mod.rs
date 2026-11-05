//! Cancellation after the actual consuming terminal prevents final publication.
pub use std::sync::Arc;
pub use worth_execution::ExecutionRequest;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::causal_inspection::application_inspection::admission::InspectionAdmissionDenial;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::causal_inspection::application_inspection::admission::InspectionRead;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::causal_inspection::application_inspection::admission::subject::InspectionSubject;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::causal_inspection::application_inspection::target::WorthQueryApplicationInspectionTarget;
pub use worth_query_execution::court_doors_for_test::domain_computation::authorization::application_disclosure::decision::WorthQueryApplicationQueryGovernance;
pub use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::causal_inspection::application_inspection::admission::preparation::prepare;

pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::admission::permission::fresh::FreshQueryPermissionStop;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::causal_inspection::application_inspection::stop::InspectionCause;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::causal_inspection::application_inspection::tests::observe;
pub use worth_query_execution::facade::application_inspection::WorthQueryApplicationInspectionDenial;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::current_output_source::TestAccountSourceBinding;
pub use std::num::NonZeroUsize;
pub use std::time::Duration;
pub use worth_execution::ExecutionAllocationPolicy;
pub use worth_execution::ExecutionAuthority;
pub use worth_execution::LeaseRequest;
pub use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;

#[test]
fn cancelled_after_consuming_completion_cannot_publish_and_keeps_original_payer() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    crate::domain_computation::primary_graph::causal_inspection::application_inspection::admission::publication_tests::uninstalled::assert_original_installation_cause(world.application.installed_schema());
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Some(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system().now() + Duration::from_secs(60)),
        cancellation.token(),
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let original = observe(&world, &external, &request, false);
    let installed = world
        .application
        .installed_schema()
        .installed_query_binding::<TestAccountSourceBinding>()
        .unwrap();
    let policy = worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            NonZeroUsize::MIN,
            Some(32 * 1024 * 1024),
            Some(100_000_000),
        ),
    );
    let authority = ExecutionAuthority::serial();
    let parent = authority
        .request_lease(LeaseRequest {
            policy,
            deadline: None,
            cancellation: worth_execution::CancellationToken::new(),
        })
        .unwrap();
    let readers = world.application.application_query_basis_observer();
    let before_readers = readers.observe().active();
    let before_memory = parent.charged_memory_bytes();
    let mut memory = None;
    let mut failed = None;
    let mut completed_actual_terminal = false;
    world
        .application
        .with_funded_application_advancement(
            &request,
            ExecutionAllocationPolicy::Execution(&parent),
            |phase| {
                let execution = phase
                    .checked_execution_request_for(world.application.product_runtime())
                    .unwrap();
                let mut complete = || {
                    let selected = world
                        .application
                        .on_branch(world.application.current_world())
                        .select_with_request_for_test(&execution)
                        .unwrap();
                    let (report, terminal) = world
                        .application
                        .with_fresh_retained_query_identities_for_test(
                            &execution,
                            &installed,
                            None,
                            &external,
                            &(*original.source_for_test()),
                            &selected,
                            &request,
                            |_, principal, scope| {
                                let mut read = prepare(
                                    &world.application,
                                    &execution,
                                    &installed,
                                    &selected,
                                    principal,
                                    scope,
                                    &request,
                                    InspectionSubject::Query(&original),
                                    None,
                                    None,
                                    &mut memory,
                                )
                                .unwrap_or_else(|_| {
                                    panic!("actual fresh inspection preparation succeeds")
                                });
                                crate::domain_computation::primary_graph::causal_inspection::application_inspection::admission::publication_tests::affinity::assert_subject_affinity(
                                    &read,
                                    &execution,
                                    principal.principal_for_test().principal_entity_id_for_test(),
                                );
                                let report = read
                                    .materialize_for_test(&execution)
                                    .unwrap_or_else(|_| panic!("actual report materializes"));
                                let terminal = read
                                    .terminal_for_test(&execution)
                                    .unwrap_or_else(|_| panic!("actual final read check succeeds"));
                                Ok::<_, FreshQueryPermissionStop<()>>((report, terminal))
                            },
                        )
                        .unwrap_or_else(|_| panic!("issued identity continuation succeeds"));
                    let (_, product, basis) = selected.into_parts_for_test();
                    drop(product);
                    let completed = terminal
                        .complete_for_test(&execution, basis.release())
                        .unwrap_or_else(|_| panic!("actual session completes after release"));
                    (completed, report)
                };
                let (mut completed, report) = complete();
                let (mut other, _) = complete();
                drop(complete);
                completed.assert_completion_facts_for_test(&mut other, &execution, &report);
                completed_actual_terminal = true;
                assert!(memory.as_ref().unwrap().bytes() > 0);
                cancellation.cancel();
                failed = Some(
                    completed
                        .publish_for_test(&execution, &original, report, &mut memory)
                        .err()
                        .expect("cancelled final publication cannot return prepared success"),
                );
            },
        )
        .unwrap();
    assert!(completed_actual_terminal);
    let denial = WorthQueryApplicationInspectionDenial::new_for_test(
        InspectionCause::Admission(failed.unwrap()),
        memory,
    );
    assert!(matches!((*denial.cause_for_test()), InspectionCause::Admission(InspectionAdmissionDenial::RequestControl(
        worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::Cancelled
    ))));
    assert!(parent.charged_memory_bytes() > before_memory);
    assert_eq!(readers.observe().active(), before_readers);
    drop(denial);
    assert_eq!(parent.charged_memory_bytes(), before_memory);
}

#[path = "affinity/mod.rs"]
pub mod affinity;
#[path = "uninstalled/mod.rs"]
pub mod uninstalled;
