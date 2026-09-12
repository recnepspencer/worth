use std::sync::mpsc::{self, TryRecvError};
use std::time::Duration;

use worth_proof::NonEmpty;
use worth_store::physical_runtime::{
    PhysicalCurrentRootAdvanceOutcome, PhysicalMutationIdempotencyMaterial,
    PhysicalRootNamespaceDurabilityOutcome, PhysicalRootPublicationPreparationOutcome,
    PhysicalRootReplacementOutcome, RecordAppendBatch, RecordByteLimit, RecordReadLimits,
};

use super::super::super::{
    configuration, durable_publication::settle_single, serving_from_initialization,
};
use crate::read_record;

const PAYLOAD: &[u8] = b"canonical root Signal progression";

#[test]
fn root_candidate_signal_completion_precedes_replacement_and_current_root_advance() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let settled = settle_single(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([164; 32]),
        RecordAppendBatch::try_from_iter([PAYLOAD]).unwrap(),
    );
    let joined = submission
        .join_data_settled_group(settled.basis, NonEmpty::new(settled.member, Vec::new()))
        .unwrap_or_else(|rejected| panic!("exact group rejected: {:?}", rejected.cause()));

    let graph_nodes_before = serving
        .physical_signal_observation()
        .unwrap()
        .active_graph_node_count();
    let replacements_before = serving.media_counters().replacements();
    let signal_gate = serving.certification_pause_physical_signal_after_dequeue();
    let (prepared_tx, prepared_rx) = mpsc::sync_channel(1);
    let preparation = std::thread::spawn(move || {
        prepared_tx
            .send(submission.prepare_root_publication(joined))
            .unwrap();
    });
    if !signal_gate.await_arrivals(1) {
        signal_gate.release();
        let _ = preparation.join();
        panic!("root candidate synchronization did not traverse the physical Signal route");
    }

    assert!(matches!(prepared_rx.try_recv(), Err(TryRecvError::Empty)));
    assert_eq!(serving.media_counters().replacements(), replacements_before);
    signal_gate.release();
    let prepared = match prepared_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("root preparation must resume after Signal completion")
    {
        PhysicalRootPublicationPreparationOutcome::Prepared(prepared) => prepared,
        PhysicalRootPublicationPreparationOutcome::NotStarted(failure) => {
            panic!("root preparation did not start: {:?}", failure.cause())
        }
        PhysicalRootPublicationPreparationOutcome::InspectionRequired(failure) => {
            panic!(
                "root preparation became indeterminate: {:?}",
                failure.cause()
            )
        }
    };
    preparation.join().unwrap();
    let record = prepared.settled_members()[0].record_id(0).unwrap();
    assert_eq!(prepared.source_root_generation(), 1);
    assert_eq!(prepared.candidate_root_generation(), 2);
    assert_eq!(serving.media_counters().replacements(), replacements_before);

    let replaced = match serving
        .certification_record_submission()
        .replace_prepared_root(prepared)
    {
        PhysicalRootReplacementOutcome::Replaced(replaced) => replaced,
        PhysicalRootReplacementOutcome::NotStarted(failure) => {
            panic!("root replacement did not start: {:?}", failure.cause())
        }
        PhysicalRootReplacementOutcome::InspectionRequired(failure) => {
            panic!(
                "root replacement became indeterminate: {:?} {:?}",
                failure.effect_fate(),
                failure.recovery_disposition()
            )
        }
    };
    assert_eq!(
        serving.media_counters().replacements(),
        replacements_before + 3
    );
    let directory_syncs_before_namespace = serving.media_counters().directory_syncs();
    let durable = match serving
        .certification_record_submission()
        .synchronize_replaced_root_namespace(replaced)
    {
        PhysicalRootNamespaceDurabilityOutcome::Durable(durable) => durable,
        PhysicalRootNamespaceDurabilityOutcome::NotStarted(failure) => {
            panic!(
                "C5_PREDICATE:outcome-order: namespace synchronization did not start: {:?}",
                failure.cause()
            )
        }
        PhysicalRootNamespaceDurabilityOutcome::InspectionRequired(failure) => {
            panic!(
                "namespace synchronization became indeterminate: {:?} {:?}",
                failure.effect_fate(),
                failure.recovery_disposition()
            )
        }
    };
    assert_eq!(
        serving.media_counters().replacements(),
        replacements_before + 3,
        "namespace durability must not replay catalog replacement"
    );
    assert!(
        serving.media_counters().directory_syncs() > directory_syncs_before_namespace,
        "namespace durability requires directory synchronization"
    );
    let completed = match serving
        .certification_record_submission()
        .advance_namespace_durable_root(durable)
    {
        PhysicalCurrentRootAdvanceOutcome::Advanced(completed) => completed,
        PhysicalCurrentRootAdvanceOutcome::InspectionRequired(failure) => {
            panic!("current-root advance rejected: {:?}", failure.cause())
        }
    };
    assert_eq!(completed.current_root().generation(), 2);
    let session = serving
        .records()
        .expect("read protection admission")
        .open(
            record,
            RecordReadLimits::new(RecordByteLimit::new(PAYLOAD.len() as u32).unwrap()),
        )
        .unwrap();
    assert_eq!(read_record(session, PAYLOAD.len()).0, PAYLOAD);
    assert_eq!(
        serving
            .physical_signal_observation()
            .unwrap()
            .active_graph_node_count(),
        graph_nodes_before
    );
    serving.close();
}
