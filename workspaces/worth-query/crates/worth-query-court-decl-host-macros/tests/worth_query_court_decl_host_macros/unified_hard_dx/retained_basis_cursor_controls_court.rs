//! End-to-end Store M4.4 courts verifying:
//! 1. Bootstrap-to-Home advancement handoff during `open_in_advancement`
//!    preserves the opening request's controls (cancellation & serial backing)
//!    without spawning a background thread.
//! 2. `next_page` narrows a permissive caller `ExecutionRequest` by the
//!    session's configured work, memory, and cancellation controls.
//! 3. Unordered root windows charge identity-order heap-sort comparisons to the
//!    caller's execution work meter.

use std::num::NonZeroUsize;

use worth_foundational::facade::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
use worth_query_decl::facade::application_schema::ApplicationSchema;
use worth_query_decl::facade::{worth_query_identity_namespace, WorthQuery};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryAdmissionClock, WorthQueryCancellationSource},
    application_contribution::{
        with_bootstrap_advancement, with_bootstrap_advancement_from_request,
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    application_installation::{WorthQueryApplicationLimits, WorthQueryApplicationProgramRoster},
    historical_cursor::{
        CancellationSource, ExecutionRequest, HistoricalBasisRetentionKind, HistoricalReadDenial,
        SerialMemoryBudget, SerialRequest,
    },
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
    DEFAULT_HOST_EXECUTION_POLICY,
};

use super::courts::{build_id, temp_home_path};
use super::program::{owner, seed, DaymarkProgram, DaymarkRuntime};
use super::queries::BacklogTasks;
use super::retained_basis_cursor_security_court::{
    budget_with_window, open_custom_daymark, security_schema_version,
};
use super::schema::*;

const UNORDERED_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.unified.unordered-tasks.v1");

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = DaymarkSchema,
    context = DaymarkContext,
    namespace = UNORDERED_NS,
    result = TaskRow,
    root = Task,
    scope = ProjectKey,
    root_path(Contains),
    limits = 64,
)]
pub struct UnorderedTasks {
    #[worth_query(scope)]
    pub project: u64,
}

fn permissive_serial() -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    )
}

fn build_open_request(
    path: std::path::PathBuf,
    clock: &WorthQueryAdmissionClock,
) -> WorthQueryApplicationHomeRequest<'static, DaymarkSchema, DaymarkProgram> {
    let declaration = DaymarkSchema::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<DaymarkSchema, DaymarkProgram>::begin()
        .with_schema(&declaration)
        .validated_program()
        .unwrap();
    let limits = WorthQueryApplicationLimits::bounded_serial(clock.clone())
        .with_execution_policy(DEFAULT_HOST_EXECUTION_POLICY)
        .install()
        .unwrap();
    WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(path),
        program,
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        (DaymarkRuntime::new(),),
        build_id(),
        limits,
        |_, graph, installed| seed(graph, installed),
    )
}

