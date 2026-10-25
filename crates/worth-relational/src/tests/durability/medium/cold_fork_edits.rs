//! Forked cold records preserve dictionary meaning through edits and recovery.
use super::*;

#[test]
fn cold_forks_edit_existing_ring_records_in_both_orders_and_checkpoint() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for fork_first in [false, true] {
        let cold = recover_shape(execution, None, 4, true);
        let fork = cold.runtime.fork().unwrap();
        let ordered = if fork_first {
            [&fork, &cold.runtime]
        } else {
            [&cold.runtime, &fork]
        };
        for (index, runtime) in ordered.into_iter().enumerate() {
            // Update first, with no preceding warm read or new-record creation.
            let label = format!("edited-copy-{index}");
            update_entity_and_release_snapshot(execution, runtime, cold.entities[0], &label);
            let deleted = delete_entity(execution, runtime, cold.entities[2]);
            release_test_commit_snapshot(runtime, &deleted);
            create_relation_in_partition(
                execution,
                runtime,
                cold.entities[0],
                cold.entities[1],
                &format!("new-edge-{index}"),
                PartitionId(4),
            );
            let mut expected = test_owner_begin_transaction_for_main(runtime);
            let records = cold
                .entities
                .iter()
                .map(|&id| {
                    expected
                        .read_entity(execution, id)
                        .unwrap()
                        .effective()
                        .cloned()
                })
                .collect::<Vec<_>>();
            assert!(records[0].is_some());
            assert_eq!(
                read_entity_name(records[0].as_ref().unwrap()).as_deref(),
                Some(label.as_str())
            );
            assert!(records[2].is_none());
            let native = runtime
                .durability_authority()
                .native_checkpoint(execution.allocation_policy())
                .unwrap();
            let mut reopened = durable_runtime_with_test_schema();
            reopened
                .durability_recovery()
                .restore_native_checkpoint(execution, &native)
                .unwrap();
            let mut reader = test_owner_begin_transaction_for_main(&reopened);
            for (&id, record) in cold.entities.iter().zip(records) {
                assert_eq!(
                    reader.read_entity(execution, id).unwrap().effective(),
                    record.as_ref()
                );
            }
            let root = runtime
                .history
                .branch_cell(&crate::history::data::BranchId("main".into()))
                .unwrap()
                .root()
                .unwrap();
            let relations = root
                .partition_state(PartitionId(4))
                .unwrap()
                .relation_arena
                .occupied_slots()
                .into_iter()
                .filter_map(|slot| {
                    root.partition_state(PartitionId(4))
                        .unwrap()
                        .relation_arena
                        .get_slot(slot)
                        .map(|view| {
                            crate::identity::data::RelationId::new(
                                PartitionId(4),
                                slot as u64,
                                view.generation(),
                            )
                        })
                })
                .collect::<Vec<_>>();
            assert!(relations.len() >= 2);
            for id in relations {
                assert_eq!(
                    reader.read_relation(execution, id).unwrap().effective(),
                    expected.read_relation(execution, id).unwrap().effective()
                );
            }
        }
    }
}
