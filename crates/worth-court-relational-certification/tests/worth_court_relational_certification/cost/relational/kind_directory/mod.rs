//! Actual SDK scans stay within the requested kind and exact retained root.
mod cold;
mod fixture;
mod query;
use fixture::*;
use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionWorkCeiling, SerialMemoryBudget, SerialRequest,
};
use worth_relational::facade::{identity::KindId, runtime::RelationalPinnedRead};
use worth_testkit_relational::maintenance::assert_change_bounded;

fn ids(
    pin: &RelationalPinnedRead<()>,
    request: &ExecutionRequest<'_, '_>,
    kind: KindId,
) -> Vec<worth_relational::facade::identity::EntityId> {
    pin.project_read()
        .bounded_entities_of_kind(request, kind, 1024)
        .unwrap()
        .unwrap()
        .into_records()
        .into_iter()
        .map(|row| row.entity_id)
        .collect()
}
fn scan_work(records: usize) -> [u64; 2] {
    let original = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1 << 30)),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&original);
    let world = populated(&request, records, false);
    let expected = world.entities[..2].to_vec();
    let pin = pin(&world, &request);
    assert_eq!(ids(&pin, &request, KindId(1)), expected);
    let (actual, report) = ExecutionWorkCeiling::new(None)
        .run_serial(&original, || ids(&pin, &request, KindId(1)))
        .unwrap();
    assert_eq!(actual, expected);
    [
        report.charged_work(),
        query::work(&world, &request, &original, &expected),
    ]
}
#[test]
fn exact_kind_read_is_flat_in_unrelated_records_in_the_same_region() {
    let small = scan_work(128);
    let large = scan_work(256);
    println!("SDK exact kind charged work (bounded/query): {small:?}->{large:?}");
    for (label, axis) in ["bounded kind directory", "packetized kind directory"]
        .into_iter()
        .zip(0..2)
    {
        assert_change_bounded(label, "unrelated records", small[axis], large[axis]);
    }
}
#[test]
fn exact_kind_membership_survives_retirement_replacement_and_retained_roots() {
    let original = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1 << 30)),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&original);
    let mut world = world();
    world.create(&request, 7, "selected");
    let retained = pin(&world, &request);
    let retained_ids = world.entities.clone();
    for step in 0..12 {
        create_other(&mut world, &request, 7 + step % 3, "other");
        world.create(&request, 7 + step % 3, "selected next");
        if step % 3 == 0 {
            world.replace(&request, world.entities.len() - 1, 8, "replacement");
        }
        if step % 3 == 1 {
            world.delete_entity(&request, world.entities.len() - 1);
        }
        if step % 3 == 2 {
            let target = world.entities.len() - 1;
            world.relate(&request, 0, target, 9, "directory edge");
        }
        let selected = pin(&world, &request);
        let (_, basis) = world
            .runtime
            .observe_branch(&world.runtime.main_branch_identity())
            .unwrap();
        let oracle = world
            .runtime
            .read_truth()
            .read_observation(&request, &basis.observation())
            .unwrap()
            .unwrap();
        for kind in [KindId(1), KindId(3)] {
            let expected: Vec<_> = oracle
                .entities()
                .iter()
                .filter(|row| row.kind.kind_id == kind)
                .map(|row| row.entity_id)
                .collect();
            assert_eq!(ids(&selected, &request, kind), expected);
        }
        let expected_relations: Vec<_> = oracle
            .relations()
            .iter()
            .filter(|row| row.kind.kind_id == KindId(2))
            .map(|row| row.relation_id)
            .collect();
        let actual_relations: Vec<_> = selected
            .project_read()
            .bounded_relations_of_kind(&request, KindId(2), 1024)
            .unwrap()
            .unwrap()
            .into_records()
            .into_iter()
            .map(|row| row.relation_id)
            .collect();
        assert_eq!(actual_relations, expected_relations);
        if step % 3 == 2 {
            use worth_relational::facade::durability::*;
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
            let source = super::cold_checkpoint::ColdCheckpointSource::capture(
                &world.runtime,
                &request,
                &checkpoint,
            );
            let reopened = super::cold_checkpoint::reopen(
                &request,
                &checkpoint,
                source,
                fixture::world().runtime,
            );
            let cold = crate::content::historical_kind::fixture::World {
                runtime: reopened,
                entities: vec![],
                relations: vec![],
            };
            let cold_pin = fixture::pin(&cold, &request);
            for kind in [KindId(1), KindId(3)] {
                assert_eq!(
                    ids(&selected, &request, kind),
                    ids(&cold_pin, &request, kind)
                );
            }
            assert_eq!(
                actual_relations,
                cold_pin
                    .project_read()
                    .bounded_relations_of_kind(&request, KindId(2), 1024)
                    .unwrap()
                    .unwrap()
                    .into_records()
                    .into_iter()
                    .map(|row| row.relation_id)
                    .collect::<Vec<_>>()
            );
        }
        assert_eq!(ids(&retained, &request, KindId(1)), retained_ids);
    }
}
