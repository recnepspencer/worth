//! End-to-end Store M4.4 Query cursor security courts (Findings 1 & 3) on
//! persistent Directory `WorthQueryApplicationHome` (`DaymarkHome`):
//! 1. Page-1 and resume binding against mismatched Query or Snapshot receipts.
//! 2. Shared boundary-codec field limit rejecting >2048-byte text sort keys before checkpoint.

use std::fmt::Write as _;
use std::num::NonZeroUsize;
use std::path::Path;

use worth_foundational::facade::CanonicalDate;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    historical_cursor::{
        AdmittedQueryWindowBinding, AdmittedQueryWindowPage, AdmittedQueryWindowSource,
        CancellationSource, CanonicalKeyDomain, ExecutionRequest, HistoricalBasisRetentionKind,
        HistoricalQueryContract, HistoricalReadDenial, KnownProjectedHeapBytes, ResidencyBudget,
        SchemaId, SchemaVersionId, SchemaVersionIdentity, SerialMemoryBudget, SerialRequest,
        PROJECTION_QUERY_ROWS,
    },
    primary_graph::*,
    WorthQueryApplicationHome, WorthQueryHomeConfig, WorthQuerySessionQuery,
};

use super::actions::SetTaskState;
use super::courts::{build_id, temp_home_path};
use super::program::{owner, seed, DaymarkHome, DaymarkProgram, DaymarkRuntime};
use super::queries::{BacklogTasks, TasksByPriority, TasksByTitle};
use super::schema::*;

