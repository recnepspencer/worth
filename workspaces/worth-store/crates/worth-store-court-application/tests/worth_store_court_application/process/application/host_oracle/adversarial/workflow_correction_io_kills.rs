//! C4b.1: Real OS kills at durable `SynchronizeFileState` I/O barriers of
//! Governed Workflow execution (`CorrectableRetentionBinding`), verifying
//! fresh-process reopen, custody continuation, admitted-key retry, and
//! MemoryOnly reference parity through terminal `Historical` state.
use super::workflow_correction_fixture::{
    advance, discover_original_start, open_workflow_home, run_correctable_step,
    seed_approved_workflow, wf_input, WorkflowHome,
};
use super::{
    assert_all_reopen_phases_killed, classify_barrier_kill, run_parallel_barriers,
    sync_barrier_placement, BarrierKill, ReopenPhase, RestorableHome, TAIL_VARIANCE_BOUND,
};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use worth_execution::ExecutionRequest;
use worth_query_host::facade::{
    application_discovery::WorthQueryWorkflowInstanceRead,
    application_entry::{
        WorkflowProgressOutcome, WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationRequestExt, WorthQueryWorkflowInstanceDiscoveryIntent,
    },
    WorthQueryApplicationHomeLocation as Location,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    operator_identity::{authenticate_reviewer, request_scope},
    readback::read_retention,
    schema::DocumentRetentionSchema as Schema,
    workflow::governed_read::{AuditRead, GovernedDocumentWorkflow, SubjectRead},
};
use worth_query_testkit_host::facade::host::checkpoint_capture;
use worth_store::physical_runtime::certification::MediaOperationRole;

const KEY_ADV_DONE: u64 = 5013;

const CHILD: &str = "process::application::host_oracle::adversarial::workflow_correction_io_kills::workflow_correction_io_kill_child";
const ENV_HOME: &str = "WORTH_ADV_WFC_HOME";
const ENV_MARKERS: &str = "WORTH_ADV_WFC_MARKERS";
const ENV_ORDINAL: &str = "WORTH_ADV_WFC_ORDINAL";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WorkflowCut {
    is_historical: bool,
    retention: u64,
}
struct WorkflowRef {
    before_step: WorkflowCut,
    after_apply: WorkflowCut,
    after_historical: WorkflowCut,
}

fn observe_workflow_cut(
    home: &WorkflowHome,
    original: &WorthQueryWorkflowInstanceDiscoveryIntent<Schema>,
    ordinal: u64,
) -> WorkflowCut {
    WorkflowCut {
        is_historical: is_workflow_historical(home, original, ordinal),
        retention: read_retention(home.runtime().runtime(), home.runtime().current_world()),
    }
}

fn build_workflow_reference(execution: ExecutionRequest<'_, '_>) -> WorkflowRef {
    let (home, auth) = open_workflow_home(&Location::MemoryOnly, None, execution);
    let original = seed_approved_workflow(&home, &auth);
    let before_step = observe_workflow_cut(&home, &original, 0);
    assert!(matches!(run_correctable_step(&home, &original), Outcome::Committed { .. }));
    let after_apply = observe_workflow_cut(&home, &original, 0);
    verify_workflow_reaches_historical(&home, &original, 0);
    let after_historical = observe_workflow_cut(&home, &original, 0);
    assert!(!before_step.is_historical && !after_apply.is_historical && after_historical.is_historical);
    assert_ne!(before_step.retention, after_apply.retention);
    assert_eq!(after_apply.retention, after_historical.retention);
    drop(auth);
    drop(home.close(&execution).err().unwrap().resume().unwrap());
    WorkflowRef { before_step, after_apply, after_historical }
}

