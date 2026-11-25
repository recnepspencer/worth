//! Fork copies preserve original Head and edited ring values across standalone export.
use super::{
    current_publications::delete_entity, record_field_values::name,
    region_publications::create_relation_in_partition, region_source::recover_shape,
};
use worth_relational::facade::{identity::*, runtime::ProjectionAspectScope};
use worth_testkit_relational::{
    aspect_storage::publications::{begin_main_transaction, release_test_commit_snapshot},
    cold_recovery::update_entity,
    durable_medium::TestDurableMedium,
    durable_publication::durable_runtime,
};
#[test]
fn head_reads_in_each_fork_preserve_original_name_without_the_missing_history() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &worth_execution::ExecutionRequest::serial(&host);
    for fork_first in [false, true] {
        let cold = recover_shape(request, Some(PartitionId(1)), 3, true);
        let fork = cold.runtime.fork().unwrap();
        let ordered = if fork_first {
            [&fork, &cold.runtime]
        } else {
            [&cold.runtime, &fork]
        };
        let main_branch = worth_relational::facade::history::BranchId("main".into());
        for runtime in ordered {
            assert_eq!(
                runtime
                    .observe_branch_region_residency(&main_branch, cold.entities[0].partition_id)
                    .unwrap()
                    .head,
                worth_relational::facade::durability::RelationalRegionProbeStatus::Cold
            );
            let mut txn = begin_main_transaction(runtime);
            let record = txn
                .read_entity(request, cold.entities[0])
                .unwrap()
                .effective()
                .cloned()
                .expect("a cold Head miss cannot escape as absence");
            assert_eq!(name(&record), Some("source"));
            let probe = runtime
                .observe_branch_region_residency(&main_branch, cold.entities[0].partition_id)
                .unwrap();
            assert_eq!(
                probe.head,
                worth_relational::facade::durability::RelationalRegionProbeStatus::Resident
            );
            assert_eq!(
                probe.full,
                worth_relational::facade::durability::RelationalRegionProbeStatus::Cold
            );
        }
    }
}
#[test]
fn cold_forks_edit_existing_ring_records_in_both_orders_and_checkpoint() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &worth_execution::ExecutionRequest::serial(&host);
    for fork_first in [false, true] {
        let cold = recover_shape(request, None, 4, true);
        let fork = cold.runtime.fork().unwrap();
        let ordered = if fork_first {
            [&fork, &cold.runtime]
        } else {
            [&cold.runtime, &fork]
        };
        for (index, runtime) in ordered.into_iter().enumerate() {
            let label = format!("edited-copy-{index}");
            update_entity(request, runtime, cold.entities[0], &label);
            let deleted = delete_entity(request, runtime, cold.entities[2]);
            release_test_commit_snapshot(runtime, &deleted);
            create_relation_in_partition(
                request,
                runtime,
                cold.entities[0],
                cold.entities[1],
                &format!("new-edge-{index}"),
                PartitionId(4),
            );
            let mut expected = begin_main_transaction(runtime);
            let records = cold
                .entities
                .iter()
                .map(|&id| {
                    expected
                        .read_entity(request, id)
                        .unwrap()
                        .effective()
                        .cloned()
                })
                .collect::<Vec<_>>();
            assert!(records[0].is_some());
            assert_eq!(name(records[0].as_ref().unwrap()), Some(label.as_str()));
            assert!(records[2].is_none());
            let (native, native_source) =
                worth_testkit_relational::cold_recovery::capture_external_carrying(
                    request,
                    runtime,
                    &cold.source,
                );
            let medium = TestDurableMedium::new();
            let mut reopened = durable_runtime(&medium);
            reopened
                .durability_recovery()
                .restore_native_checkpoint(request, &native, native_source)
                .unwrap();
            let mut reader = begin_main_transaction(&reopened);
            for (&id, record) in cold.entities.iter().zip(records) {
                assert_eq!(
                    reader.read_entity(request, id).unwrap().effective(),
                    record.as_ref()
                );
            }
            let (_, basis) = runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap();
            let view = runtime
                .read_truth()
                .project_observation(&basis.observation())
                .unwrap();
            let relations = view
                .relation_records_with_projection_scope(
                    request,
                    KindId(2),
                    ProjectionAspectScope::empty(),
                    |record| {
                        let id = record.relation_id();
                        (id.partition_id == PartitionId(4)).then_some(id)
                    },
                )
                .unwrap();
            assert!(relations.len() >= 2);
            let main_branch = worth_relational::facade::history::BranchId("main".into());
            let slot_census = runtime
                .observe_branch_partition_slot_census(&main_branch, PartitionId(4))
                .expect("edited partition 4 is resident");
            assert_eq!(slot_census.occupied_relation_slots.len(), relations.len());
            for id in relations {
                assert_eq!(
                    reader.read_relation(request, id).unwrap().effective(),
                    expected.read_relation(request, id).unwrap().effective()
                );
            }
        }
    }
}
