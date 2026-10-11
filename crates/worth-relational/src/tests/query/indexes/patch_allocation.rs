use super::*;
use crate::facade::indexes::DerivedIndexMaintenanceBudget as Budget;
use crate::facade::transactions::{
    CreateIntent, DeleteEntityIntent, EntityMutationIntent, MutationIntent, WorkerIntentBatch,
};
use std::num::NonZeroUsize;
use worth_execution::{CancellationToken, ExecutionRequest, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[test]
fn a_two_row_commit_prices_its_paths_in_a_two_thousand_row_index() {
    const ROWS: usize = 2_000;
    const WIDTH: usize = 3_072;
    const MEMORY: u64 = 1_048_576;
    assert!(MEMORY < (ROWS * WIDTH / 4) as u64);
    let host = crate::test_execution_policy::serial_request();
    let setup = ExecutionRequest::serial(&host);
    let runtime = runtime_with_index_field_aspects();
    let mut batch = WorkerIntentBatch::new("patch-memory-population");
    for row in 0..ROWS {
        let id = format!("row{row:04}");
        let name = format!("{id}{}", "x".repeat(WIDTH - id.len()));
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(
            crate::transactions::data::EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: crate::symbols::data::ClientKey::raw(&id),
                fields: name_field_patch(&name),
            },
        )));
    }
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(batch, setup.allocation_policy())
        .unwrap();
    let first = transaction.commit(&setup, &runtime).unwrap();
    let entities = changed_entities(&first);
    assert_eq!(entities.len(), ROWS);
    let index = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "patch-memory.name".into(),
            branch_scoped: true,
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
        })
        .index_id;
    let built = runtime.index_authority().build_for_commit(
        &setup,
        DerivedIndexBuildRequest {
            source_commit_id: first.commit.commit_id,
            branch_id: BranchId("main".into()),
            index_ids: vec![index],
        },
    );
    assert!(built.failed_indexes.is_empty(), "{built:?}");
    // Two 2-level map copies plus two singleton row edits, key encoding,
    // pending queues, locators and publication buffers reserve 112,204 bytes.
    // Former 112,204 less inline row nodes (17,152), absent key Arcs (80),
    // and copied-path pricing for mutable scratch trees (2,984).
    const PATCH_BYTES: u64 = 91_988;
    // The singleton scan also holds 320 framework bytes on this lease.
    const FRAMEWORK_BYTES: u64 = 320;
    for memory in [PATCH_BYTES + FRAMEWORK_BYTES - 1, MEMORY] {
        let lease = test_execution_authority()
            .request_lease(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    ExecutionPosture::Automatic,
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(NonZeroUsize::MIN, memory, 1_000_000),
                ),
                deadline: None,
                cancellation: CancellationToken::new(),
            })
            .unwrap();
        let execution = ExecutionRequest::leased(&lease);
        let mut transaction = test_owner_begin_transaction_for_main(&runtime);
        let mut edits = WorkerIntentBatch::new("two-row-patch");
        for entity_id in entities.iter().copied().take(2) {
            edits = edits.push(MutationIntent::Entity(EntityMutationIntent::Delete(
                DeleteEntityIntent { entity_id },
            )));
        }
        transaction
            .push_batch(edits, setup.allocation_policy())
            .unwrap();
        let mut candidate = runtime
            .prepare_branch_transaction(&setup, transaction)
            .unwrap();
        let work = runtime.index_authority().prepare_for_candidate(
            &mut candidate,
            &[index],
            Some(&first.snapshot),
            Budget {
                maximum_work_units: 100_000,
                maximum_cold_record_slots: 0,
                maximum_derived_rows: 100_000,
            },
            &execution,
        );
        if memory == PATCH_BYTES + FRAMEWORK_BYTES - 1 {
            let denied = work.unwrap_err();
            assert_eq!(
                denied.kind,
                crate::indexes::data::DerivedIndexMaintenanceDenialKind::Execution(
                    crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted {
                        requested: 91_988,
                        admitted: 91_987,
                        ancestor: 0,
                    }
                )
            );
            assert_eq!(
                runtime
                    .index_access()
                    .latest_generation(index, &BranchId("main".into()))
                    .unwrap()
                    .source_commit_id,
                first.commit.commit_id
            );
            drop(candidate);
            continue;
        }
        let work = work.expect("two copied paths fit the caller's memory");
        assert_eq!(crate::indexes::last_allocation_bytes(), PATCH_BYTES);
        assert_eq!(work.patch_records, 2);
        assert_eq!(work.entry_edits, 2);
        assert_eq!(work.cold_record_slots, 0);
        let crate::mvcc::RelationalPublicationOutcome::Performed(performed) =
            runtime.publication_port().compare_and_publish(candidate)
        else {
            panic!("prepared patch must publish")
        };
        let second = runtime.settle_performed_publication(performed).unwrap();
        let generation = runtime
            .index_access()
            .latest_generation(index, &BranchId("main".into()))
            .unwrap();
        let crate::facade::indexes::DerivedIndexEntries::EntityField(entries) = &generation.entries
        else {
            panic!("field entries")
        };
        assert_eq!(entries.len(), ROWS - 2);
        release_test_commit_snapshot(&runtime, &second);
    }
    release_test_commit_snapshot(&runtime, &first);
}
