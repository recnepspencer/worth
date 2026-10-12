//! Process isolation keeps live runtime identity an identical input at each width.
use super::super::{
    courtroom_support::{assert_authoritative_value, observe},
    schema::{IntentEffectField, IntentGateField},
    world::request_scope,
};
use super::*;
use std::process::Command;
use worth_query_host::facade::primary_graph::{
    principal_target_reads_on_this_thread_for_test as reads,
    signal_request_charges_on_this_thread_for_test as signal_charges,
};

#[path = "width_differential/commit_work.rs"]
mod commit_work;
#[path = "width_differential/principal_target_read_work.rs"]
mod principal_target_read_work;

const WIDTH: &str = "WORTH_QUERY_CUSTODY_DIFFERENTIAL_WIDTH";
const DIGEST: &str = "CUSTODY_DIGEST ";
const LEASED_DIGEST: &str = "LEASED_CUSTODY_DIGEST ";
const LAST_WORK: &str = "LAST_REQUEST_WORK ";

#[test]
fn wake_and_reconsideration_keep_bits_and_order_with_exact_width_work() {
    let host_request = crate::test_request_policy::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    if let Ok(width) = std::env::var(WIDTH) {
        let placement = match width.as_str() {
            "0" => Placement::Serial,
            "1" => Placement::Leased(NonZeroUsize::MIN),
            "4" => Placement::Leased(NonZeroUsize::new(4).unwrap()),
            _ => panic!("the parent declares exactly three widths"),
        };
        capture(resource_request, placement);
        return;
    }
    let test = format!(
        "{}::wake_and_reconsideration_keep_bits_and_order_with_exact_width_work",
        module_path!().split_once("::").unwrap().1
    );
    let mut reference = None;
    let mut leased_reference = None;
    let mut serial_work = None;
    for width in ["0", "1", "4"] {
        let output = crate::process_deadline::output(
            Command::new(std::env::current_exe().unwrap())
                .args(["--exact", &test, "--nocapture"])
                .env(WIDTH, width),
        )
        .expect("run the same real fixture with fresh live identities");
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
        let actual_work = stdout
            .lines()
            .filter_map(|line| line.strip_prefix(LAST_WORK))
            .map(|line| {
                line.split(',')
                    .map(|units| units.parse::<u64>().unwrap())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(actual_work.len(), 2);
        if width == "0" {
            serial_work = Some(actual_work);
        } else {
            // One committed operation uses one caller's request. Every other
            // request must agree with Serial; only its checked native kernels
            // are absent from the unleased engine.
            let native_work = commit_work::quote(Placement::Leased(NonZeroUsize::MIN));
            for (actual, serial) in actual_work.iter().zip(serial_work.as_ref().unwrap()) {
                assert_eq!(actual.len(), serial.len());
                let differences = actual
                    .iter()
                    .zip(serial)
                    .map(|(actual, serial)| actual.checked_sub(*serial).unwrap())
                    .collect::<Vec<_>>();
                assert_eq!(differences.iter().sum::<u64>(), native_work);
                assert_eq!(differences.iter().filter(|work| **work != 0).count(), 1);
            }
        }
        if width != "0" {
            let actual = stdout
                .lines()
                .filter(|line| line.starts_with(LEASED_DIGEST))
                .map(str::to_owned)
                .collect::<Vec<_>>();
            assert_eq!(actual.len(), 2, "both leased semantic cases ran");
            if let Some(reference) = &leased_reference {
                assert_eq!(&actual, reference, "leased width {width}");
            } else {
                leased_reference = Some(actual);
            }
        }
    }
}

fn capture(resource_request: worth_execution::ExecutionRequest<'_, '_>, placement: Placement) {
    let _observer = worth_query_execution::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let _restore = Restore(place(placement), bound(None));
    for reconsider in [false, true] {
        let world = CourtroomWorld::publish(if reconsider { "blocked" } else { "ready" });
        if !reconsider {
            world.clock_control.push(1, 4);
        }
        reports();
        signal_charges();
        let before_reads = reads();
        let mut first = observe(&resource_request, &world);
        let first_work = reports()
            .into_iter()
            .map(|report| report.unwrap().charged_work())
            .collect::<Vec<_>>();
        assert!(!first_work.is_empty());
        let first_reads = reads() - before_reads;
        let first_signal: u64 = signal_charges().into_iter().sum();
        assert_eq!(
            first_work.iter().sum::<u64>(),
            first_signal + first_reads * principal_target_read_work::PRINCIPAL_TARGET_READ_WORK,
            "the first observation performs no native commit"
        );
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
            world
                .application
                .publish_native_field_write_for_test(
                    world.application.current_world(),
                    world.intent_record_identity(),
                    IntentGateField::reference(),
                    "ready".to_string(),
                    &request_scope(),
                )
                .expect("the native fixture write succeeds");
        } else {
            world.clock_control.push(2, 5);
        }
        reports();
        signal_charges();
        let before_reads = reads();
        let mut last = observe(&resource_request, &world);
        let last_work = reports()
            .into_iter()
            .map(|report| report.unwrap().charged_work())
            .collect::<Vec<_>>();
        assert!(!last_work.is_empty());
        let last_reads = reads() - before_reads;
        let last_signal: u64 = signal_charges().into_iter().sum();
        assert_eq!(
            last_work.iter().sum::<u64>(),
            last_signal
                + last_reads * principal_target_read_work::PRINCIPAL_TARGET_READ_WORK
                + commit_work::quote(placement),
            "every leased width pays the same checked native kernels"
        );
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
        if matches!(placement, Placement::Leased(_)) {
            println!(
                "{LEASED_DIGEST}{:?}",
                (
                    first_counts,
                    last_counts,
                    &provenance,
                    &first_work,
                    &last_work,
                    &order
                )
            );
        }
        println!(
            "{LAST_WORK}{}",
            last_work
                .iter()
                .map(u64::to_string)
                .collect::<Vec<_>>()
                .join(",")
        );
        println!(
            "{DIGEST}{:?}",
            (
                first_counts,
                last_counts,
                provenance,
                first_work,
                (first_signal, first_reads),
                (last_signal, last_reads),
                order
            )
        );
    }
}
