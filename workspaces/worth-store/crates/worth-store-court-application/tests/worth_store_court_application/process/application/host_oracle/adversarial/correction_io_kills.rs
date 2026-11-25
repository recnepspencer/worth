//! C4b.1: Real OS kills at durable `SynchronizeFileState` I/O barriers of
//! Authored Correction (`reverse(&source)` via `select_coordinates` on a
//! reopened Directory Home), verifying fresh-process reopen, custody
//! continuation + in-same-open pin settlement on close, MemoryOnly parity,
//! and admitted-key retry without double derivation.
use super::correction_fixture::{open_label_home, scope, LabelHome};
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
use std::time::Duration;
use worth_execution::ExecutionRequest;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationCorrectionDenial as CorrDenial,
        WorthQueryApplicationCorrectionPreparation as Prepared,
        WorthQueryApplicationMutationOutcome as Outcome, WorthQueryApplicationRequestExt,
        WorthQueryCorrectionHistoryLimits,
    },
    primary_graph::{
        QueryCorrectionMaterialDenial, WorthQueryApplicationCommitReceipt,
        WorthQueryCorrectionDerivationDenial, WorthQueryCorrectionSourceCoordinates,
        WorthQueryRecordedCorrectionSource, WorthQueryRecoveredApplicationCommitReceipt,
    },
    runtime::ExecutionAllocationPolicy,
    WorthQueryApplicationHomeLocation as Location,
};
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture as label;
use worth_query_testkit_host::facade::host::checkpoint_capture;
use worth_relational::facade::history::RelationalCommitReceipt;
use worth_runtime_world::facade::WorldRecoveredContinuationOutcome;
use worth_store::physical_runtime::certification::MediaOperationRole;
use worth_store_testkit_application_oracle::facade::host_observation::projection::HeadProjection;

use label::correction_source::label_edit::LabelEdit;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReceiptIdentity {
    native: RelationalCommitReceipt,
    world_incarnation: u64,
    world_generation: u64,
    world_commit: u64,
}
impl ReceiptIdentity {
    fn from_live(r: &WorthQueryApplicationCommitReceipt) -> Self {
        let pub_ref = r.committed_product_publication();
        Self {
            native: r.commit_reference().clone(),
            world_incarnation: pub_ref.product_incarnation().ordinal(),
            world_generation: pub_ref.product_generation().get(),
            world_commit: pub_ref.composite_commit().ordinal(),
        }
    }
    fn from_recovered(r: &WorthQueryRecoveredApplicationCommitReceipt) -> Self {
        let head = HeadProjection::observe(r.new_product_head());
        Self {
            native: r.commit_reference().clone(),
            world_incarnation: head.incarnation,
            world_generation: head.generation,
            world_commit: head.commit,
        }
    }
}

const CHILD: &str =
    "process::application::host_oracle::adversarial::correction_io_kills::correction_io_kill_child";
const ENV_HOME: &str = "WORTH_ADV_CORR_HOME";
const ENV_MARKERS: &str = "WORTH_ADV_CORR_MARKERS";
const ENV_ORDINAL: &str = "WORTH_ADV_CORR_ORDINAL";
const REVERSE_KEY: &str = "adv-reverse-label";

type SummaryState = (String, String);

fn observe_summary(home: &LabelHome) -> SummaryState {
    let req = scope();
    let runtime = home.runtime();
    let principal = label::authentication::authenticate_external(runtime.runtime().installed_schema(), "alice", Duration::from_secs(120), &req);
    let rows = runtime.request(&principal, &req).query(label::AccountSummaryParameters).execute().unwrap();
    let row = &rows.rows()[0];
    (row.status.clone(), row.label.clone())
}

fn seed_label_edit(home: &LabelHome) -> WorthQueryCorrectionSourceCoordinates {
    let req = scope();
    let runtime = home.runtime();
    let principal = label::authentication::authenticate_external(runtime.runtime().installed_schema(), "alice", Duration::from_secs(120), &req);
    let Outcome::Committed { receipt, result } = runtime
        .request(&principal, &req).advanced()
        .mutate(LabelEdit { status: "open".into(), label: "changed".into() })
        .without_source().idempotency(&"original-label".to_owned())
        .execute_in_program(runtime, ExecutionAllocationPolicy::SystemAllocation).unwrap()
    else { panic!("seed LabelEdit must commit"); };
    assert_eq!(result.label, "changed");
    receipt.correction_source().coordinates()
}