#[test]
fn retained_basis_cursor_open_in_advancement_respects_opening_controls_for_orphan_retirement() {
    let path = temp_home_path("retained-basis-open-controls-orphan");
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
        let preview = home.fork_preview("controlled-orphan").unwrap();
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

    // Cancelled bootstrap opening request refuses cleanly and does not retire the orphan.
    let cancel_src = CancellationSource::new();
    cancel_src.cancel();
    let cancelled_serial =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancel_src.token(), None);
    let cancelled_exec = ExecutionRequest::serial(&cancelled_serial);
    let req_cancelled = build_open_request(path.clone(), &clock);
    let cancelled_res = with_bootstrap_advancement_from_request(cancelled_exec, |phase| {
        req_cancelled.open_in_advancement(&phase)
    });
    assert!(matches!(
        cancelled_res,
        Err(WorthQueryAdvancementDenial::Interrupted(
            WorthQueryManagedComputationInterruption::Cancelled
        ))
    ));

    // Reopening via serial `with_bootstrap_advancement_from_request` hands off
    // directly on the opening thread and retires the orphaned branch cleanly.
    let serial = permissive_serial();
    let exec = ExecutionRequest::serial(&serial);
    let req_live = build_open_request(path.clone(), &clock);
    let home =
        with_bootstrap_advancement_from_request(exec, |phase| req_live.open_in_advancement(&phase))
            .unwrap()
            .unwrap();
    let me = owner(&home, &clock);
    let session = home.session(&me);
    let store = home
        .durable_historical_basis_store(budget)
        .unwrap()
        .unwrap();
    assert!(!store.inflight_retain_branches().contains(&orphan_name));
    assert!(!store.pending_retire_branches().contains(&orphan_name));

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
    assert!(!still_exists);

    // Also verify ordinary `with_bootstrap_advancement` reopen remains clean.
    drop((store, session, me));
    home.close_clean().unwrap();
    let req_again = build_open_request(path.clone(), &clock);
    let home2 = with_bootstrap_advancement(DEFAULT_HOST_EXECUTION_POLICY, |phase| {
        req_again.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap();
    home2.close_clean().unwrap();
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn retained_basis_cursor_next_page_enforces_session_budget_under_permissive_caller_request() {
    let path = temp_home_path("retained-basis-session-budget-narrowing");
    let clock = WorthQueryAdmissionClock::system();
    let schema = security_schema_version();
    let budget = budget_with_window(16 * 1024);
    let serial = permissive_serial();
    let permissive_exec = ExecutionRequest::serial(&serial);
    let tasks: Vec<(u64, String)> = (100..108)
        .map(|k| (k, format!("Task-{k}-{}", "B".repeat(1024))))
        .collect();

    let home = open_custom_daymark(&path, &clock, tasks);
    let me = owner(&home, &clock);
    let base_session = home.session(&me);
    let receipt = home
        .retain_historical_basis(
            &base_session,
            "session-budget-pin",
            HistoricalBasisRetentionKind::PinLease,
            schema.clone(),
            budget,
        )
        .unwrap();

    // Tight session work ceiling must still bind `next_page` under permissive caller request.
    let base_mem = base_session.policy().charged_memory_bytes();
    let base_work = base_session.policy().work_ceiling();
    let tight_work_session = base_session.clone().with_budget(ExecutionBudget::new(
        NonZeroUsize::MIN,
        base_mem,
        Some(1),
    ));
    let mut work_cur = home
        .open_historical_query_cursor(
            &tight_work_session,
            &permissive_exec,
            receipt.commit_id,
            receipt.root_semantic_commitment,
            BacklogTasks { project: 2 },
            &schema,
            budget,
        )
        .unwrap();
    let work_err = work_cur.next_page(&permissive_exec, 4).unwrap_err();
    assert!(
        matches!(work_err, HistoricalReadDenial::RequestStopped),
        "tight session work cap must return RequestStopped, got {work_err:?}"
    );
    drop(work_cur);

    // Tight session memory ceiling must still bind `next_page` under permissive caller request.
    let tight_mem_session = base_session.clone().with_budget(ExecutionBudget::new(
        NonZeroUsize::MIN,
        Some(2048),
        base_work,
    ));
    let mut mem_cur = home
        .open_historical_query_cursor(
            &tight_mem_session,
            &permissive_exec,
            receipt.commit_id,
            receipt.root_semantic_commitment,
            BacklogTasks { project: 2 },
            &schema,
            budget,
        )
        .unwrap();
    let mem_err = mem_cur.next_page(&permissive_exec, 4).unwrap_err();
    assert!(
        matches!(mem_err, HistoricalReadDenial::RequestStopped),
        "tight session memory cap must return RequestStopped, got {mem_err:?}"
    );
    drop(mem_cur);

    // Session cancellation token must still stop `next_page` under permissive caller request.
    let session_cancel = WorthQueryCancellationSource::new();
    let cancel_session = base_session
        .clone()
        .with_cancellation(session_cancel.token());
    let mut cancel_cur = home
        .open_historical_query_cursor(
            &cancel_session,
            &permissive_exec,
            receipt.commit_id,
            receipt.root_semantic_commitment,
            BacklogTasks { project: 2 },
            &schema,
            budget,
        )
        .unwrap();
    session_cancel.cancel();
    assert!(matches!(
        cancel_cur.next_page(&permissive_exec, 4).unwrap_err(),
        HistoricalReadDenial::RequestStopped
    ));
    drop(cancel_cur);

    // Baseline session with the same permissive caller request succeeds cleanly.
    let mut ok_cur = home
        .open_historical_query_cursor(
            &base_session,
            &permissive_exec,
            receipt.commit_id,
            receipt.root_semantic_commitment,
            BacklogTasks { project: 2 },
            &schema,
            budget,
        )
        .unwrap();
    let page = ok_cur.next_page(&permissive_exec, 4).unwrap();
    assert_eq!(page.rows().len(), 4);
    drop(ok_cur);

    drop((base_session, me));
    home.close_clean().unwrap();
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn retained_basis_cursor_unordered_window_charges_identity_sort_comparisons_to_work_meter() {
    let path = temp_home_path("retained-basis-unordered-sort-work");
    let clock = WorthQueryAdmissionClock::system();
    let schema = security_schema_version();
    let budget = budget_with_window(64 * 1024);
    let tasks: Vec<(u64, String)> = (100..132).map(|k| (k, format!("Task-{k}"))).collect();

    let home = open_custom_daymark(&path, &clock, tasks);
    let me = owner(&home, &clock);
    let session = home.session(&me);
    let receipt = home
        .retain_historical_basis(
            &session,
            "unordered-pin",
            HistoricalBasisRetentionKind::PinLease,
            schema.clone(),
            budget,
        )
        .unwrap();

    // Measure unordered execution under permissive request first to verify full paging.
    let open_serial = permissive_serial();
    let open_exec = ExecutionRequest::serial(&open_serial);
    let mut full_cur = home
        .open_historical_query_cursor(
            &session,
            &open_exec,
            receipt.commit_id,
            receipt.root_semantic_commitment,
            UnorderedTasks { project: 2 },
            &schema,
            budget,
        )
        .unwrap();
    let mut all_keys = Vec::new();
    loop {
        let page = full_cur.next_page(&open_exec, 10).unwrap();
        let more = page.has_more();
        all_keys.extend(page.into_rows().into_iter().map(|r| r.task_key));
        if !more {
            break;
        }
    }
    assert_eq!(all_keys.len(), 32);
    drop(full_cur);

    // With 32 unordered roots, identity-order heap sort charges each comparison
    // to `ResultTreeWork`. A tight work ceiling (60 units) must abort during
    // identity-order sorting, while a generous ceiling succeeds.
    let tight_policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, None, Some(60)),
    );
    let tight_session = session.clone().with_policy(tight_policy);
    let mut tight_cur = home
        .open_historical_query_cursor(
            &tight_session,
            &open_exec,
            receipt.commit_id,
            receipt.root_semantic_commitment,
            UnorderedTasks { project: 2 },
            &schema,
            budget,
        )
        .unwrap();
    assert!(matches!(
        tight_cur.next_page(&open_exec, 4).unwrap_err(),
        HistoricalReadDenial::RequestStopped
    ));
    drop(tight_cur);

    drop((session, me));
    home.close_clean().unwrap();
    let _ = std::fs::remove_dir_all(path);
}
