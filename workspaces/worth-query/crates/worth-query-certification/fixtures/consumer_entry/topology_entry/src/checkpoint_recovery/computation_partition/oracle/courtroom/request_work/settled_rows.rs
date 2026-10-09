use super::*;
type ScaleProgram = OracleProgram<false, TOTALS_WORK, 1, 4>;
#[test]
fn settled_current_ordinary_rows_do_not_charge_the_callers_request() {
    let _guard = checkpoint_recovery_test_guard();
    {
        let app = super::population::install_population();
        let (scope, principal) = authenticate(&app);
        let request = app.request(&principal, &scope);
        let mut root = request
            .demand(PlanarOutputDemand::new(SCOPE))
            .start_in_program::<ScaleProgram, OracleRoot>(&app)
            .unwrap();
        assert!(matches!(
            root.advance(&request).unwrap(),
            WorthQueryApplicationOutputDemandProgress::Settled(_)
        ));
        let mut output = request
            .demand(RegionOutputDemand(SCOPE.to_owned()))
            .start_dependent_in_program::<ScaleProgram, RegionConnection>(&app)
            .unwrap();
        assert!(matches!(
            output.advance(&request).unwrap(),
            WorthQueryApplicationOutputDemandProgress::Settled(_)
        ));
        let unrelated_request = app.request(&principal, &scope);
        // All three queries read all three triangle vertices. These same
        // interests are held in both measurements; only the additional,
        // disjoint triangles differ in their number of open settled rows.
        let mut triangle_rows = Vec::new();
        for corner in 0..3 {
            let mut row = unrelated_request
                .demand(PlanarOutputDemand::new(format!("unrelated-{corner}")))
                .start_in_program::<ScaleProgram, OracleRoot>(&app)
                .unwrap();
            assert!(matches!(
                row.advance(&unrelated_request).unwrap(),
                WorthQueryApplicationOutputDemandProgress::Settled(_)
            ));
            triangle_rows.push(row);
        }
        let mut queue_caller = unrelated_request
            .demand(PlanarOutputDemand::new("unrelated-1004"))
            .start_in_program::<ScaleProgram, OracleRoot>(&app)
            .unwrap();
        assert!(matches!(
            queue_caller.advance(&unrelated_request).unwrap(),
            WorthQueryApplicationOutputDemandProgress::Settled(_)
        ));
        let mut costs = Vec::new();
        let mut visited = Vec::new();
        let mut quiet_populations = Vec::new();
        let mut pending_costs = Vec::new();
        let mut pending_visited = Vec::new();
        let mut held = Vec::new();
        let mut populations = Vec::new();
        for unrelated in [0, 1_000] {
            let opened = unrelated - held.len();
            let setup_contacts = app.producer_contacts_on_this_thread_for_test();
            for number in held.len()..unrelated {
                let mut row = unrelated_request
                    .demand(PlanarOutputDemand::new(format!("unrelated-{}", number + 3)))
                    .start_in_program::<ScaleProgram, OracleRoot>(&app)
                    .unwrap();
                assert!(matches!(
                    row.advance(&unrelated_request).unwrap(),
                    WorthQueryApplicationOutputDemandProgress::Settled(_)
                ));
                held.push(row);
            }
            assert_eq!(held.len(), unrelated);
            // These are newly admitted rows, not cached interests: each first
            // demand republishes its pre-existing byte-equal Native value once.
            assert_eq!(
                app.producer_contacts_on_this_thread_for_test() - setup_contacts,
                opened as u64,
                "one first decision per newly installed ordinary row"
            );
            for row in &mut held {
                assert!(matches!(
                    row.advance(&unrelated_request).unwrap(),
                    WorthQueryApplicationOutputDemandProgress::Settled(_)
                ));
            }
            for row in &mut triangle_rows {
                assert!(matches!(
                    row.advance(&unrelated_request).unwrap(),
                    WorthQueryApplicationOutputDemandProgress::Settled(_)
                ));
            }
            assert!(matches!(
                queue_caller.advance(&unrelated_request).unwrap(),
                WorthQueryApplicationOutputDemandProgress::Settled(_)
            ));
            assert_eq!(
                app.queued_required_work_for_test(),
                0,
                "all newly admitted interests are current before the edit schedule"
            );
            // Native values exist in both measurements; zero versus a thousand
            // actual registry rows now differ, not merely open interests.

            // Both measurements have a full retained lineage window. Cold
            // history has fewer retirement candidates, independently of open rows.
            let measured = 2 * installation::RETAINED_POSITIONS as u64 + 1;
            for phase in 0..=measured {
                let source = unrelated_request
                    .query(PlanarRead {
                        body_key: "unrelated-0".to_owned(),
                    })
                    .execute()
                    .unwrap();
                let replacement = length(2 + phase % 2);
                assert_ne!(
                    source.rows()[0].y,
                    replacement,
                    "both setup and measurement really change vertex zero"
                );
                let dirty_root = source.observed_sources()[0].root_entity_for_test();
                let edit = unrelated_request
                    .mutate(PlanarSourceAdjustment {
                        scope_key: "unrelated-0".to_owned(),
                        replacement_y: replacement,
                    })
                    .expect_source(source.observed_sources()[0].clone())
                    .idempotency(&(0x612_7000_u64 + unrelated as u64 * (measured + 1) + phase))
                    .execute_performed::<ScaleProgram, OracleRoot>(&app, worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation);
                assert!(matches!(edit, Ok(worth_query_host::facade::application_entry::WorthQueryApplicationPerformedMutationOutcome::Performed(_))), "vertex zero changes: {:?}", edit.as_ref().err());
                drop((edit, source));
                if phase == measured {
                    // PlanarRead declares two successors: the edit marks all
                    // three held triangle roots, and no disjoint root.
                    assert_eq!(
                        app.queued_required_work_for_test(),
                        triangle_rows.len(),
                        "only the same triangle has work pending"
                    );
                }
                let before = app.producer_contacts_at_root_on_this_thread_for_test(dirty_root);
                app.caller_request_work_for_test();
                app.caller_request_navigation_for_test();
                let rows_before = app.whole_registry_rows_visited_for_test();
                assert!(matches!(
                    queue_caller.advance(&unrelated_request).unwrap(),
                    WorthQueryApplicationOutputDemandProgress::Settled(_)
                ));
                let work = app.caller_request_work_for_test();
                assert_eq!(work.len(), 1);
                assert_eq!(
                    app.producer_contacts_at_root_on_this_thread_for_test(dirty_root) - before,
                    1,
                    "population {unrelated}, phase {phase}: the pending root is progressed"
                );
                if phase == measured {
                    pending_costs.push(work[0]);
                    // Three held triangle priors coexist with their three
                    // refreshed rows; SCOPE root/leaf and queue caller add three.
                    let expected_rows = 2 * triangle_rows.len() + 3 + unrelated;
                    assert_eq!(
                        app.registry_row_count_for_test(),
                        expected_rows,
                        "model population at pending settlement"
                    );
                    populations.push(app.registry_row_count_for_test());
                    pending_visited.push(app.whole_registry_rows_visited_for_test() - rows_before);
                }
                for (navigation, operations) in app.caller_request_navigation_for_test() {
                    assert!(
                        navigation
                            <= operations
                                * primary_graph::WorthQueryPrimaryGraphApplicationRuntime::<
                                    CheckpointSchema,
                                >::maximum_ordered_descent_work_for_test(
                                )
                    );
                }
                let output = unrelated_request
                    .query(PlanarOutputRead {
                        body_key: "unrelated-0".to_owned(),
                    })
                    .execute()
                    .unwrap();
                assert_eq!(output.rows()[0].value, length(3 + phase % 2));
                for row in &mut triangle_rows {
                    assert!(matches!(
                        row.advance(&unrelated_request).unwrap(),
                        WorthQueryApplicationOutputDemandProgress::Settled(_)
                    ));
                }
                assert!(matches!(
                    queue_caller.advance(&unrelated_request).unwrap(),
                    WorthQueryApplicationOutputDemandProgress::Settled(_)
                ));
                if phase == 0 {
                    assert_eq!(
                        app.queued_required_work_for_test(),
                        0,
                        "setup acknowledges newly opened interests before measurement"
                    );
                }
            }
            // Measure this lane while its actual population is still zero or
            // 1000, before creating the next lane's rows. All setup is drained.
            for row in &mut held {
                assert!(matches!(
                    row.advance(&unrelated_request).unwrap(),
                    WorthQueryApplicationOutputDemandProgress::Settled(_)
                ));
            }
            assert!(matches!(
                output.advance(&request).unwrap(),
                WorthQueryApplicationOutputDemandProgress::Settled(_)
            ));
            assert!(matches!(
                root.advance(&request).unwrap(),
                WorthQueryApplicationOutputDemandProgress::Settled(_)
            ));
            assert_eq!(app.queued_required_work_for_test(), 0);
            // Rejoining each triangle releases its superseded prior. The
            // three current triangle rows and the three callers remain cached.
            let expected_rows = triangle_rows.len() + 3 + unrelated;
            assert_eq!(
                app.registry_row_count_for_test(),
                expected_rows,
                "model population at the quiet boundary"
            );
            quiet_populations.push(app.registry_row_count_for_test());
            app.caller_request_work_for_test();
            app.caller_request_navigation_for_test();
            let rows_before = app.whole_registry_rows_visited_for_test();
            assert!(matches!(
                output.advance(&request).unwrap(),
                WorthQueryApplicationOutputDemandProgress::Settled(_)
            ));
            let work = app.caller_request_work_for_test();
            assert_eq!(work.len(), 1);
            costs.push(work[0]);
            visited.push(app.whole_registry_rows_visited_for_test() - rows_before);
            for (navigation, operations) in app.caller_request_navigation_for_test() {
                assert!(
                    navigation
                        <= operations
                            * primary_graph::WorthQueryPrimaryGraphApplicationRuntime::<
                                CheckpointSchema,
                            >::maximum_ordered_descent_work_for_test(
                            )
                );
            }
        }
        assert_eq!(
            populations[1] - populations[0],
            1_000,
            "the pending-work requests differ by a thousand actual registry rows"
        );
        assert_eq!(pending_costs[0], pending_costs[1], "the same pending work costs the same with zero or a thousand disjoint settled open rows");
        assert_eq!(
            pending_visited,
            [0, 0],
            "whole-map row visits with the same pending work"
        );
        assert_eq!(
            quiet_populations[1] - quiet_populations[0],
            1_000,
            "quiet lanes differ by actual registry rows"
        );
        assert_eq!(
            costs[0], costs[1],
            "settled current unrelated rows perform no request work; whole-map visits: {visited:?}"
        );
        assert_eq!(
            visited,
            [0, 0],
            "a held current Ready needs no global row question"
        );
    }
}