pub(super) fn open_custom_daymark(
    path: &Path,
    clock: &WorthQueryAdmissionClock,
    tasks: Vec<(u64, String)>,
) -> DaymarkHome {
    WorthQueryApplicationHome::open_directory(
        path,
        WorthQueryHomeConfig::new(build_id(), (DaymarkRuntime::new(),))
            .with_clock(clock.clone())
            .with_initial_state(move |_, graph, installed| {
                seed(graph, installed)?;
                let due = CanonicalDate::parse_iso("2026-12-01").unwrap();
                for (task, title) in &tasks {
                    let task_key =
                        || WorthQueryApplicationEntityKey::new(format!("task-{task}")).unwrap();
                    graph.bind_entity(
                        WorthQueryApplicationEntitySeed::new(Task::reference(), task_key())
                            .field(TaskKey::at("TaskKey"), *task)
                            .field(Title::at("Title"), title.clone())
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

pub(super) fn security_schema_version() -> SchemaVersionIdentity {
    SchemaVersionIdentity::new(
        SchemaId("daymark.security.schema".into()),
        SchemaVersionId(1),
        [0x5E; 32],
    )
    .expect("valid schema version")
}

pub(super) fn budget_with_window(streaming_window_bytes: u32) -> ResidencyBudget {
    ResidencyBudget {
        head_working_set_bytes: 128 * 1024,
        historical_working_set_bytes: 64 * 1024,
        max_in_flight_fill_bytes: 32 * 1024,
        streaming_window_bytes,
    }
}

fn expected_contract<Intent: WorthQuerySessionQuery<DaymarkSchema> + Clone>(
    session: &worth_query_host::facade::WorthQueryHostSession<'_, DaymarkSchema, DaymarkProgram>,
    intent: &Intent,
    schema: &SchemaVersionIdentity,
) -> HistoricalQueryContract {
    let digest = intent.derive_historical_query_digest(session).unwrap();
    let mut hex = String::with_capacity(64);
    for b in digest {
        let _ = write!(&mut hex, "{b:02x}");
    }
    HistoricalQueryContract::new(
        schema.clone(),
        CanonicalKeyDomain::Entity,
        None,
        PROJECTION_QUERY_ROWS,
        format!("query.binding.v3:{hex}"),
    )
}

struct OneShotPageSource<Row>(Option<WorthQueryApplicationRootWindowPage<Row>>);

impl<Row: KnownProjectedHeapBytes + Send> AdmittedQueryWindowSource<Row>
    for OneShotPageSource<Row>
{
    fn read_window(
        &mut self,
        _execution: &ExecutionRequest<'_, '_>,
        _window: WorthQueryApplicationRootWindow,
    ) -> Result<AdmittedQueryWindowPage<Row>, HistoricalReadDenial> {
        let page = self.0.take().expect("one-shot test page consumed once");
        AdmittedQueryWindowPage::from_executed_page(page)
    }
}

#[test]
fn retained_basis_cursor_rejects_mismatched_query_and_snapshot_receipts_on_page_one_and_resume() {
    let path = temp_home_path("retained-basis-receipt-binding");
    let clock = WorthQueryAdmissionClock::system();
    let schema = security_schema_version();
    let budget = budget_with_window(16 * 1024);
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let exec = ExecutionRequest::serial(&serial);
    let tasks: Vec<(u64, String)> = (100..106).map(|k| (k, format!("Task {k}"))).collect();

    let home = open_custom_daymark(&path, &clock, tasks);
    let me = owner(&home, &clock);
    let session = home.session(&me);

    // Mint a genuine executor page from TasksByPriority at State A (same snapshot, DIFFERENT query).
    let wrong_query_page_1 = TasksByPriority { project: 2 }
        .execute_window_in_session(
            &session,
            None,
            WorthQueryApplicationRootWindow::first(NonZeroUsize::new(2).unwrap()),
        )
        .unwrap()
        .into_root_window_page()
        .unwrap();
    let wrong_query_page_2 = TasksByPriority { project: 2 }
        .execute_window_in_session(
            &session,
            None,
            WorthQueryApplicationRootWindow::first(NonZeroUsize::new(2).unwrap()),
        )
        .unwrap()
        .into_root_window_page()
        .unwrap();
    let snap_a = wrong_query_page_1.progress().snapshot_version();

    let receipt_a = home
        .retain_historical_basis(
            &session,
            "snap-a",
            HistoricalBasisRetentionKind::PinLease,
            schema.clone(),
            budget,
        )
        .unwrap();

    // Advance HEAD to State B and mint genuine BacklogTasks pages at State B (same query, DIFFERENT snapshot).
    session
        .run(SetTaskState {
            task: 100,
            state: "DONE".to_owned(),
        })
        .unwrap();
    let wrong_snapshot_page_1 = BacklogTasks { project: 2 }
        .execute_window_in_session(
            &session,
            None,
            WorthQueryApplicationRootWindow::first(NonZeroUsize::new(2).unwrap()),
        )
        .unwrap()
        .into_root_window_page()
        .unwrap();
    let wrong_snapshot_page_2 = BacklogTasks { project: 2 }
        .execute_window_in_session(
            &session,
            None,
            WorthQueryApplicationRootWindow::first(NonZeroUsize::new(2).unwrap()),
        )
        .unwrap()
        .into_root_window_page()
        .unwrap();
    assert_ne!(wrong_snapshot_page_1.progress().snapshot_version(), snap_a);

    let store = home
        .durable_historical_basis_store(budget)
        .unwrap()
        .unwrap();
    let backlog = BacklogTasks { project: 2 };
    let backlog_contract = expected_contract(&session, &backlog, &schema);
    let backlog_binding = AdmittedQueryWindowBinding {
        contract_digest: backlog.installed_contract_digest(&session).unwrap(),
        request_binding_digest: backlog.admitted_request_binding_digest(&session).unwrap(),
        snapshot_version: snap_a,
    };

    // 1. Page 1 MUST reject a receipt from a different query.
    let mut cur_wrong_q = store
        .open_admitted_query_cursor(
            &exec,
            receipt_a.commit_id,
            receipt_a.root_semantic_commitment,
            backlog_contract.clone(),
            backlog_binding,
            &schema,
            |_| Ok(Box::new(OneShotPageSource(Some(wrong_query_page_1)))),
        )
        .unwrap();
    assert!(matches!(
        cur_wrong_q.next_page(&exec, 2).unwrap_err(),
        HistoricalReadDenial::UnsupportedHistoricalMeaning { .. }
    ));
    drop(cur_wrong_q);

    // 2. Page 1 MUST reject a receipt from a different snapshot.
    let mut cur_wrong_s = store
        .open_admitted_query_cursor(
            &exec,
            receipt_a.commit_id,
            receipt_a.root_semantic_commitment,
            backlog_contract.clone(),
            backlog_binding,
            &schema,
            |_| Ok(Box::new(OneShotPageSource(Some(wrong_snapshot_page_1)))),
        )
        .unwrap();
    assert!(matches!(
        cur_wrong_s.next_page(&exec, 2).unwrap_err(),
        HistoricalReadDenial::UnsupportedHistoricalMeaning { .. }
    ));
    drop(cur_wrong_s);

    // Mint a genuine page-1 checkpoint token for BacklogTasks on Snapshot A.
    let mut legit = home
        .open_historical_query_cursor(
            &session,
            &exec,
            receipt_a.commit_id,
            receipt_a.root_semantic_commitment,
            BacklogTasks { project: 2 },
            &schema,
            budget,
        )
        .unwrap();
    let p1_token = legit
        .next_page(&exec, 2)
        .unwrap()
        .checkpoint_token()
        .clone();
    drop(legit);

    // 3. Resume MUST reject a receipt from a different query.
    let mut res_wrong_q = store
        .resume_admitted_query_cursor(
            &exec,
            &p1_token,
            backlog_contract.clone(),
            backlog_binding,
            &schema,
            |_| Ok(Box::new(OneShotPageSource(Some(wrong_query_page_2)))),
        )
        .unwrap();
    assert!(matches!(
        res_wrong_q.next_page(&exec, 2).unwrap_err(),
        HistoricalReadDenial::UnsupportedHistoricalMeaning { .. }
    ));
    drop(res_wrong_q);

    // 4. Resume MUST reject a receipt from a different snapshot.
    let mut res_wrong_s = store
        .resume_admitted_query_cursor(
            &exec,
            &p1_token,
            backlog_contract,
            backlog_binding,
            &schema,
            |_| Ok(Box::new(OneShotPageSource(Some(wrong_snapshot_page_2)))),
        )
        .unwrap();
    assert!(matches!(
        res_wrong_s.next_page(&exec, 2).unwrap_err(),
        HistoricalReadDenial::UnsupportedHistoricalMeaning { .. }
    ));
    drop(res_wrong_s);

    drop((store, session, me));
    home.close_clean().unwrap();
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn retained_basis_cursor_paginates_large_text_sort_keys_with_compact_sealed_position() {
    let path = temp_home_path("retained-basis-large-sort-text");
    let clock = WorthQueryAdmissionClock::system();
    let schema = security_schema_version();
    let budget = budget_with_window(64 * 1024);
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let exec = ExecutionRequest::serial(&serial);
    // Task 100 sorts first ('A' < 'B' < 'C') and has a 2500-byte title.
    // Because cursors seal only the anchor EntityId inside Store (never serializing
    // sort-key text), pagination succeeds cleanly with a compact opaque token!
    let huge_title = "A_SORTKEY_PAYLOAD_".repeat(145);
    let tasks = vec![
        (100, huge_title.clone()),
        (101, "B-short".to_owned()),
        (102, "C-short".to_owned()),
    ];
    let wire = {
        let home = open_custom_daymark(&path, &clock, tasks.clone());
        let me = owner(&home, &clock);
        let session = home.session(&me);
        let receipt = home
            .retain_historical_basis(
                &session,
                "large-sort",
                HistoricalBasisRetentionKind::PinLease,
                schema.clone(),
                budget,
            )
            .unwrap();
        let mut cursor = home
            .open_historical_query_cursor(
                &session,
                &exec,
                receipt.commit_id,
                receipt.root_semantic_commitment,
                TasksByTitle { project: 2 },
                &schema,
                budget,
            )
            .unwrap();
        let p1 = cursor.next_page(&exec, 1).unwrap();
        assert_eq!(p1.rows().len(), 1);
        assert_eq!(p1.rows()[0].task_key, 100);
        assert!(p1.has_more());
        let wire = p1.checkpoint_token().encode_wire();
        assert!(
            wire.len() < 512,
            "sealed root-entity token must remain compact even for 2500-byte sort keys, got {} bytes",
            wire.len()
        );
        assert!(!wire.contains("A_SORTKEY_PAYLOAD_"));
        assert!(!wire.contains("QV9TT1JUS0VZX1BBWUxPQURf"));
        drop((cursor, session, me));
        home.close_clean().unwrap();
        wire
    };
    {
        let home = open_custom_daymark(&path, &clock, tasks);
        let me = owner(&home, &clock);
        let session = home.session(&me);
        let mut resumed = home
            .resume_historical_query_cursor_wire(
                &session,
                &exec,
                &wire,
                TasksByTitle { project: 2 },
                &schema,
                budget,
            )
            .unwrap();
        let p2 = resumed.next_page(&exec, 4).unwrap();
        let keys: Vec<u64> = p2.rows().iter().map(|r| r.task_key).collect();
        assert_eq!(keys, vec![101, 102]);
        assert!(!p2.has_more());
        drop((resumed, session, me));
        home.close_clean().unwrap();
    }
    let _ = std::fs::remove_dir_all(path);
}
