//! Actual independently admitted rich scopes cannot be substituted even for one original.
use worth_execution::ExecutionMemoryReservation;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::causal_inspection::application_inspection::admission::InspectionAdmissionDenial;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::causal_inspection::application_inspection::admission::InspectionDiagnosticPreparation;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::causal_inspection::application_inspection::admission::subject::InspectionSubject;
use worth_query_execution::facade::application_inspection::WorthQueryApplicationMaterializationRequest;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::causal_inspection::application_inspection::admission::preparation::prepare;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::causal_inspection::application_inspection::admission::require_request;
use worth_query_execution::court_doors_for_test::domain_computation::authorization::application_disclosure::pending_access::funded::prepare_funded_pending_disclosure;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::admission::permission::fresh::FreshQueryPermissionStop;
use worth_query_execution::facade::application_inspection::WorthQueryApplicationEvidenceFamily as Family;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::diagnostic_comparison::DiagnosticAccountBinding;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_inspection_capability_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use std::time::Duration;
use std::time::UNIX_EPOCH;

#[test]
fn separately_admitted_rich_body_cannot_replace_scope_or_requested_families() {
    let world = installed_inspection_capability_world();
    world
        .authorization_time
        .hold(UNIX_EPOCH + Duration::from_secs(100));
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let (original, rows) = crate::domain_computation::primary_graph::causal_inspection::application_inspection::admission::rich::tests::observation::read::<DiagnosticAccountBinding>(
        &world, &external, &request, "no-match", true,
    );
    assert!(rows.is_empty());
    let capability_a = crate::domain_computation::primary_graph::causal_inspection::application_inspection::admission::rich::tests::observation::capability(&world, &external, &request);
    let capability_b = crate::domain_computation::primary_graph::causal_inspection::application_inspection::admission::rich::tests::observation::capability(&world, &external, &request);
    let installed = world
        .application
        .installed_schema()
        .installed_query_binding::<DiagnosticAccountBinding>()
        .unwrap();
    let detail = WorthQueryApplicationMaterializationRequest::family(Family::QueryInspection);
    let readers = world.application.application_query_basis_observer();
    let baseline = readers.observe().active();
    world
        .application
        .with_host_advancement(|phase| {
            let execution = phase
                .checked_execution_request_for(world.application.product_runtime())
                .unwrap();
            let selected = world
                .application
                .on_branch(world.application.current_world())
                .select_with_request_for_test(&execution)
                .unwrap();
            let mut memory_a = None;
            let mut memory_b = None;
            let ((report_a, terminal_a), (mut report_b, terminal_b)) = world
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
                        let pending =
                            |capability, memory: &mut Option<ExecutionMemoryReservation>| {
                                prepare_funded_pending_disclosure(
                                    &world.application,
                                    installed.query().binding_identity(),
                                    principal.principal_for_test().principal_entity_id_for_test(),
                                    scope.scope_for_test().entity_id_for_test(),
                                    capability,
                                    || require_request(&execution, &request),
                                    || InspectionAdmissionDenial::ForeignIssuedAccess,
                                    &execution,
                                    memory,
                                    InspectionAdmissionDenial::Storage,
                                )
                                .unwrap_or_else(|_| {
                                    panic!("genuine diagnostic preparation succeeds")
                                })
                            };
                        let pending_a = pending(capability_a, &mut memory_a);
                        let pending_b = pending(capability_b, &mut memory_b);
                        let mut read_a = prepare(
                            &world.application,
                            &execution,
                            &installed,
                            &selected,
                            principal,
                            scope,
                            &request,
                            InspectionSubject::Query(&original),
                            None,
                            Some(InspectionDiagnosticPreparation::Rich {
                                request: detail,
                                pending: pending_a,
                            }),
                            &mut memory_a,
                        )
                        .unwrap_or_else(|_| panic!("first real rich read admits"));
                        let report_a = read_a
                            .materialize_rich_for_test(&execution, installed.query())
                            .unwrap_or_else(|_| panic!("first rich report assembles"));
                        let terminal_a = read_a
                            .terminal_for_test(&execution)
                            .unwrap_or_else(|_| panic!("first terminal admits"));
                        let mut read_b = prepare(
                            &world.application,
                            &execution,
                            &installed,
                            &selected,
                            principal,
                            scope,
                            &request,
                            InspectionSubject::Query(&original),
                            None,
                            Some(InspectionDiagnosticPreparation::Rich {
                                request: detail,
                                pending: pending_b,
                            }),
                            &mut memory_b,
                        )
                        .unwrap_or_else(|_| panic!("second real rich read admits"));
                        let report_b = read_b
                            .materialize_rich_for_test(&execution, installed.query())
                            .unwrap_or_else(|_| panic!("second rich report assembles"));
                        let terminal_b = read_b
                            .terminal_for_test(&execution)
                            .unwrap_or_else(|_| panic!("second terminal admits"));
                        Ok::<_, FreshQueryPermissionStop<()>>((
                            (report_a, terminal_a),
                            (report_b, terminal_b),
                        ))
                    },
                )
                .unwrap_or_else(|_| panic!("real identity continuation succeeds"));
            assert!((*report_a.body_for_test()).matches_for_test(&(*report_a.reference_for_test())));
            assert!((*report_b.body_for_test()).matches_for_test(&(*report_b.reference_for_test())));
            assert!(
                !(*report_a.body_for_test()).matches_for_test(&(*report_b.reference_for_test())),
                "same original and mask do not grant another session membership"
            );
            let saved = *(*(*report_b.body_for_test()).artifact_for_test()).request_for_test();
            (*(*(*report_b.body_mut_for_test()).artifact_mut_for_test()).request_mut_for_test()) = saved.with_family(Family::BridgeRoute);
            assert!(
                !(*report_b.body_for_test()).matches_for_test(&(*report_b.reference_for_test())),
                "same session cannot publish another requested family selection"
            );
            (*(*(*report_b.body_mut_for_test()).artifact_mut_for_test()).request_mut_for_test()) = saved;
            let (_, product, basis) = selected.into_parts_for_test();
            drop(product);
            let release = basis.release();
            let completed_a = terminal_a
                .complete_for_test(&execution, release.clone())
                .unwrap_or_else(|_| panic!("first actual terminal completes"));
            let completed = terminal_b
                .complete_for_test(&execution, release)
                .unwrap_or_else(|_| panic!("second actual terminal completes"));
            // Keep B's authentic base report and completed terminal, substitute only A's body.
            (*report_b.body_mut_for_test()) = report_a.body_into_for_test();
            let denial = completed
                .publish_rich_for_test(&execution, &original, report_b, &mut memory_b)
                .err()
                .expect("the real publication owner must reject a separately admitted body");
            assert!(matches!(
                denial,
                InspectionAdmissionDenial::ForeignIssuedAccess
            ));
            drop(completed_a);
        })
        .unwrap();
    original.close();
    assert_eq!(readers.observe().active(), baseline);
}

#[path = "observation/mod.rs"]
pub mod observation;
