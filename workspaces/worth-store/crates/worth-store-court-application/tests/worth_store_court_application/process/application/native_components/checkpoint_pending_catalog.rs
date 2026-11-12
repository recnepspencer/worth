//! Actual selected DATA must not accept an unpublished catalog before ACK completes.
use super::checkpoint::select_before_native_ack;
use worth_execution::{CancellationSource, ExecutionRequest, SerialRequest};
use worth_relational::facade::history::BranchId;

#[test]
fn pending_selected_data_keeps_the_unaccepted_catalog_in_the_next_publication() {
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    use worth_relational::facade::durability::RelationalCheckpointAcknowledgementProgress as Progress;
    use worth_relational::facade::indexes::{
        DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind,
    };
    let directory = tempfile::tempdir().unwrap();
    let host = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let fixture = super::coupled_home::workload(directory.path(), true, &execution);
    let home = fixture.home;
    let runtime = home.relational().runtime();
    let branch = BranchId("main".into());
    let seed =
        crate::workload::commit(runtime, &execution, &branch, crate::batches::create(1)).unwrap();
    runtime
        .snapshots()
        .release_snapshot(&seed.snapshot)
        .unwrap();
    home.checkpoint(&execution).unwrap();
    let next =
        crate::workload::commit(runtime, &execution, &branch, crate::batches::create(2)).unwrap();
    runtime
        .snapshots()
        .release_snapshot(&next.snapshot)
        .unwrap();
    let definition = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "pending-selected-definition".into(),
            branch_scoped: true,
            kind: DerivedIndexKind::EntityField {
                field_locator: worth_relational::facade::transactions::planned_single_field_locator(
                    crate::batches::aspect_key("name"),
                    crate::batches::field_key("name"),
                ),
            },
        })
        .unwrap();
    let selected = select_before_native_ack(&home, || {});
    let work = selected.capture.covered_position().unwrap().0 + 1;
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(64 << 20), Some(work)),
    );
    let limited = SerialRequest::from_policy(&policy, CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&limited);
    let stopped = home.recover_checkpoint(&request).unwrap_err();
    let worth_store_application::ApplicationHomeCheckpointDenial::RecoveryRequired(observer) =
        stopped
    else {
        panic!("actual Home must retain its pending original ACK: {stopped:?}")
    };
    assert!(!observer.acknowledgement_complete);
    assert_eq!(observer.selected_generation, Some(selected.generation));
    drop(observer);
    assert_eq!(
        selected.capture.acknowledgement_progress().unwrap(),
        Some(Progress::RetiringSources)
    );
    // DATA placement leaves derived state cold; reconstruct its exact predecessor
    // through the index owner without completing catalog acceptance.
    let built = runtime.index_authority().build_for_commit(
        &execution,
        worth_relational::facade::indexes::DerivedIndexBuildRequest {
            source_commit_id: next.commit.commit_id,
            branch_id: next.commit.branch_id.clone(),
            index_ids: vec![definition.index_id],
        },
    );
    assert!(built.execution_denial.is_none(), "{built:?}");
    assert!(built.failed_indexes.is_empty(), "{built:?}");
    assert_eq!(
        built
            .generations
            .iter()
            .map(|generation| generation.index_id)
            .collect::<Vec<_>>(),
        vec![definition.index_id]
    );
    let later =
        crate::workload::commit(runtime, &execution, &branch, crate::batches::create(3)).unwrap();
    let canonical = runtime
        .replay()
        .canonical_commit_envelope(&execution, later.commit.commit_id)
        .unwrap()
        .unwrap();
    let wire: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&canonical).unwrap()).unwrap();
    let expected: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&definition).unwrap()).unwrap();
    assert_eq!(
        wire["index_definition_delta"]["definitions"],
        serde_json::Value::Array(vec![expected])
    );
    runtime
        .snapshots()
        .release_snapshot(&later.snapshot)
        .unwrap();
    let completed = home.recover_checkpoint(&execution).unwrap();
    assert_eq!(completed.generation(), selected.generation);
    assert_eq!(
        selected.capture.acknowledgement_progress().unwrap(),
        Some(Progress::Complete)
    );
    let decoded =
        worth_relational::facade::durability::RelationalNativeCheckpoint::from_untrusted_bytes(
            selected.capture.bytes(),
        );
    assert_eq!(decoded.acknowledgement_progress().unwrap(), None);
    drop(selected);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("World close: {:?}", refused.reason()));
}
