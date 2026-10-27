//! Real public coupling edits and anchor retirement versus independent UF truth.
use super::*;

#[test]
fn seeded_island_edits_match_union_find_membership_identity_and_lineage() {
    let _guard = checkpoint_recovery_test_guard();
    for mut seed in [7_u64, 19, 73] {
        let mut world = world::World::new(5, &[]);
        let mut live_keys = keys(5);
        let mut vertices = world.members(&live_keys);
        let mut edges = union_find::successors(5);
        let mut model = identity_model::Model::new(
            &world,
            &live_keys,
            &union_find::partitions(&vertices, &edges),
        );
        let untouched = model.name_of(&vertices[&key(4, "a")]);
        let pairs = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];
        let mut coupling = BTreeSet::new();
        for _ in 0..24 {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            let (from, to) = pairs[((seed >> 32) as usize) % pairs.len()];
            let edge = (key(from, "a"), key(to, "a"));
            let linked = !coupling.remove(&edge);
            if linked {
                assert!(coupling.insert(edge.clone()));
            }
            world.couple(edge.0, edge.1, linked);
            let authored = edges.union(&coupling).cloned().collect();
            let expected = union_find::partitions(&vertices, &authored);
            model.verify(&world, &live_keys, expected);
            assert_eq!(model.name_of(&vertices[&key(4, "a")]), untouched);
        }
        let retired_key = key(0, "a");
        let retired = vertices[&retired_key].clone();
        let prior = model.name_of(&retired);
        assert_eq!(
            vertices.values().min(),
            Some(&retired),
            "remove the actual least issued Body generation"
        );
        world.replace_anchor();
        live_keys.retain(|key| key != &retired_key);
        live_keys.push(key(0, "replacement"));
        vertices = world.members(&live_keys);
        assert!(
            !vertices.values().any(|member| member == &retired),
            "replacement cannot alias the retired generation"
        );
        edges.retain(|(from, to)| from != &retired_key && to != &retired_key);
        coupling.retain(|(from, to)| from != &retired_key && to != &retired_key);
        edges.insert((key(0, "c"), key(0, "replacement")));
        edges.insert((key(0, "replacement"), key(0, "b")));
        let authored = edges.union(&coupling).cloned().collect();
        model.verify(
            &world,
            &live_keys,
            union_find::partitions(&vertices, &authored),
        );
        assert_eq!(
            model.name_of(&vertices[&key(0, "b")]),
            prior,
            "anchor removal keeps the old surviving heir name"
        );
        assert_eq!(model.name_of(&vertices[&key(4, "a")]), untouched);
    }
}