fn build_correction_reference(execution: ExecutionRequest<'_, '_>) -> (SummaryState, SummaryState) {
    let home = open_label_home(Location::MemoryOnly, None, execution);
    let coords = seed_label_edit(&home);
    let before = observe_summary(&home);
    let req = scope();
    let limits = WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::new(8).unwrap());
    let runtime = home.runtime();
    let principal = label::authentication::authenticate_external(
        runtime.runtime().installed_schema(), "alice", Duration::from_secs(120), &req,
    );
    let source = runtime
        .request(&principal, &req)
        .corrections()
        .select_coordinates(&coords, limits)
        .unwrap();
    let Ok(Prepared::Prepared(p)) = runtime
        .request(&principal, &req)
        .corrections()
        .reverse(&source)
        .idempotency(REVERSE_KEY)
        .limits(limits)
        .prepare()
    else {
        panic!("MemoryOnly correction reference must prepare");
    };
    publish_primary(p, 0);
    drop(source);
    let after = observe_summary(&home);
    assert_ne!(before, after);
    drop(home.close(&execution).err().unwrap().resume().unwrap());
    (before, after)
}

#[test]
fn authored_correction_reverse_io_boundary_kill_sweep() {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = ExecutionRequest::leased(&lease);
    let (expected_before, expected_after) = build_correction_reference(execution);
    let mut coords_slot = None;
    let saved = RestorableHome::prepare(|loc| {
        let home = open_label_home(Location::Directory(loc.to_path_buf()), None, execution);
        coords_slot = Some(seed_label_edit(&home));
        home.close(&execution).unwrap().reclaim();
    });
    let coords = coords_slot.unwrap();
    let coords_bytes = serde_json::to_vec(&coords).unwrap();
    let m1 = measure_syncs(&saved, &coords_bytes);
    let m2 = measure_syncs(&saved, &coords_bytes);
    assert!(m1.abs_diff(m2) <= TAIL_VARIANCE_BOUND, "correction calibration spread {m1} vs {m2}");
    let total = m1.max(m2);
    let outcomes = run_parallel_barriers(
        "correction",
        total,
        || {
            let w_lease = checkpoint_capture::lease(256 << 20);
            let w_exec = ExecutionRequest::leased(&w_lease);
            let mut slot = None;
            let home = RestorableHome::prepare(|loc| {
                let h = open_label_home(Location::Directory(loc.to_path_buf()), None, w_exec);
                slot = Some(seed_label_edit(&h));
                h.close(&w_exec).unwrap().reclaim();
            });
            let c = slot.unwrap();
            let bytes = serde_json::to_vec(&c).unwrap();
            (home, c, bytes)
        },
        |(worker_home, w_coords, w_bytes), ordinal| {
            let w_lease = checkpoint_capture::lease(256 << 20);
            let w_exec = ExecutionRequest::leased(&w_lease);
            worker_home.restore();
            let kill = kill_child(worker_home.location(), w_bytes, ordinal, total);
            let phase = verify_correction_reopen(
                worker_home.location(), w_coords, (&expected_before, &expected_after), (ordinal, w_exec),
            );
            if matches!(kill, BarrierKill::CompletedAtTail { .. }) {
                assert_eq!(phase, ReopenPhase::Committed, "correction sync#{ordinal}: completed tail run must reopen committed without custody");
            }
            (kill, phase)
        },
    );
    assert_all_reopen_phases_killed("correction", &outcomes);
}

