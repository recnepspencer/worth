//! C3.6 real process-kill court: kill after prepared-correction pin transfer
//! (`PreparedCommand` -> `Publication { original }`) and before `Capture`.
//! Reopen must durably abandon the never-captured publication pin (never wedging
//! on `MaterialNotReady`), preserve zero correction effect and zero orphan pins
//! across two reopens, and keep the selected source usable for fresh reversal.
use super::*;
use crate::process::host::durable_retry::child_reexecution::{
    drive_test, expected, location, wait_for_kill,
};
use worth_foundational::facade::WorldOwnerCoordinate;
use worth_query_host::facade::primary_graph::WorthQueryCorrectionSourceCoordinates;

const TEST_PATH: &str = "host::application_home::correction_material::kill_pin_transfer::kill_after_pin_transfer_before_capture_abandons_orphan_pin_and_keeps_source_usable";
const REVERSE_KEY: &str = "reverse-pin-transfer-kill";

#[derive(serde::Serialize, serde::Deserialize)]
struct PinTransferRecord {
    source_commit_id: u64,
    baseline_version: u64,
}

fn read_state(home: &Home) -> (String, u64) {
    let request = scope();
    let runtime = home.runtime();
    let principal = label::authentication::authenticate_external(
        runtime.runtime().installed_schema(),
        "alice",
        Duration::from_secs(120),
        &request,
    );
    let rows = runtime
        .request(&principal, &request)
        .query(label::AccountSummaryParameters)
        .execute()
        .unwrap();
    assert_eq!(rows.rows().len(), 1);
    (
        rows.rows()[0].label.clone(),
        rows.receipt().inspect().basis().version(),
    )
}

fn replay_source_coordinates(
    home: &Home,
    expected_commit_id: u64,
) -> WorthQueryCorrectionSourceCoordinates {
    let request = scope();
    let runtime = home.runtime();
    let principal = label::authentication::authenticate_external(
        runtime.runtime().installed_schema(),
        "alice",
        Duration::from_secs(120),
        &request,
    );
    let Outcome::RecoveredCommitted(replayed) = runtime
        .request(&principal, &request)
        .advanced()
        .mutate(LabelEdit {
            status: "open".into(),
            label: "changed".into(),
        })
        .without_source()
        .idempotency(&"original-label".to_owned())
        .execute_in_program(
            runtime,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
    else {
        panic!("original-label key must replay RecoveredCommitted");
    };
    assert_eq!(replayed.commit_reference().commit_id.0, expected_commit_id);
    let durable = replayed.new_product_head().durable().unwrap();
    WorthQueryCorrectionSourceCoordinates::new(
        WorldOwnerCoordinate::Home {
            owner: durable.branch().owner(),
        },
        durable.selected_commit().ordinal(),
        replayed.commit_reference().clone(),
        worth_query_host::facade::primary_graph::WorthQueryCorrectionSourceMember::Primary,
    )
}

#[test]
fn kill_after_pin_transfer_before_capture_abandons_orphan_pin_and_keeps_source_usable() {
    drive_test(TEST_PATH, writer, reader);
}

fn writer() {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open(location(), execution);
    let receipt = edit(&home);
    let (label, baseline_version) = read_state(&home);
    assert_eq!(label, "changed");
    let record = PinTransferRecord {
        source_commit_id: receipt.commit_reference().commit_id.0,
        baseline_version,
    };
    std::fs::write(expected(), serde_json::to_vec(&record).unwrap()).unwrap();
    let request = scope();
    let runtime = home.runtime();
    let principal = label::authentication::authenticate_external(
        runtime.runtime().installed_schema(),
        "alice",
        Duration::from_secs(120),
        &request,
    );
    let limits =
        WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::new(8).unwrap());
    let source = runtime
        .request(&principal, &request)
        .corrections()
        .select(&receipt, limits)
        .unwrap();
    drop(receipt);
    let Prepared::Prepared(prepared) = runtime
        .request(&principal, &request)
        .corrections()
        .reverse(&source)
        .idempotency(REVERSE_KEY)
        .limits(limits)
        .prepare()
        .unwrap()
    else {
        panic!("prepared reversal must prepare before publish");
    };
    // Park immediately after `Transfer(PreparedCommand -> Publication)` is durable
    // in `query.corrections`, before `Capture` runs.
    runtime
        .runtime()
        .observe_next_correction_pin_transfer_for_test(Box::new(|| {
            wait_for_kill();
        }));
    let _ = prepared.publish();
    panic!("writer must be killed at the post-transfer pre-capture seam");
}

fn reader() {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let record: PinTransferRecord =
        serde_json::from_slice(&std::fs::read(expected()).unwrap()).unwrap();
    let limits =
        WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::new(8).unwrap());

    // Reopen #1: cold readmission releases the zero-effect World open and durably
    // abandons the never-captured Publication pin without wedging on MaterialNotReady.
    let home = open(location(), execution);
    assert_eq!(
        read_state(&home),
        ("changed".to_owned(), record.baseline_version)
    );
    assert_eq!(count(&home, &execution), 1);
    // Graceful close calls finish_retirement -> discharge_unperformed(None), which
    // would refuse PublicationPending if any orphan Publication pin survived.
    home.close(&execution)
        .unwrap_or_else(|refused| {
            panic!("first reopen close must succeed with no orphan pin: {refused:?}")
        })
        .reclaim();

    // Reopen #2: confirm zero correction effect and zero orphan pin persist, that
    // the selected source remains usable under the original key, and that after
    // fresh reversal completes, optional expiry succeeds (pin_count == 0).
    let home2 = open(location(), execution);
    assert_eq!(
        read_state(&home2),
        ("changed".to_owned(), record.baseline_version)
    );
    assert_eq!(count(&home2, &execution), 1);
    let coordinates = replay_source_coordinates(&home2, record.source_commit_id);
    {
        let request = scope();
        let runtime = home2.runtime();
        let principal = label::authentication::authenticate_external(
            runtime.runtime().installed_schema(),
            "alice",
            Duration::from_secs(120),
            &request,
        );
        let source = runtime
            .request(&principal, &request)
            .corrections()
            .select_coordinates(&coordinates, limits)
            .unwrap();
        let Prepared::Prepared(prepared) = runtime
            .request(&principal, &request)
            .corrections()
            .reverse(&source)
            .idempotency(REVERSE_KEY)
            .limits(limits)
            .prepare()
            .unwrap()
        else {
            panic!("abandoned correction key must prepare fresh after two reopens");
        };
        match prepared.publish().unwrap() {
            Outcome::Committed { result, .. } => {
                assert_eq!(result.downcast_ref::<LabelEdit>().unwrap().label, "primary");
            }
            other => panic!("fresh reversal after abandoned pin transfer must commit: {other:?}"),
        }
        drop(source);
        // Optional expiry succeeds only when zero orphan pins remain on the source.
        runtime
            .expire_optional_correction_material(&execution, &coordinates)
            .expect("no orphan pin remains on the selected source");
    }
    home2
        .close(&execution)
        .unwrap_or_else(|refused| panic!("second reopen close: {refused:?}"))
        .reclaim();
}