#[test]
fn governed_workflow_io_boundary_kill_sweep() {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = ExecutionRequest::leased(&lease);
    let reference = build_workflow_reference(execution);
    let wf_saved = RestorableHome::prepare(|loc| {
        let (home, auth) =
            open_workflow_home(&Location::Directory(loc.to_path_buf()), None, execution);
        let _ = seed_approved_workflow(&home, &auth);
        drop(auth);
        home.close(&execution).unwrap().reclaim();
    });
    let m1 = measure_syncs(&wf_saved);
    let m2 = measure_syncs(&wf_saved);
    assert!(m1.abs_diff(m2) <= TAIL_VARIANCE_BOUND, "workflow calibration spread {m1} vs {m2}");
    let total = m1.max(m2);
    let outcomes = run_parallel_barriers(
        "workflow",
        total,
        || {
            let w_lease = checkpoint_capture::lease(256 << 20);
            let w_exec = ExecutionRequest::leased(&w_lease);
            let mut slot = None;
            let saved = RestorableHome::prepare(|loc| {
                let (home, auth) =
                    open_workflow_home(&Location::Directory(loc.to_path_buf()), None, w_exec);
                slot = Some(seed_approved_workflow(&home, &auth));
                drop(auth);
                home.close(&w_exec).unwrap().reclaim();
            });
            (saved, slot.unwrap())
        },
        |(worker_home, w_original), ordinal| {
            let w_lease = checkpoint_capture::lease(256 << 20);
            let w_exec = ExecutionRequest::leased(&w_lease);
            worker_home.restore();
            let kill = kill_child(worker_home.location(), ordinal, total);
            let (home, auth) = open_workflow_home(
                &Location::Directory(worker_home.location().to_path_buf()), None, w_exec,
            );
            // Observe reopened lifecycle and retention BEFORE settling any unpublished custody.
            let reopened = observe_workflow_cut(&home, w_original, ordinal);
            let has_custody = !workflow_custody_handles(&home).is_empty();
            let phase = if has_custody {
                assert_eq!(
                    reopened, reference.before_step,
                    "workflow sync#{ordinal}: pending custody must keep published head at before_step"
                );
                settle_workflow_custody(&home, &w_exec);
                ReopenPhase::Custody
            } else if reopened == reference.before_step {
                ReopenPhase::PreOp
            } else if reopened == reference.after_apply {
                ReopenPhase::Committed
            } else {
                panic!("workflow sync#{ordinal}: illegal reopened cut {reopened:?}");
            };
            if matches!(kill, BarrierKill::CompletedAtTail { .. }) {
                assert_eq!(phase, ReopenPhase::Committed, "workflow sync#{ordinal}: completed tail must reopen at after_apply without custody");
            }
            let settled_cut = observe_workflow_cut(&home, w_original, ordinal);
            assert!(
                settled_cut == reference.before_step || settled_cut == reference.after_apply,
                "workflow sync#{ordinal}: illegal post-custody cut {settled_cut:?}"
            );
            if settled_cut == reference.before_step {
                assert!(matches!(
                    run_correctable_step(&home, w_original),
                    Outcome::Committed { .. } | Outcome::RecoveredCommitted(_) | Outcome::AlreadyCommitted(_)
                ));
                assert_eq!(
                    observe_workflow_cut(&home, w_original, ordinal),
                    reference.after_apply,
                    "workflow sync#{ordinal}: post-apply cut must match MemoryOnly before final advance"
                );
            }
            verify_workflow_reaches_historical(&home, w_original, ordinal);
            assert_eq!(
                observe_workflow_cut(&home, w_original, ordinal),
                reference.after_historical,
                "workflow sync#{ordinal}: terminal cut must match MemoryOnly after_historical"
            );
            drop(auth);
            home.close(&w_exec).unwrap().reclaim();
            (kill, phase)
        },
    );
    assert_all_reopen_phases_killed("workflow", &outcomes);
}

fn is_workflow_historical(
    home: &WorkflowHome,
    original: &WorthQueryWorkflowInstanceDiscoveryIntent<Schema>,
    ordinal: u64,
) -> bool {
    let runtime = home.runtime().runtime();
    let scope = request_scope();
    let reviewer = authenticate_reviewer(runtime.installed_schema(), &scope);
    let basis = runtime.request(&reviewer, &scope).retain_current_read().unwrap();
    let read = runtime
        .request(&reviewer, &scope)
        .at(&basis)
        .read_original_workflow::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
            home.vocabulary(), original, wf_input(), wf_input(),
        )
        .unwrap();
    match read {
        WorthQueryWorkflowInstanceRead::Historical { .. } => true,
        WorthQueryWorkflowInstanceRead::Live { .. } => false,
        WorthQueryWorkflowInstanceRead::Missing { .. } => {
            panic!("workflow sync#{ordinal}: seeded workflow instance disappeared")
        }
    }
}

fn measure_syncs(saved: &RestorableHome) -> u64 {
    saved.restore();
    let markers = tempfile::tempdir().unwrap();
    assert_eq!(
        crate::killed_child::run_and_kill(
            CHILD,
            &[
                (ENV_HOME, saved.location()),
                (ENV_MARKERS, markers.path()),
                (ENV_ORDINAL, Path::new("count")),
            ],
            &markers.path().join("ready"),
            &markers.path().join("finished"),
        ),
        crate::killed_child::ChildEnd::Finished
    );
    std::fs::read_to_string(markers.path().join("finished")).unwrap().trim().parse().unwrap()
}