fn verify_correction_reopen(
    location: &Path,
    coords: &WorthQueryCorrectionSourceCoordinates,
    expected: (&SummaryState, &SummaryState),
    run: (u64, ExecutionRequest<'_, '_>),
) -> ReopenPhase {
    let (expected_before, expected_after) = expected;
    let (ordinal, execution) = run;
    let home = open_label_home(Location::Directory(location.to_path_buf()), None, execution);
    let reopened = observe_summary(&home);
    let has_custody = !correction_custody_handles(&home).is_empty();
    let phase = if has_custody {
        assert_eq!(reopened, *expected_before, "correction sync#{ordinal}: pending custody must keep head at predecessor");
        settle_custody(&home, &execution);
        ReopenPhase::Custody
    } else if reopened == *expected_before {
        ReopenPhase::PreOp
    } else if reopened == *expected_after {
        ReopenPhase::Committed
    } else {
        panic!("correction sync#{ordinal}: reopened summary {reopened:?} diverges from MemoryOnly");
    };
    let req = scope();
    let limits = WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::new(8).unwrap());
    let runtime = home.runtime();
    let principal = label::authentication::authenticate_external(
        runtime.runtime().installed_schema(), "alice", Duration::from_secs(120), &req,
    );
    let source = runtime
        .request(&principal, &req)
        .corrections()
        .select_coordinates(coords, limits)
        .unwrap();
    let (key, original_id) =
        settle_or_publish_reversal(&home, (&principal, &req, &source), (limits, ordinal));
    let repeat = runtime
        .request(&principal, &req)
        .corrections()
        .reverse(&source)
        .idempotency(&key)
        .limits(limits)
        .prepare()
        .unwrap();
    let repeat_id = match &repeat {
        Prepared::Settled(Outcome::AlreadyCommitted(r)) => ReceiptIdentity::from_live(r),
        Prepared::Settled(Outcome::RecoveredCommitted(r)) => ReceiptIdentity::from_recovered(r),
        Prepared::Settled(other) => panic!("correction sync#{ordinal}: repeat settled unexpectedly: {other:?}"),
        _ => panic!("correction sync#{ordinal}: repeat must return original receipt without re-deriving"),
    };
    assert_eq!(repeat_id, original_id, "correction sync#{ordinal}: retry receipt must match original Native & World identities");
    drop(repeat);
    assert_eq!(observe_summary(&home), *expected_after, "correction sync#{ordinal}: MemoryOnly parity");
    drop(source);
    home.close(&execution)
        .unwrap_or_else(|e| panic!("correction sync#{ordinal}: graceful close after phase={phase:?}: {e:?}"))
        .reclaim();
    phase
}

fn settle_or_publish_reversal(
    home: &LabelHome,
    ctx: (
        &worth_query_host::facade::admission::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<label::IdentityExecutionSchema>,
        &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
        &worth_query_host::facade::primary_graph::WorthQuerySelectedCorrectionSource<'_, label::IdentityExecutionSchema>,
    ),
    opts: (WorthQueryCorrectionHistoryLimits, u64),
) -> (String, ReceiptIdentity) {
    let (principal, req, source) = ctx;
    let (limits, ordinal) = opts;
    let runtime = home.runtime();
    match runtime
        .request(principal, req)
        .corrections()
        .reverse(source)
        .idempotency(REVERSE_KEY)
        .limits(limits)
        .prepare()
    {
        Ok(Prepared::Prepared(p)) => (REVERSE_KEY.to_owned(), publish_primary(p, ordinal)),
        Ok(Prepared::Settled(Outcome::AlreadyCommitted(r))) => {
            (REVERSE_KEY.to_owned(), ReceiptIdentity::from_live(&r))
        }
        Ok(Prepared::Settled(Outcome::RecoveredCommitted(r))) => {
            (REVERSE_KEY.to_owned(), ReceiptIdentity::from_recovered(&r))
        }
        Err(CorrDenial::Derivation(WorthQueryCorrectionDerivationDenial::Material(
            QueryCorrectionMaterialDenial::PreparationDischarged,
        ))) => {
            let fresh_key = format!("{REVERSE_KEY}-fresh-{ordinal}");
            let Ok(Prepared::Prepared(p)) = runtime
                .request(principal, req)
                .corrections()
                .reverse(source)
                .idempotency(&fresh_key)
                .limits(limits)
                .prepare()
            else {
                panic!("correction sync#{ordinal}: fresh key after discharged preparation");
            };
            (fresh_key, publish_primary(p, ordinal))
        }
        Ok(Prepared::Settled(other)) => panic!("correction sync#{ordinal}: settled {other:?}"),
        Ok(Prepared::PublicationPending) => panic!("correction sync#{ordinal}: pending"),
        Ok(Prepared::ProductUnpublished(_)) => panic!("correction sync#{ordinal}: unpublished"),
        Err(cause) => panic!("correction sync#{ordinal}: prepare denial {cause:?}"),
    }
}

fn publish_primary(
    prepared: worth_query_host::facade::application_entry::WorthQueryPreparedApplicationCorrection<'_>,
    ordinal: u64,
) -> ReceiptIdentity {
    match prepared.publish().unwrap() {
        Outcome::Committed { receipt, result } => {
            assert_eq!(result.downcast_ref::<LabelEdit>().unwrap().label, "primary");
            ReceiptIdentity::from_live(&receipt)
        }
        Outcome::AlreadyCommitted(r) => ReceiptIdentity::from_live(&r),
        Outcome::RecoveredCommitted(r) => ReceiptIdentity::from_recovered(&r),
        other => panic!("correction sync#{ordinal}: unexpected publish outcome {other:?}"),
    }
}

