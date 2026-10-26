//! Retained reconstruction scales with its rows and refuses without partial publication.
use super::*;
use crate::facade::indexes::DerivedIndexMaintenanceBudget;
use crate::facade::transactions::{CreateIntent, MutationIntent, WorkerIntentBatch};
use crate::indexes::data::DerivedIndexMaintenanceDenialKind as Denial;
use worth_execution::ExecutionRequest;
const MEMORY: u64 = 64 * 1024 * 1024;
fn reconstruct(n: usize, memory: u64, succeeds: bool) -> u64 {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_index_field_aspects();
    let mut committed = None;
    // Populate through ordinary commits within the native 4,096-record limit.
    for start in (0..n).step_by(4096) {
        let mut batch = WorkerIntentBatch::new("cold-memory-source");
        for row in start..n.min(start + 4096) {
            let name = format!("row{row:05}");
            batch = batch.push(MutationIntent::Create(CreateIntent::Entity(
                crate::transactions::data::EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw(&name),
                    fields: name_field_patch(&name),
                },
            )));
        }
        let mut transaction = test_owner_begin_transaction_for_main(&runtime);
        transaction
            .push_batch(batch, execution.allocation_policy())
            .unwrap();
        let next = transaction.commit(&execution, &runtime).unwrap();
        if let Some(old) = committed.take() {
            release_test_commit_snapshot(&runtime, &old);
        }
        committed = Some(next);
    }
    let committed = committed.unwrap();
    let index = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "cold-memory.name".into(),
            branch_scoped: true,
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
        })
        .index_id;
    // A newer head makes the source an actual retained target, still held by
    // its exact snapshot. Reconstruction never treats the active head as retained.
    let advanced = test_owner_begin_transaction_for_main(&runtime)
        .commit(&execution, &runtime)
        .unwrap();
    let reconstruction_host = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(Some(memory)),
        worth_execution::CancellationToken::new(),
        None,
    );
    let result = runtime.index_authority().reconstruct_for_commit(
        &ExecutionRequest::serial(&reconstruction_host),
        DerivedIndexBuildRequest {
            source_commit_id: committed.commit.commit_id,
            branch_id: BranchId("main".into()),
            index_ids: vec![index],
        },
        DerivedIndexMaintenanceBudget {
            maximum_work_units: Some(100_000_000),
            maximum_cold_record_slots: Some(1_000_000),
            maximum_derived_rows: Some(1_000_000),
        },
    );
    let charged = crate::indexes::last_allocation_bytes();
    if succeeds {
        let result = result.unwrap();
        let crate::facade::indexes::DerivedIndexEntries::EntityField(entries) =
            &result.generations[0].entries
        else {
            panic!("field index")
        };
        assert_eq!(entries.len(), n);
    } else {
        assert!(matches!(
            result.unwrap_err().kind,
            Denial::Execution(
                crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted { .. }
            )
        ));
        assert!(runtime
            .index_access()
            .latest_generation(index, &BranchId("main".into()))
            .is_none());
    }
    release_test_commit_snapshot(&runtime, &advanced);
    release_test_commit_snapshot(&runtime, &committed);
    charged
}
#[test]
fn retained_reconstruction_growth_and_memory_refusal_preserve_publication() {
    let small = reconstruct(16, MEMORY, true);
    let large = reconstruct(256, MEMORY, true);
    assert!(
        large <= small * 32,
        "sixteen times the rows must not cause quadratic allocation"
    );
    reconstruct(256, 1024, false);
}
