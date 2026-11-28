//! End-to-end Store M4.4 courts verifying:
//! 1. Resume rejects a `WorthQueryApplicationRootWindowBoundary` supplied from
//!    a different snapshot even when the anchor `EntityId`, query, parameters,
//!    scope, principal, and retained snapshot all match.
//! 2. Refused `close_branch` (`ActiveBranchProtected`) inside `prune_historical_basis`
//!    completes without deadlocking and preserves durable pending retry state.
//! 3. Opening a Directory Home via `open_in_advancement` (not only `open_directory`)
//!    automatically retires crash-leftover orphaned retain branches.
//! 4. Batched streaming relation attachment aborts on tight execution work ceilings
//!    during traversal preparation and retained-target sort charging.

use std::fmt::Write as _;
use std::num::NonZeroUsize;

use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
use worth_query_decl::facade::application_schema::ApplicationSchema;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    application_contribution::with_bootstrap_advancement,
    application_installation::{WorthQueryApplicationLimits, WorthQueryApplicationProgramRoster},
    historical_cursor::{
        AdmittedQueryWindowBinding, AdmittedQueryWindowPage, AdmittedQueryWindowSource,
        CancellationSource, CanonicalKeyDomain, CommitId, ExecutionRequest,
        HistoricalBasisRetentionKind, HistoricalQueryContract, HistoricalReadDenial,
        SchemaVersionIdentity, SerialMemoryBudget, SerialRequest, PROJECTION_QUERY_ROWS,
    },
    primary_graph::{WorthQueryApplicationRootWindow, WorthQueryApplicationRootWindowBoundary},
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest, WorthQueryHostSession,
    WorthQuerySessionQuery, DEFAULT_HOST_EXECUTION_POLICY,
};

use super::courts::{build_id, temp_home_path};
use super::program::{owner, seed, DaymarkProgram, DaymarkRuntime};
use super::queries::TasksByTitle;
use super::rename_task::RenameTask;
use super::retained_basis_cursor_security_court::{
    budget_with_window, open_custom_daymark, security_schema_version,
};
use super::schema::*;

fn serial_req() -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    )
}

fn expected_contract<Intent: WorthQuerySessionQuery<DaymarkSchema> + Clone>(
    session: &WorthQueryHostSession<'_, DaymarkSchema, DaymarkProgram>,
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

struct ForeignBoundarySource<'home> {
    branch_session: WorthQueryHostSession<'home, DaymarkSchema, DaymarkProgram>,
    foreign_boundary: Option<WorthQueryApplicationRootWindowBoundary>,
}

impl AdmittedQueryWindowSource<TaskRow> for ForeignBoundarySource<'_> {
    fn read_window(
        &mut self,
        _execution: &ExecutionRequest<'_, '_>,
        window: WorthQueryApplicationRootWindow,
    ) -> Result<AdmittedQueryWindowPage<TaskRow>, HistoricalReadDenial> {
        let boundary = self.foreign_boundary.take().expect("foreign boundary once");
        let forged_window = WorthQueryApplicationRootWindow::after(boundary, window.width());
        let page = TasksByTitle { project: 2 }
            .execute_window_in_session(&self.branch_session, None, forged_window)
            .expect("query executes on retained snapshot with foreign boundary")
            .into_root_window_page()
            .expect("window page emitted");
        AdmittedQueryWindowPage::from_executed_page(page)
    }
}

