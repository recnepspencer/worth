//! Process isolation keeps live runtime identity an identical input at each width.
use super::super::{
    courtroom_support::{assert_authoritative_value, observe},
    schema::{IntentEffectField, IntentGateField},
    world::request_scope,
};
use super::*;
use std::{
    process::{Command, Stdio},
    time::{Duration, Instant},
};

const WIDTH: &str = "WORTH_QUERY_CUSTODY_DIFFERENTIAL_WIDTH";
const DIGEST: &str = "CUSTODY_DIGEST ";

#[test]
fn wake_and_reconsideration_keep_bits_work_and_delivery_order_across_widths() {
    if let Ok(width) = std::env::var(WIDTH) {
        let placement = match width.as_str() {
            "0" => Placement::Serial,
            "1" => Placement::Leased(NonZeroUsize::MIN),
            "4" => Placement::Leased(NonZeroUsize::new(4).unwrap()),
            _ => panic!("the parent declares exactly three widths"),
        };
        capture(placement);
        return;
    }
    let test = format!(
        "{}::wake_and_reconsideration_keep_bits_work_and_delivery_order_across_widths",
        module_path!().split_once("::").unwrap().1
    );
    let mut reference = None;
    for width in ["0", "1", "4"] {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", &test, "--nocapture"])
            .env(WIDTH, width)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("run the same real fixture with fresh live identities");
        let deadline = Instant::now() + Duration::from_secs(30);
        while child.try_wait().unwrap().is_none() {
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("the width fixture exceeded its assertion deadline");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "width {width}: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).unwrap();
        let actual = stdout
            .lines()
            .filter(|line| line.starts_with(DIGEST))
            .map(str::to_owned)
            .collect::<Vec<_>>();
        assert_eq!(actual.len(), 2, "both actual semantic cases ran");
        println!("width {width}: {actual:?}");
        if let Some(reference) = &reference {
            assert_eq!(&actual, reference, "width {width}");
        } else {
            reference = Some(actual);
        }
    }
}

fn capture(placement: Placement) {
    let _restore = Restore(place(placement), bound(None));
    for reconsider in [false, true] {
        let world = CourtroomWorld::publish(if reconsider { "blocked" } else { "ready" });
        if !reconsider {
            world.clock_control.push(1, 4);
        }
        reports();
        let mut first = observe(&world);
        let first_work = reports()
            .into_iter()
            .map(|report| report.unwrap().charged_work())
            .collect::<Vec<_>>();
        assert!(!first_work.is_empty());
        assert_eq!(first.committed_operation_count(), 0);
        if reconsider {
            assert_eq!(first.retained_suppressed_wake_count(), 1);
        }
        let first_counts = (
            first.due_wake_count(),
            first.retained_due_wake_count(),
            first.retained_suppressed_wake_count(),
        );
        let first_batch = first.take_granular_invalidation_batch();
        if reconsider {
            world.application.publish_native_field_write_for_test(
                world.application.current_world(),
                world.intent_record_identity(),
                IntentGateField::reference(),
                "ready".to_string(),
                &request_scope(),
            );
        } else {
            world.clock_control.push(2, 5);
        }
        reports();
        let mut last = observe(&world);
        let last_work = reports()
            .into_iter()
            .map(|report| report.unwrap().charged_work())
            .collect::<Vec<_>>();
        assert!(!last_work.is_empty());
        assert_eq!(last.committed_operation_count(), 1);
        assert_eq!(last.retained_due_wake_count(), 0);
        if reconsider {
            assert_eq!(last.authoritative_commit_count(), 1);
        }
        let last_counts = (
            last.due_wake_count(),
            last.authoritative_commit_count(),
            last.committed_operation_count(),
            last.retained_due_wake_count(),
        );
        let provenance = last
            .execution_provenance()
            .iter()
            .map(|item| {
                (
                    item.intent_identity().to_string(),
                    item.intent_revision(),
                    item.signal_decision(),
                )
            })
            .collect::<Vec<_>>();
        let last_batch = last.take_granular_invalidation_batch();
        if reconsider {
            assert_eq!(last_batch.observation().direct_truth_delivery_count(), 1);
            assert_eq!(
                last_batch.observation().signal_performed_delivery_count(),
                1
            );
        }
        let order = [first_batch, last_batch]
            .into_iter()
            .map(|batch| {
                batch
                    .bridge_deliveries()
                    .iter()
                    .map(|delivery| {
                        let change = delivery.correspondence_receipt().change_set();
                        (
                            change.dependency().dependency_ordinal(),
                            change
                                .changes()
                                .iter()
                                .map(|item| item.semantic_change().cloned())
                                .collect::<Vec<_>>(),
                            delivery.performed_signal().is_some(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        if reconsider {
            assert_eq!(order[1].len(), 1);
            assert!(
                order[1][0].2,
                "the direct delivery retains its performed Signal witness"
            );
        } else {
            assert!(order.iter().all(Vec::is_empty));
        }
        assert_authoritative_value(
            &world,
            IntentEffectField::reference(),
            "payload".to_string(),
        );
        println!(
            "{DIGEST}{:?}",
            (
                first_counts,
                last_counts,
                provenance,
                first_work,
                last_work,
                order
            )
        );
    }
}
