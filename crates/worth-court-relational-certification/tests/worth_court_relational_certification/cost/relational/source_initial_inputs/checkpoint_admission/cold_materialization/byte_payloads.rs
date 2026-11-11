//! Large byte Head/Full denial, retry and last-owner funding observations.
use crate::relational::durable_publication::source_initial_inputs::{
    checkpoint_admission::cold_materialization::{
        byte_fixture::{assert_bytes, ByteHome},
        read_head, update,
    },
};
use worth_execution::ExecutionRequest;
use worth_testkit_relational::{
    aspect_storage::publications::{begin_main_transaction, release_test_commit_snapshot},
    durable_publication::create_entity_outcome,
};
const PAYLOAD: usize = 64 * 1024;

#[test]
fn cold_inline_bytes_head_preserves_payload_before_and_after_an_ordinary_edit() {
    let setup = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&setup);
    for count in [8, PAYLOAD] {
        let home = ByteHome::new(request, count);
        let (runtime, _) = home.reopen(request);
        read_head(request, &runtime, home.entity).unwrap();
        let mut transaction = begin_main_transaction(&runtime);
        let record = transaction.read_entity(request, home.entity).unwrap();
        assert_bytes(record.base().unwrap(), count, 0x51);
        drop((record, transaction));
        let editable = create_entity_outcome(request, &runtime, "editable-after-read");
        let editable_id = editable.semantic_result().created_entities()[0].assigned;
        release_test_commit_snapshot(&runtime, &editable);
        drop(editable);
        update(request, &runtime, editable_id);
        let mut transaction = begin_main_transaction(&runtime);
        let record = transaction.read_entity(request, home.entity).unwrap();
        assert_bytes(record.base().unwrap(), count, 0x51);
    }
}

#[test]
fn cold_inline_bytes_history_preserves_full_adjacency_and_original_payload() {
    let setup = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&setup);
    for count in [8, PAYLOAD] {
        let home = ByteHome::new(request, count);
        let (runtime, _) = home.reopen(request);
        assert!(runtime
            .storage_access()
            .outgoing_relations_for_entity(request, home.entity, home.current_version)
            .unwrap()
            .is_empty());
        let view = runtime.inspect_what_happened().open_historical_view(request, home.old_version,
            worth_relational::facade::inspection::HistoricalInspectionMode::AllowCanonicalReconstruction)
            .unwrap().view.unwrap();
        assert_bytes(view.read_view.get_entity(home.entity).unwrap(), count, 0x17);
    }
}
