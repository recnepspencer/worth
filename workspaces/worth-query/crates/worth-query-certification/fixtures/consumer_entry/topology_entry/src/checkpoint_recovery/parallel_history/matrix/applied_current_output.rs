//! Candidate prerequisites select appended output with unrelated live attempts retained.
use super::super::expected_history::{Member, Partition, World};
use super::{application::*, control, edit, facts, installation, producer, source};
use std::{num::NonZeroUsize, sync::Arc};
use worth_query_host::facade::admission::authenticated_principal as authentication;
use worth_query_host::facade::{
    application_entry::*, primary_graph::*, runtime::ExecutionAllocationPolicy,
};

#[test]
fn appended_prerequisite_canonically_settles_without_visiting_unrelated_live_attempts() {
    let _guard = crate::checkpoint_recovery::checkpoint_recovery_test_guard();
    let empty = journey(0);
    let populated = journey(16);
    assert!(empty.0 > 0, "real request work was measured");
    assert_eq!(
        empty, populated,
        "unrelated LIVE attempts do not change lookup work or outcome"
    );
}

fn journey(live: usize) -> (u64, u64, Vec<(u64, u64)>) {
    // Identical physical population in both runs; only live attempt custody differs.
    let mut members = vec![member(1, vec![], 3), member(2, vec![0], 5)];
    members.extend((10..26).map(|key| member(key, vec![], 1)));
    let control = Arc::new(control::Control::new(
        World {
            members,
            requested: vec![1],
        },
        true,
        false,
        0,
        false,
    ));
    // Sixteen registered operations retain their genuine queried Sources and
    // selected program observations. Give both populations identical finite World room
    // for that custody and temporary authorization/envelope observations; the
    // ordinary serial-history fixture admits only sixteen total observations.
    let limits = crate::checkpoint_recovery::support::limits_with_room(
        32,
        128,
        64,
        crate::checkpoint_recovery::support::invalidation(),
    );
    let app = installation::install_with_limits(control.clone(), limits);
    let (scope, principal) = installation::authenticate(&app);
    let request = app.request(&principal, &scope);
    let measured = with_registered_attempts(&app, live, || {
        assert_eq!(
            app.active_application_attempts_for_test(),
            live,
            "unpublished independently registered attempts remain LIVE during selection"
        );
        assert!(control.declarations.lock().unwrap().is_empty());
        for key in [1, 2, 100] {
            assert_eq!(
                request.query(facts::Read { key }).execute().unwrap().rows()[0].value,
                0
            );
        }
        let source = request.query(source::Read { key: 1 }).execute().unwrap();
        let mut envelope = request
            .edit_envelope()
            .idempotency(0x6c00_u64)
            .runtime_stages(&app)
            .ordered_authored()
            .unwrap();
        let token = envelope
            .push_stage_with_source::<Root, _, _>(
                edit::Input {
                    key: 1,
                    revision: 1,
                },
                source.observed_sources()[0].clone(),
            )
            .unwrap();
        let _observation = WorthQueryExecutionObservation::install_on_this_thread();
        advancement_observations_on_this_thread_for_test();
        let (result, writes) =
            worth_relational::facade::inspection::observe_maintenance_work(1, || {
                envelope.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            });
        let reports = advancement_observations_on_this_thread_for_test();
        assert!(!reports.is_empty());
        assert!(reports.iter().all(|r| r.refusal.is_none()));
        let work = reports.iter().map(|r| r.report.charged_work()).sum();
        let span = reports.iter().map(|r| r.report.charged_span()).sum();
        let WorthQueryEditEnvelopeOutcome::Committed(receipt) = result.unwrap() else {
            panic!("the real source and appended prerequisite chain co-commit");
        };
        assert_eq!(writes.published_rows.len() + writes.omitted_publications, 1);
        assert_eq!(receipt.result(&token).unwrap().revision, 1);
        assert_eq!(
            receipt.stage_receipt(&token).unwrap().commit_reference(),
            receipt.receipt().commit_reference()
        );
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            receipt
                .receipt()
                .committed_product_publication()
                .composite_commit()
        );
        assert_eq!(app.active_application_attempts_for_test(), live);
        assert_eq!(
            *control.declarations.lock().unwrap(),
            [(1, vec![]), (2, vec![1]), (100, vec![2])]
                .into_iter()
                .collect()
        );
        // No committed predecessor output existed: 2 read appended 1, then 100 read appended 2.
        // Literal answers derive from partition values and the authored revision, not runtime output.
        let mut values = Vec::new();
        for (key, expected) in [(1, 6), (2, 11), (100, 11)] {
            let rows = request.query(facts::Read { key }).execute().unwrap();
            assert_eq!(
                (rows.rows()[0].value, rows.rows()[0].applied),
                // This fixture producer writes Value only; Applied is seeded zero.
                // Actual production/reuse is proved by declarations and native work.
                (expected, 0)
            );
            values.push((key, rows.rows()[0].value));
        }
        let head = request
            .retain_current_read()
            .unwrap()
            .selected_commit()
            .clone();
        let (_, reuse) = worth_relational::facade::inspection::observe_maintenance_work(1, || {
            for key in [1, 2, 100] {
                let observed = request.query(source::Read { key }).execute().unwrap();
                let mut delivery = request
                    .demand(producer::Demand(key))
                    .start_in_program::<Program, Root>(&app)
                    .unwrap();
                let settled = (0..128)
                    .find_map(|_| match delivery.advance(&request).unwrap() {
                        WorthQueryApplicationOutputDemandProgress::Settled(value) => Some(value),
                        _ => None,
                    })
                    .expect("canonical consumed settlement remains current on delivery");
                assert_eq!(settled.producer_contacts_in_this_demand(), 0);
                let outputs = settled.outputs_of::<super::operation::Outputs>().unwrap();
                let node = outputs
                    .entity::<super::operation::NodeOutput<Schema>>()
                    .unwrap();
                assert_eq!(
                    node.entity_id(),
                    observed.observed_sources()[0].root_entity_for_test()
                );
            }
        });
        assert_eq!(reuse.published_rows.len() + reuse.omitted_publications, 0);
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            &head
        );
        (work, span, values, head)
    });
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &measured.3,
        "cancelling the retained actors publishes nothing"
    );
    assert_eq!(app.active_application_attempts_for_test(), 0);
    for key in 10..26 {
        assert_eq!(
            request
                .query(source::Read { key })
                .execute()
                .unwrap()
                .rows()[0]
                .revision,
            0,
            "cancelled registered attempts preserve unrelated committed sources"
        );
    }
    crate::checkpoint_recovery::support::host_call(&app, |phase| {
        app.on_branch(app.current_world())
            .close(phase)
            .map_err(Box::new)
    })
    .expect("all canonical participant and abandoned-candidate custody closes");
    (measured.0, measured.1, measured.2)
}

