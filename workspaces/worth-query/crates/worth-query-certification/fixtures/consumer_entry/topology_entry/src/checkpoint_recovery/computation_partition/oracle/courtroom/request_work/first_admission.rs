//! Cold occurrence admission, settlement and release do not scan other rows.
use super::population::{install_population, ScaleProgram};
use super::*;
#[test]
fn first_ordinary_and_performed_requests_visit_only_their_occurrence() {
    let _guard = checkpoint_recovery_test_guard();
    let mut ordinary = Vec::new();
    let mut performed = Vec::new();
    for unrelated in [0, 1_000] {
        let app = install_population();
        let (scope, principal) = authenticate(&app);
        let request = app.request(&principal, &scope);
        let mut held = Vec::new();
        for number in 0..unrelated {
            let mut row = request
                .demand(PlanarOutputDemand::new(format!("unrelated-{}", number + 3)))
                .start_in_program::<ScaleProgram, OracleRoot>(&app)
                .unwrap();
            assert!(matches!(
                row.advance(&request).unwrap(),
                WorthQueryApplicationOutputDemandProgress::Settled(_)
            ));
            held.push(row);
        }
        if unrelated == 0 {
            // Match the populated lane's thousand equal-value publications.
            // Only registry retention differs; created output receipts start
            // at the same Native ordinal and perform the same fact-key work.
            for number in 0..1_000_u64 {
                let name = format!("unrelated-{}", number + 3);
                let source = request
                    .query(PlanarRead {
                        body_key: name.clone(),
                    })
                    .execute()
                    .unwrap();
                let value =
                    worth_query_consumer_values::PositiveLength::get(&source.rows()[0].y) + 1;
                let published = request
                    .mutate(PlanarEdit(PlanarMutation {
                        scope_key: name.clone(),
                        operation:
                            worth_query_consumer_values::PlanarOperation::PublishDerivedOutput(
                                worth_query_consumer_values::PlanarDerivedOutput {
                                    body_key: name,
                                    value: length(value),
                                },
                            ),
                    }))
                    .expect_source(source.observed_sources()[0].clone())
                    .idempotency(&(0x612_8900 + number))
                    .execute_in_program::<ScaleProgram>(&app, worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation)
                    .unwrap();
                assert!(matches!(
                    published,
                    WorthQueryApplicationMutationOutcome::Committed { .. }
                ));
            }
        }
        for row in &mut held {
            assert!(matches!(
                row.advance(&request).unwrap(),
                WorthQueryApplicationOutputDemandProgress::Settled(_)
            ));
        }
        assert_eq!(app.queued_required_work_for_test(), 0);
        assert_eq!(
            app.registry_row_count_for_test(),
            unrelated,
            "the cold lane's actual population"
        );
        app.demand_admission_work_for_test();
        app.caller_request_work_for_test();
        let visits = app.whole_registry_rows_visited_for_test();
        let mut row = request
            .demand(PlanarOutputDemand::new("unrelated-1005"))
            .start_in_program::<ScaleProgram, OracleRoot>(&app)
            .unwrap();
        assert_eq!(
            app.registry_row_count_for_test(),
            unrelated + 1,
            "one newly admitted occurrence"
        );
        assert!(matches!(
            row.advance(&request).unwrap(),
            WorthQueryApplicationOutputDemandProgress::Settled(_)
        ));
        drop(row);
        assert_eq!(
            app.registry_row_count_for_test(),
            unrelated + 1,
            "healthy Ready remains cached on release"
        );
        // The model admits one previously absent occurrence; all row predicates
        // are keyed or occurrence-bounded, so whole-map walks yield zero rows.
        assert_eq!(
            app.whole_registry_rows_visited_for_test() - visits,
            0,
            "ordinary first request"
        );
        ordinary.push((
            app.demand_admission_work_for_test(),
            app.caller_request_work_for_test(),
        ));
        let source = request
            .query(PlanarRead {
                body_key: "unrelated-1006".to_owned(),
            })
            .execute()
            .unwrap();
        let visits = app.whole_registry_rows_visited_for_test();
        let changed = request
            .mutate(PlanarSourceAdjustment {
                scope_key: "unrelated-1006".to_owned(),
                replacement_y: length(2),
            })
            .expect_source(source.observed_sources()[0].clone())
            .idempotency(&0x612_8800_u64)
            .execute_performed::<ScaleProgram, OracleRoot>(
                &app,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let worth_query_host::facade::application_entry::WorthQueryApplicationPerformedMutationOutcome::Performed(changed)=changed
            else { panic!("the performed source commits") };
        let mut outputs=changed.start_required_outputs(&app, &request,
            worth_query_host::facade::application_entry::WorthQueryOutputDemandControls::host_policy()).unwrap_or_else(|failure|panic!("{:?}",failure.denial()));
        assert_eq!(
            app.registry_row_count_for_test(),
            unrelated + 2,
            "performed first admission adds one occurrence"
        );
        assert!(matches!(outputs.required_output_mut().advance(&app, &request).unwrap(),
            worth_query_host::facade::application_entry::WorthQueryApplicationProgramOutputProgress::Settled(_)));
        drop(outputs);
        // OracleRoot declares one root and two leaf connections (Region and
        // Alternate). Settlement admits those two leaves; all three stay cached.
        let performed_graph_rows = 1 + 2;
        assert_eq!(
            app.registry_row_count_for_test(),
            unrelated + 1 + performed_graph_rows,
            "the performed Ready graph remains cached"
        );
        assert_eq!(
            app.whole_registry_rows_visited_for_test() - visits,
            0,
            "performed commit, admission, settlement and release"
        );
        performed.push((
            app.demand_admission_work_for_test(),
            app.caller_request_work_for_test(),
        ));
        for (navigation, operations) in app.caller_request_navigation_for_test() {
            assert!(navigation<=operations*primary_graph::WorthQueryPrimaryGraphApplicationRuntime::<CheckpointSchema>::maximum_ordered_descent_work_for_test());
        }
    }
    assert_eq!(
        ordinary[0], ordinary[1],
        "ordinary admission and advance costs"
    );
    assert_eq!(
        performed[0], performed[1],
        "performed admission and advance costs"
    );
}