fn correction_custody_handles(
    home: &LabelHome,
) -> Vec<worth_runtime_world::facade::ProductUnpublishedRecoveryHandle> {
    home.runtime().runtime().product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap()).unwrap()
        .rows().iter().map(|row| row.handle().clone()).collect()
}

fn settle_custody(home: &LabelHome, execution: &ExecutionRequest<'_, '_>) {
    for handle in correction_custody_handles(home) {
        let rec = home.runtime().runtime().readmit_product_publication_recovery(&handle).unwrap();
        loop {
            match rec.continue_original_in_request(execution).unwrap() {
                WorldRecoveredContinuationOutcome::Pending(c) => drop(c),
                WorldRecoveredContinuationOutcome::Published { .. } => break,
                other => panic!("correction custody continue: {other:?}"),
            }
        }
    }
}

fn measure_syncs(saved: &RestorableHome, coords: &[u8]) -> u64 {
    saved.restore();
    let markers = tempfile::tempdir().unwrap();
    std::fs::write(markers.path().join("coords.json"), coords).unwrap();
    let env = [(ENV_HOME, saved.location()), (ENV_MARKERS, markers.path()), (ENV_ORDINAL, Path::new("count"))];
    assert_eq!(
        crate::killed_child::run_and_kill(CHILD, &env, &markers.path().join("ready"), &markers.path().join("finished")),
        crate::killed_child::ChildEnd::Finished
    );
    std::fs::read_to_string(markers.path().join("finished")).unwrap().trim().parse().unwrap()
}

fn kill_child(location: &Path, coords: &[u8], ordinal: u64, total: u64) -> BarrierKill {
    let markers = tempfile::tempdir().unwrap();
    std::fs::write(markers.path().join("coords.json"), coords).unwrap();
    let ord = PathBuf::from(ordinal.to_string());
    let ready = markers.path().join("ready");
    let env = [(ENV_HOME, location), (ENV_MARKERS, markers.path()), (ENV_ORDINAL, &ord)];
    let end = crate::killed_child::run_and_kill(CHILD, &env, &ready, &markers.path().join("finished"));
    classify_barrier_kill(end, &ready, MediaOperationRole::SynchronizeFileState, ordinal, total, &format!("correction sync#{ordinal} of {total}"))
}

#[test]
fn correction_io_kill_child() {
    let Some(location) = std::env::var_os(ENV_HOME).map(PathBuf::from) else { return; };
    let markers = PathBuf::from(std::env::var_os(ENV_MARKERS).unwrap());
    let ord_str = std::env::var(ENV_ORDINAL).unwrap();
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = ExecutionRequest::leased(&lease);
    let done = Arc::new(AtomicBool::new(false));
    let (placement, activation, watcher) = if ord_str == "count" {
        sync_barrier_placement(u64::MAX, None, None)
    } else {
        sync_barrier_placement(ord_str.parse().unwrap(), Some(markers.join("ready")), Some(done.clone()))
    };
    let home = open_label_home(Location::Directory(location), Some(placement), execution);
    let coords: WorthQueryCorrectionSourceCoordinates =
        serde_json::from_slice(&std::fs::read(markers.join("coords.json")).unwrap()).unwrap();
    let req = scope();
    let limits = WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::new(8).unwrap());
    let runtime = home.runtime();
    let principal = label::authentication::authenticate_external(
        runtime.runtime().installed_schema(), "alice", Duration::from_secs(120), &req,
    );
    let source = runtime.request(&principal, &req).corrections().select_coordinates(&coords, limits).unwrap();
    activation.arm().unwrap();
    if let Ok(Prepared::Prepared(p)) = runtime
        .request(&principal, &req).corrections().reverse(&source).idempotency(REVERSE_KEY).limits(limits).prepare()
    {
        let _ = p.publish();
    }
    drop(source);
    done.store(true, Ordering::SeqCst);
    if let Some(h) = watcher {
        let _ = h.join();
        let actual = activation.matching_operation_count();
        crate::killed_child::announce(&markers.join("finished"), format!("missed:actual={actual}").as_bytes());
        return;
    }
    let count = activation.matching_operation_count();
    home.close(&execution).unwrap().reclaim();
    crate::killed_child::announce(&markers.join("finished"), count.to_string().as_bytes());
}