fn member(key: u64, upstream: Vec<usize>, value: u64) -> Member {
    Member {
        key,
        upstream,
        partitions: vec![Partition {
            key: 0,
            value,
            work: 1,
            fails: false,
        }],
    }
}

// The existing registration pause parks genuine provider entries, before their
// branch commit lane. Prepared-but-uncommitted handles do not register entries.
#[expect(
    clippy::disallowed_methods,
    reason = "certification-harness thread-scope=1: Fixture-owned actors retain genuine registered attempts during candidate lookup; installed runtime owns all computation and publication."
)]
fn with_registered_attempts<R>(
    app: &installation::Application,
    live: usize,
    observe: impl FnOnce() -> R,
) -> R {
    let cancellations: Vec<_> = (0..live)
        .map(|_| authentication::WorthQueryCancellationSource::new())
        .collect();
    std::thread::scope(|threads| {
        let mut attempts = Vec::new();
        let observed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            for (i, cancellation) in cancellations.iter().enumerate() {
                let pause = Arc::new(
                    app.pause_after_application_attempt_registration_for_test(NonZeroUsize::MIN),
                );
                let worker_pause = Arc::clone(&pause);
                let worker = threads.spawn(move || {
                    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let (_authentication_scope, principal) = installation::authenticate(app);
                        let scope = authentication::WorthQueryRequestScope::new(
                            None,
                            cancellation.token(),
                            authentication::WorthQueryAdmissionClock::system(),
                        );
                        let request = app.request(&principal, &scope);
                        let source = request
                            .query(source::Read { key: 10 + i as u64 })
                            .execute()
                            .unwrap();
                        request
                            .advanced()
                            .mutate(edit::Input {
                                key: 10 + i as u64,
                                revision: 1,
                            })
                            .expect_source(source.observed_sources()[0].clone())
                            .idempotency(&(0x6b00_u64 + i as u64))
                            .execute_performed::<Program, Root>(
                                app,
                                ExecutionAllocationPolicy::SystemAllocation,
                            )
                    }));
                    // Early setup failure wakes the observer; unwind must release
                    // all parked actors before the enclosing scope joins them.
                    worker_pause.release();
                    match outcome.unwrap_or_else(|panic| std::panic::resume_unwind(panic)) {
                        Ok(WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
                            WorthQueryApplicationMutationOutcome::Commit(
                                WorthQueryApplicationUncommitted::Cancelled,
                            ),
                        )) => {}
                        Ok(WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
                            WorthQueryApplicationMutationOutcome::Commit(
                                WorthQueryApplicationUncommitted::Denied(cause),
                            ),
                        )) => panic!(
                            "registered actor denied at {:?}: {:?}",
                            cause.stage(),
                            cause.kind()
                        ),
                        Err(cause) => panic!("registered actor preparation failed: {cause:?}"),
                        _ => panic!(
                            "registered actor did not stop at its real cancellation boundary"
                        ),
                    }
                });
                attempts.push((pause, worker));
                assert!(attempts.last().unwrap().0.wait_for_arrivals());
            }
            assert_eq!(app.active_application_attempts_for_test(), live);
            observe()
        }));
        for cancellation in &cancellations {
            cancellation.cancel();
        }
        for (pause, _) in &attempts {
            pause.release();
        }
        for (_, worker) in attempts {
            worker
                .join()
                .expect("every cancelled registered actor is joined");
        }
        assert_eq!(app.active_application_attempts_for_test(), 0);
        observed.unwrap_or_else(|panic| std::panic::resume_unwind(panic))
    })
}
