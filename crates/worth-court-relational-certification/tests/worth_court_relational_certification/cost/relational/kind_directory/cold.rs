//! Fixed selected rows do not read unrelated regions or history members.
use super::super::cold_checkpoint::{reopen, ColdCheckpointSource};
use super::*;
use crate::content::historical_kind::fixture::World;
use std::sync::atomic::Ordering;
use worth_relational::facade::{durability::*, transactions::*};
fn scan(regions: usize, history: usize) -> [u64; 2] {
    let original = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1 << 30)),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&original);
    let mut world = populated(&request, regions, true);
    world.create(&request, 7, "selected-c");
    let target = world.entities.len() - 1;
    world.relate(&request, 0, 1, 7, "selected edge");
    for step in 0..history {
        let committed = world
            .commit(
                &request,
                MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
                    UpdateRelationEndpointsIntent {
                        relation_id: world.relations[0],
                        kind_id: KindId(2),
                        source: EntityReference::Existing(world.entities[0]),
                        target: EntityReference::Existing(
                            world.entities[if step % 2 == 0 { target } else { 1 }],
                        ),
                    },
                )),
            )
            .unwrap();
        world.release(&committed);
    }
    let expected = ids(&pin(&world, &request), &request, KindId(1));
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
    let source = ColdCheckpointSource::capture(&world.runtime, &request, &checkpoint);
    let receiving = reopen(
        &request,
        &checkpoint,
        source.clone(),
        fixture::world().runtime,
    );
    let cold = World {
        runtime: receiving,
        entities: vec![],
        relations: vec![],
    };
    let selected = pin(&cold, &request);
    assert!(source.reads.lock().unwrap().is_empty());
    assert_eq!(ids(&selected, &request, KindId(1)), expected);
    let bytes = source.bytes.load(Ordering::SeqCst);
    // Use a second real pin over the now admitted immutable Head so this work
    // observation isolates query routing from cold boundary authentication.
    let query = query::work(&cold, &request, &original, &expected);
    [bytes, query]
}
#[test]
fn fixed_kind_read_is_flat_in_unrelated_regions_and_history() {
    for (axis, small, large) in [
        ("regions", scan(32, 8), scan(64, 8)),
        ("history", scan(8, 64), scan(8, 128)),
    ] {
        println!("SDK kind cold bytes/query work ({axis}): {small:?}->{large:?}");
        for index in 0..2 {
            assert_change_bounded("kind directory", axis, small[index], large[index]);
        }
    }
}
