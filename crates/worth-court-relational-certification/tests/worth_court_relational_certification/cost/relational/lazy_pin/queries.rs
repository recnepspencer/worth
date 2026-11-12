//! Cold query execution must demand the regions its actual plan reaches.
use super::*;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::{query::*, transactions::RecordRef};

#[test]
fn cold_explicit_scan_and_traversal_keep_their_selected_rows() {
    let original = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1 << 30)),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&original);
    let mut world = World::new();
    world.create(&request, 7, "selected");
    world.create(&request, 8, "neighbor");
    world.create(&request, 50, "unreached");
    world.relate(&request, 0, 1, 9, "cross-region edge");
    let checkpoint = world
        .runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            &request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    for mode in 0..3 {
        let source = Source::capture(&world.runtime, &request, &checkpoint);
        let runtime = reopen(&request, &checkpoint, source.clone(), World::new().runtime);
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let handle = runtime
            .snapshots()
            .snapshot_for_observation(&basis.observation())
            .unwrap();
        let read = runtime
            .read_truth()
            .pin_read_capability(&handle, &[], &request)
            .unwrap();
        assert_eq!(source.region_read_count(), 0);
        let mut packet = PlannedQueryPacket::explicit_targets(
            "cold selected query",
            read.query_context().clone(),
            vec![RecordRef::Entity(world.entities[0])],
        );
        match mode {
            1 => {
                packet.scope = QueryScope::EntityKindScan {
                    kind_id: KindId(1),
                    partition_scope: Some(Arc::from([PartitionId(7)])),
                };
                packet.ordering = QueryOrderingContract::CanonicalEntityIdOrder;
            }
            2 => {
                packet.scope = QueryScope::OutgoingNeighborhood {
                    seeds: Arc::from([world.entities[0]]),
                    relation_kind_scope: None,
                };
                packet.ordering = QueryOrderingContract::CanonicalTraversalOrder;
            }
            _ => {}
        }
        let plan = runtime
            .read_truth()
            .plan_query_packet(&handle, packet)
            .unwrap();
        let result = read
            .execute_query_plan(plan, &request)
            .unwrap()
            .unwrap()
            .outcome
            .result;
        let mut entities: Vec<_> = result.entities.iter().map(|row| row.entity_id).collect();
        entities.sort();
        let mut expected = if mode == 2 {
            world.entities[..2].to_vec()
        } else {
            vec![world.entities[0]]
        };
        expected.sort();
        assert_eq!(entities, expected, "query mode {mode}");
        assert_eq!(
            result
                .relations
                .iter()
                .map(|row| row.relation_id)
                .collect::<Vec<_>>(),
            if mode == 2 {
                world.relations.clone()
            } else {
                vec![]
            }
        );
    }
}
