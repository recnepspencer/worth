//! End-to-end Store M4.4 `S5` retained-basis historical Query cursor court
//! over a real persistent Directory `WorthQueryApplicationHome` (`DaymarkHome`).

use std::path::Path;

use worth_foundational::facade::CanonicalDate;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    historical_cursor::{
        CancellationSource, Disposition, ExecutionRequest, HistoricalBasisRetentionKind,
        HistoricalCursor, HistoricalReadDenial, Reportable, ResidencyBudget,
        RetainedBasisTokenDecodeError, SchemaId, SchemaVersionId, SchemaVersionIdentity,
        SerialMemoryBudget, SerialRequest, RETAINED_BASIS_TOKEN_PREFIX,
    },
    primary_graph::*,
    WorthQueryApplicationHome, WorthQueryHomeConfig,
};

use super::actions::SetTaskState;
use super::courts::{build_id, temp_home_path};
use super::program::{owner, seed, DaymarkHome, DaymarkRuntime};
use super::queries::{BacklogTasks, ReadTaskDetails, ReadyTasks, TasksByPriority};
use super::schema::*;

const BACKLOG: std::ops::Range<u64> = 100..118;

fn open_daymark_backlog(path: &Path, clock: &WorthQueryAdmissionClock) -> DaymarkHome {
    WorthQueryApplicationHome::open_directory(
        path,
        WorthQueryHomeConfig::new(build_id(), (DaymarkRuntime::new(),))
            .with_clock(clock.clone())
            .with_initial_state(|_, graph, installed| {
                seed(graph, installed)?;
                let due = CanonicalDate::parse_iso("2026-12-01").unwrap();
                for task in BACKLOG {
                    let task_key =
                        || WorthQueryApplicationEntityKey::new(format!("task-{task}")).unwrap();
                    graph.bind_entity(
                        WorthQueryApplicationEntitySeed::new(Task::reference(), task_key())
                            .field(TaskKey::at("TaskKey"), task)
                            .field(Title::at("Title"), format!("Backlog {task}"))
                            .field(State::at("State"), "OPEN".to_owned())
                            .field(Priority::at("Priority"), task % 3)
                            .field(DueDate::at("DueDate"), due),
                    )?;
                    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                        Contains::reference(),
                        format!("contains-{task}"),
                        WorthQueryApplicationEntityKey::new("project-2".to_owned()).unwrap(),
                        task_key(),
                    ))?;
                }
                Ok(())
            }),
    )
    .expect("open_directory succeeds")
}

fn daymark_schema_version() -> SchemaVersionIdentity {
    SchemaVersionIdentity::new(
        SchemaId("daymark.unified.schema".into()),
        SchemaVersionId(1),
        [0x5D; 32],
    )
    .expect("valid schema version")
}

fn test_budget() -> ResidencyBudget {
    ResidencyBudget {
        head_working_set_bytes: 128 * 1024,
        historical_working_set_bytes: 64 * 1024,
        max_in_flight_fill_bytes: 32 * 1024,
        streaming_window_bytes: 16 * 1024,
    }
}

fn drain_query_cursor(
    cursor: &mut HistoricalCursor<'_, TaskRow>,
    exec: &ExecutionRequest<'_, '_>,
    page_size: usize,
) -> Vec<TaskRow> {
    let mut out = Vec::new();
    loop {
        let page = cursor.next_page(exec, page_size).expect("next page");
        let more = page.has_more();
        out.extend(page.into_rows());
        if !more {
            return out;
        }
    }
}

