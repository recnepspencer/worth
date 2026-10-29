//! Process isolation keeps live runtime identity an identical input at each width.
pub use super::super::{
    courtroom_support::{assert_authoritative_value, observe},
    schema::{IntentEffectField, IntentGateField},
    world::request_scope,
};
pub use super::*;
pub use std::process::Command;
pub use worth_query_host::facade::primary_graph::{
    principal_target_reads_on_this_thread_for_test as reads,
    signal_request_charges_on_this_thread_for_test as signal_charges,
};
pub const WIDTH: &str = "WORTH_QUERY_CUSTODY_DIFFERENTIAL_WIDTH";
pub const DIGEST: &str = "CUSTODY_DIGEST ";
pub const LEASED_DIGEST: &str = "LEASED_CUSTODY_DIGEST ";
pub const LAST_WORK: &str = "LAST_REQUEST_WORK ";

pub fn capture(resource_request: worth_execution::ExecutionRequest<'_, '_>, placement: Placement) {
    // Single-thread libtest writes its test-name prefix before the first record.
    println!();
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