fn kill_child(home_dir: &Path, ordinal: u64, total: u64) -> BarrierKill {
    let markers_dir = tempfile::tempdir().unwrap();
    let ord = PathBuf::from(ordinal.to_string());
    let ready = markers_dir.path().join("ready");
    let end = crate::killed_child::run_and_kill(
        CHILD,
        &[(ENV_HOME, home_dir), (ENV_MARKERS, markers_dir.path()), (ENV_ORDINAL, &ord)],
        &ready,
        &markers_dir.path().join("finished"),
    );
    classify_barrier_kill(
        end,
        &ready,
        MediaOperationRole::SynchronizeFileState,
        ordinal,
        total,
        &format!("workflow sync#{ordinal} of {total}"),
    )
}

#[test]
fn workflow_correction_io_kill_child() {
    let Some(location) = std::env::var_os(ENV_HOME).map(PathBuf::from) else {
        return;
    };
    let markers = PathBuf::from(std::env::var_os(ENV_MARKERS).unwrap());
    let ord_str = std::env::var(ENV_ORDINAL).unwrap();
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = ExecutionRequest::leased(&lease);
    let done = Arc::new(AtomicBool::new(false));
    let (placement, activation, watcher) = if ord_str == "count" {
        sync_barrier_placement(u64::MAX, None, None)
    } else {
        sync_barrier_placement(
            ord_str.parse().unwrap(),
            Some(markers.join("ready")),
            Some(done.clone()),
        )
    };
    let (home, auth) =
        open_workflow_home(&Location::Directory(location), Some(placement), execution);
    let original = discover_original_start(&home);
    activation.arm().unwrap();
    let _ = run_correctable_step(&home, &original);
    done.store(true, Ordering::SeqCst);
    if let Some(h) = watcher {
        let _ = h.join();
        let actual = activation.matching_operation_count();
        crate::killed_child::announce(&markers.join("finished"), format!("missed:actual={actual}").as_bytes());
        return;
    }
    let count = activation.matching_operation_count();
    drop(auth);
    home.close(&execution).unwrap().reclaim();
    crate::killed_child::announce(&markers.join("finished"), count.to_string().as_bytes());
}

fn workflow_custody_handles(
    home: &WorkflowHome,
) -> Vec<worth_runtime_world::facade::ProductUnpublishedRecoveryHandle> {
    home.runtime()
        .runtime()
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .iter()
        .map(|row| row.handle().clone())
        .collect()
}

fn settle_workflow_custody(home: &WorkflowHome, execution: &ExecutionRequest<'_, '_>) {
    for handle in workflow_custody_handles(home) {
        let rec = home.runtime().runtime().readmit_product_publication_recovery(&handle).unwrap();
        loop {
            match rec.continue_original_in_request(execution).unwrap() {
                worth_runtime_world::facade::WorldRecoveredContinuationOutcome::Pending(c) => {
                    drop(c)
                }
                worth_runtime_world::facade::WorldRecoveredContinuationOutcome::Published {
                    ..
                } => break,
                other => panic!("workflow custody continue: {other:?}"),
            }
        }
    }
}

fn verify_workflow_reaches_historical(
    home: &WorkflowHome,
    original: &WorthQueryWorkflowInstanceDiscoveryIntent<Schema>,
    ordinal: u64,
) {
    let runtime = home.runtime().runtime();
    let scope = request_scope();
    let reviewer = authenticate_reviewer(runtime.installed_schema(), &scope);
    let basis = runtime.request(&reviewer, &scope).retain_current_read().unwrap();
    let read = runtime
        .request(&reviewer, &scope)
        .at(&basis)
        .read_original_workflow::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
            home.vocabulary(), original, wf_input(), wf_input(),
        )
        .unwrap();
    drop(basis);
    match read {
        WorthQueryWorkflowInstanceRead::Historical { .. } => {}
        WorthQueryWorkflowInstanceRead::Live { instance, .. } => {
            let progress = advance(home, instance, KEY_ADV_DONE);
            assert!(
                matches!(progress, WorkflowProgressOutcome::Completed(_)),
                "workflow sync#{ordinal}: post-apply advance must complete, got {progress:?}"
            );
            let basis = runtime.request(&reviewer, &scope).retain_current_read().unwrap();
            let after = runtime
                .request(&reviewer, &scope)
                .at(&basis)
                .read_original_workflow::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
                    home.vocabulary(), original, wf_input(), wf_input(),
                )
                .unwrap();
            assert!(
                matches!(after, WorthQueryWorkflowInstanceRead::Historical { .. }),
                "workflow sync#{ordinal}: completed workflow must be historical"
            );
        }
        WorthQueryWorkflowInstanceRead::Missing { .. } => {
            panic!("workflow sync#{ordinal}: seeded workflow instance disappeared")
        }
    }
}