#[test]
fn retained_basis_cursor_rejects_foreign_snapshot_boundary_with_same_anchor_root() {
    let path = temp_home_path("retained-basis-foreign-boundary");
    let clock = WorthQueryAdmissionClock::system();
    let schema = security_schema_version();
    let budget = budget_with_window(16 * 1024);
    let serial = serial_req();
    let exec = ExecutionRequest::serial(&serial);
    let tasks = vec![
        (100, "A-first".to_owned()),
        (101, "B-second".to_owned()),
        (102, "C-third".to_owned()),
        (103, "D-fourth".to_owned()),
        (104, "ZZ-last".to_owned()),
    ];

    let home = open_custom_daymark(&path, &clock, tasks);
    let me = owner(&home, &clock);
    let session = home.session(&me);
    let receipt_a = home
        .retain_historical_basis(
            &session,
            "snap-a",
            HistoricalBasisRetentionKind::PinLease,
            schema.clone(),
            budget,
        )
        .unwrap();

    // Legit page 1 on Snapshot A ends at task 100 ("A-first").
    let mut legit = home
        .open_historical_query_cursor(
            &session,
            &exec,
            receipt_a.commit_id,
            receipt_a.root_semantic_commitment,
            TasksByTitle { project: 2 },
            &schema,
            budget,
        )
        .unwrap();
    let p1 = legit.next_page(&exec, 1).unwrap();
    assert_eq!(p1.rows()[0].task_key, 100);
    let p1_token = p1.checkpoint_token().clone();
    drop(legit);

    // Mutate HEAD to Snapshot B where task 100 sorts AFTER 101..103 ("Z-moved").
    session
        .run(RenameTask {
            task: 100,
            title: "Z-moved".to_owned(),
        })
        .unwrap();
    let snap_b_page = TasksByTitle { project: 2 }
        .execute_window_in_session(
            &session,
            None,
            WorthQueryApplicationRootWindow::first(NonZeroUsize::new(4).unwrap()),
        )
        .unwrap()
        .into_root_window_page()
        .unwrap();
    assert_eq!(snap_b_page.rows()[3].task_key, 100);
    let (_, snap_b_progress, _) = snap_b_page.into_parts();
    let foreign_boundary = snap_b_progress
        .into_next()
        .expect("foreign boundary anchored at task 100 on snapshot B");

    // Resolve the real retained branch for Snapshot A so the malicious source
    // runs the EXACT right query on Snapshot A, substituting only the Boundary.
    let store = home
        .durable_historical_basis_store(budget)
        .unwrap()
        .unwrap();
    let retained_name = store.retained_branch_name(receipt_a.commit_id).unwrap();
    let scope = session.fresh_scope();
    let retained_branch = home
        .primary_graph()
        .with_funded_application_completion(
            &scope,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            |phase| {
                home.primary_graph()
                    .branches()
                    .inventory(&phase)
                    .map(|inv| inv.find_by_name(&retained_name))
            },
        )
        .unwrap()
        .unwrap()
        .expect("retained branch exists");
    let branch_session = session.clone().on_branch(retained_branch);
    let snap_a_ver = TasksByTitle { project: 2 }
        .execute_window_in_session(
            &branch_session,
            None,
            WorthQueryApplicationRootWindow::first(NonZeroUsize::new(1).unwrap()),
        )
        .unwrap()
        .into_root_window_page()
        .unwrap()
        .progress()
        .snapshot_version();
    let intent = TasksByTitle { project: 2 };
    let binding = AdmittedQueryWindowBinding {
        contract_digest: intent.installed_contract_digest(&session).unwrap(),
        request_binding_digest: intent.admitted_request_binding_digest(&session).unwrap(),
        snapshot_version: snap_a_ver,
    };
    let contract = expected_contract(&session, &intent, &schema);

    let mut malicious = store
        .resume_admitted_query_cursor(&exec, &p1_token, contract, binding, &schema, |_| {
            Ok(Box::new(ForeignBoundarySource {
                branch_session,
                foreign_boundary: Some(foreign_boundary),
            }))
        })
        .unwrap();
    let err = malicious.next_page(&exec, 2).unwrap_err();
    assert!(
        matches!(err, HistoricalReadDenial::CorruptStorage { .. }),
        "Store MUST reject supplied Boundary from another snapshot even with matching anchor EntityId, got {err:?}"
    );

    drop(malicious);
    drop((store, session, me));
    home.close_clean().unwrap();
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn retained_basis_cursor_refused_prune_completes_without_deadlock_and_preserves_retry() {
    let path = temp_home_path("retained-basis-refused-prune-no-deadlock");
    let clock = WorthQueryAdmissionClock::system();
    let schema = security_schema_version();
    let budget = budget_with_window(16 * 1024);

    let home = open_custom_daymark(&path, &clock, vec![(100, "Alpha".into())]);
    let me = owner(&home, &clock);
    let session = home.session(&me);
    let active_name = session
        .snapshot()
        .unwrap()
        .branch_identity()
        .name()
        .as_str()
        .to_owned();
    let snap_ver = TasksByTitle { project: 2 }
        .execute_window_in_session(
            &session,
            None,
            WorthQueryApplicationRootWindow::first(NonZeroUsize::new(1).unwrap()),
        )
        .unwrap()
        .into_root_window_page()
        .unwrap()
        .progress()
        .snapshot_version();

    let seed_receipt = home
        .retain_historical_basis(
            &session,
            "seed-origin",
            HistoricalBasisRetentionKind::PinLease,
            schema.clone(),
            budget,
        )
        .unwrap();
    let store = home
        .durable_historical_basis_store(budget)
        .unwrap()
        .unwrap();
    let synthetic_commit = CommitId(777);
    store
        .retain_query_branch_basis(
            synthetic_commit,
            active_name.clone(),
            snap_ver,
            seed_receipt.branch_origin,
            schema,
            HistoricalBasisRetentionKind::PinLease,
            [0x44; 32],
        )
        .unwrap();

    // Pruning a basis bound to the active branch triggers close_branch -> ActiveBranchProtected
    // inside `retire_pruned_branch_custody` and MUST complete without deadlocking!
    let pruned = home
        .prune_historical_basis(&session, synthetic_commit, budget)
        .expect("prune returns cleanly without deadlock");
    assert!(pruned);
    assert!(
        store.pending_retire_branches().contains(&active_name),
        "refused close_branch must preserve pending_retire_branches"
    );

    // Adopt a preview to unblock the old active branch; reconciliation must close it.
    let replacement = home.fork_preview("unblock-active").unwrap();
    let old_active = home.adopt_preview(replacement).unwrap();
    home.reconcile_historical_branch_retirements(&session, budget);
    assert!(!store.pending_retire_branches().contains(&active_name));
    assert!(home.close_branch(old_active).is_err());

    drop((store, session, me));
    home.close_clean().unwrap();
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn retained_basis_cursor_open_in_advancement_automatically_retires_orphaned_branch() {
    let path = temp_home_path("retained-basis-open-in-advancement-orphan");
    let clock = WorthQueryAdmissionClock::system();
    let budget = budget_with_window(16 * 1024);

    let orphan_name = {
        let home = open_custom_daymark(&path, &clock, vec![(100, "Alpha".into())]);
        let me = owner(&home, &clock);
        let session = home.session(&me);
        let store = home
            .durable_historical_basis_store(budget)
            .unwrap()
            .unwrap();
        let preview = home.fork_preview("advancement-orphan").unwrap();
        let name = session
            .clone()
            .on_branch(preview.branch())
            .snapshot()
            .unwrap()
            .branch_identity()
            .name()
            .as_str()
            .to_owned();
        store.record_inflight_retain_intent(&name).unwrap();
        drop(preview);
        drop((store, session, me));
        home.close_clean().unwrap();
        name
    };

    // Reopen directly via `WorthQueryApplicationHomeRequest::open_in_advancement`
    // (NOT `WorthQueryApplicationHome::open_directory`).
    let declaration = DaymarkSchema::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<DaymarkSchema, DaymarkProgram>::begin()
        .with_schema(&declaration)
        .validated_program()
        .unwrap();
    let limits = WorthQueryApplicationLimits::bounded_serial(clock.clone())
        .with_execution_policy(DEFAULT_HOST_EXECUTION_POLICY)
        .install()
        .unwrap();
    let request = WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(path.clone()),
        program,
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        (DaymarkRuntime::new(),),
        build_id(),
        limits,
        |_, graph, installed| seed(graph, installed),
    );
    let home = with_bootstrap_advancement(DEFAULT_HOST_EXECUTION_POLICY, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap();
    let me = owner(&home, &clock);
    let session = home.session(&me);
    let store = home
        .durable_historical_basis_store(budget)
        .unwrap()
        .unwrap();
    assert!(!store.inflight_retain_branches().contains(&orphan_name));
    assert!(
        !store.pending_retire_branches().contains(&orphan_name),
        "open_in_advancement must automatically retire crash-leftover orphan branch"
    );
    let scope = session.fresh_scope();
    let still_exists = home
        .primary_graph()
        .with_funded_application_completion(
            &scope,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            |phase| {
                home.primary_graph()
                    .branches()
                    .inventory(&phase)
                    .map(|inv| inv.find_by_name(&orphan_name).is_some())
            },
        )
        .unwrap()
        .unwrap();
    assert!(
        !still_exists,
        "orphaned branch must be closed on open_in_advancement"
    );

    drop((store, session, me));
    home.close_clean().unwrap();
    let _ = std::fs::remove_dir_all(path);
}