#[test]
fn query_home_retained_basis_cursor_resumes_exact_projected_rows_across_mutations_checkpoint_reopen_and_refuses_after_prune(
) {
    let path = temp_home_path("query-host-retained-basis-s5");
    let clock = WorthQueryAdmissionClock::system();
    let schema = daymark_schema_version();
    let budget = test_budget();
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let exec = ExecutionRequest::serial(&serial);

    let (receipt, asc_p1_rows, asc_wire, desc_p1_rows, desc_wire, ready_wire) = {
        let home = open_daymark_backlog(&path, &clock);
        let me = owner(&home, &clock);
        let session = home.session(&me);
        for task in [100, 105] {
            session
                .run(SetTaskState {
                    task,
                    state: "IN_PROGRESS".to_owned(),
                })
                .unwrap();
        }
        let receipt = home
            .retain_historical_basis(
                &session,
                "s5-state-a",
                HistoricalBasisRetentionKind::PinLease,
                schema.clone(),
                budget,
            )
            .expect("retain real Query/Relational basis");
        let (cid, comm) = (receipt.commit_id, receipt.root_semantic_commitment);

        let mut non_pageable = home
            .open_historical_query_cursor(
                &session,
                &exec,
                cid,
                comm,
                ReadTaskDetails { task: 100 },
                &schema,
                budget,
            )
            .expect("admit cursor");
        let np_err = non_pageable.next_page(&exec, 1).unwrap_err();
        assert!(matches!(
            np_err,
            HistoricalReadDenial::UnsupportedHistoricalMeaning { .. }
        ));
        assert_eq!(np_err.disposition(), Disposition::CorrectInput);
        drop(non_pageable);

        let mut asc_cur = home
            .open_historical_query_cursor(
                &session,
                &exec,
                cid,
                comm,
                BacklogTasks { project: 2 },
                &schema,
                budget,
            )
            .expect("open BacklogTasks cursor");
        assert!(asc_cur.loan_active());
        let asc_p1 = asc_cur.next_page(&exec, 5).expect("asc page 1");
        assert!(asc_p1.rows().len() == 5 && asc_p1.has_more());
        let asc_wire = asc_p1.checkpoint_token().encode_wire();
        assert!(asc_wire.starts_with(RETAINED_BASIS_TOKEN_PREFIX));
        let asc_p1_rows = asc_p1.into_rows();
        drop(asc_cur);

        let mut desc_cur = home
            .open_historical_query_cursor(
                &session,
                &exec,
                cid,
                comm,
                TasksByPriority { project: 2 },
                &schema,
                budget,
            )
            .expect("open TasksByPriority cursor");
        let desc_p1 = desc_cur.next_page(&exec, 3).expect("desc page 1");
        assert!(desc_p1.rows().len() == 3 && desc_p1.has_more());
        let desc_wire = desc_p1.checkpoint_token().encode_wire();
        let desc_p1_rows = desc_p1.into_rows();
        drop(desc_cur);

        let ready = ReadyTasks {
            project: 2,
            priority: Some(1),
            due_before: None,
        };
        let mut ready_cur = home
            .open_historical_query_cursor(&session, &exec, cid, comm, ready, &schema, budget)
            .expect("open ReadyTasks cursor");
        let ready_p1 = ready_cur.next_page(&exec, 2).expect("ready page 1");
        assert_eq!(ready_p1.rows().len(), 2);
        let ready_wire = ready_p1.checkpoint_token().encode_wire();
        drop(ready_cur);

        for task in 105..118 {
            session
                .run(SetTaskState {
                    task,
                    state: "DONE".to_owned(),
                })
                .unwrap();
        }
        home.checkpoint(&exec).expect("checkpoint after mutations");
        drop(session);
        drop(me);
        home.close_clean().expect("close clean before reopen");
        (
            receipt,
            asc_p1_rows,
            asc_wire,
            desc_p1_rows,
            desc_wire,
            ready_wire,
        )
    };

    // Cold reopen Directory Home: HEAD has DONE tasks; resumed tokens yield State A.
    {
        let home = open_daymark_backlog(&path, &clock);
        let me = owner(&home, &clock);
        let session = home.session(&me);
        let (cid, comm) = (receipt.commit_id, receipt.root_semantic_commitment);
        assert_eq!(
            session.query(ReadTaskDetails { task: 110 }).unwrap()[0].state,
            "DONE"
        );

        let bad_wire = home
            .resume_historical_query_cursor_wire(
                &session,
                &exec,
                "whc1_!!!",
                BacklogTasks { project: 2 },
                &schema,
                budget,
            )
            .err()
            .unwrap();
        assert_eq!(
            bad_wire,
            HistoricalReadDenial::MalformedToken(RetainedBasisTokenDecodeError::InvalidEncoding)
        );
        assert_eq!(bad_wire.disposition(), Disposition::CorrectInput);

        // Derived query contract binding: wrong scope, wrong query/ordering, wrong param ALL refuse!
        let check_meaning_refusal = |err: HistoricalReadDenial| {
            assert!(matches!(
                err,
                HistoricalReadDenial::UnsupportedHistoricalMeaning { .. }
            ));
            assert_eq!(err.disposition(), Disposition::CorrectInput);
        };
        check_meaning_refusal(
            home.resume_historical_query_cursor_wire(
                &session,
                &exec,
                &asc_wire,
                BacklogTasks { project: 1 },
                &schema,
                budget,
            )
            .err()
            .expect("wrong scope refuses"),
        );
        check_meaning_refusal(
            home.resume_historical_query_cursor_wire(
                &session,
                &exec,
                &asc_wire,
                TasksByPriority { project: 2 },
                &schema,
                budget,
            )
            .err()
            .expect("wrong query refuses"),
        );
        let wrong_param = ReadyTasks {
            project: 2,
            priority: Some(2),
            due_before: None,
        };
        check_meaning_refusal(
            home.resume_historical_query_cursor_wire(
                &session,
                &exec,
                &ready_wire,
                wrong_param,
                &schema,
                budget,
            )
            .err()
            .expect("wrong filter param refuses"),
        );

        // Resume ascending BacklogTasks -> exact 18 State A tasks.
        let mut resumed_asc = home
            .resume_historical_query_cursor_wire(
                &session,
                &exec,
                &asc_wire,
                BacklogTasks { project: 2 },
                &schema,
                budget,
            )
            .expect("resume ascending BacklogTasks");
        let mut asc_all = asc_p1_rows;
        asc_all.extend(drain_query_cursor(&mut resumed_asc, &exec, 4));
        drop(resumed_asc);
        assert_eq!(
            asc_all.iter().map(|r| r.task_key).collect::<Vec<_>>(),
            (100..118).collect::<Vec<_>>()
        );
        assert!(asc_all.iter().all(|r| {
            r.state
                == if r.task_key == 100 || r.task_key == 105 {
                    "IN_PROGRESS"
                } else {
                    "OPEN"
                }
        }));

        // Resume non-unique descending TasksByPriority -> exact 18 State A rows.
        let mut resumed_desc = home
            .resume_historical_query_cursor_wire(
                &session,
                &exec,
                &desc_wire,
                TasksByPriority { project: 2 },
                &schema,
                budget,
            )
            .expect("resume descending TasksByPriority");
        let mut desc_all = desc_p1_rows;
        desc_all.extend(drain_query_cursor(&mut resumed_desc, &exec, 4));
        drop(resumed_desc);
        assert_eq!(desc_all.len(), 18);
        assert!(desc_all.windows(2).all(|w| w[0].priority >= w[1].priority));
        let mut desc_keys: Vec<u64> = desc_all.iter().map(|r| r.task_key).collect();
        desc_keys.sort_unstable();
        assert_eq!(desc_keys, (100..118).collect::<Vec<_>>());

        let mut live_cur = home
            .open_historical_query_cursor(
                &session,
                &exec,
                cid,
                comm,
                BacklogTasks { project: 2 },
                &schema,
                budget,
            )
            .expect("open live cursor before prune");
        let live_p1 = live_cur.next_page(&exec, 5).expect("live page 1");
        assert_eq!(live_p1.rows().len(), 5);
        assert!(home
            .prune_historical_basis(&session, cid, budget)
            .expect("prune live"));
        let mut live_all = live_p1.into_rows();
        live_all.extend(drain_query_cursor(&mut live_cur, &exec, 4));
        assert_eq!(
            live_all.iter().map(|r| r.task_key).collect::<Vec<_>>(),
            (100..118).collect::<Vec<_>>()
        );
        drop(live_cur);
        home.checkpoint(&exec).expect("checkpoint after prune");
        drop((session, me));
        home.close_clean().expect("close clean after prune");
    }

    // Reopen after prune: resuming wire token MUST refuse HistoryUnavailable (Final).
    {
        let home = open_daymark_backlog(&path, &clock);
        let me = owner(&home, &clock);
        let session = home.session(&me);
        let err = home
            .resume_historical_query_cursor_wire(
                &session,
                &exec,
                &asc_wire,
                BacklogTasks { project: 2 },
                &schema,
                budget,
            )
            .err()
            .expect("pruned Query basis refuses after reopen");
        assert_eq!(
            err,
            HistoricalReadDenial::HistoryUnavailable {
                commit_id: receipt.commit_id
            }
        );
        assert_eq!(err.disposition(), Disposition::Final);
        drop((session, me));
        home.close_clean().expect("final close clean");
    }
    let _ = std::fs::remove_dir_all(path);
}
