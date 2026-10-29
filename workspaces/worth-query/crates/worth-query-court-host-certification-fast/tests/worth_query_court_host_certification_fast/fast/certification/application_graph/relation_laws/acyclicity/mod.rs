pub(crate) use worth_query_worlds::facade::certification::relation_laws::acyclicity::*;

#[test]
fn cycle_closed_across_commits_refuses_original_edge_and_keeps_committed_truth() {
    let world = install::<Acyclic>(3, &[]);
    committed(world.edit(1, &[(0, 1)]));
    let before = read::committed(&world);
    let refusal = denied(world.edit(2, &[(1, 0)]));
    closing_edge(&refusal, &before, &[(1, 0)]);
    assert_eq!(read::committed(&world).edges, before.edges);
    committed(world.edit(3, &[(1, 2)]));
    assert_eq!(
        read::committed(&world).edges,
        BTreeSet::from([(0, 1), (1, 2)])
    );
    world.close();
}

#[test]
fn two_new_edges_are_judged_as_one_proposed_graph() {
    let world = install::<Acyclic>(3, &[]);
    let before = read::committed(&world);
    let refusal = denied(world.edit(1, &[(0, 1), (1, 0)]));
    closing_edge(&refusal, &before, &[(0, 1), (1, 0)]);
    assert!(read::committed(&world).edges.is_empty());
    committed(world.edit(2, &[(0, 1), (1, 2)]));
    assert_eq!(
        read::committed(&world).edges,
        BTreeSet::from([(0, 1), (1, 2)])
    );
    world.close();
}

#[test]
fn self_endpoint_reaches_acyclicity_and_unrestricted_twin_can_publish_it() {
    let world = install::<Acyclic>(2, &[]);
    let before = read::committed(&world);
    closing_edge(&denied(world.edit(1, &[(0, 0)])), &before, &[(0, 0)]);
    committed(world.edit(2, &[(0, 1)]));
    world.close();
    let twin = install::<Unrestricted>(2, &[]);
    committed(twin.edit(1, &[(0, 0)]));
    assert_eq!(read::committed(&twin).edges, BTreeSet::from([(0, 0)]));
    twin.close();
}

#[test]
fn a_thousand_edge_history_has_no_custom_rule_depth_limit() {
    let edges: Vec<_> = (0..1000).map(|node| (node, node + 1)).collect();
    let world = install::<Acyclic>(1002, &edges);
    let before = read::committed(&world);
    closing_edge(&denied(world.edit(1, &[(1000, 0)])), &before, &[(1000, 0)]);
    assert_eq!(read::committed(&world).edges, before.edges);
    committed(world.edit(2, &[(1000, 1001)]));
    let mut expected = before.edges;
    expected.insert((1000, 1001));
    assert_eq!(read::committed(&world).edges, expected);
    world.close();
}

#[test]
fn seeded_edge_batches_match_an_independent_reachability_model() {
    let world = install::<Acyclic>(6, &[]);
    let mut edges = BTreeSet::new();
    let mut state = 0x9181_u64;
    for serial in 1..=48 {
        let mut additions = Vec::new();
        for _ in 0..(1 + serial % 2) {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let source = (state >> 32) % 6;
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            additions.push((source, (state >> 32) % 6));
        }
        let mut proposed = edges.clone();
        proposed.extend(additions.iter().copied());
        let outcome = world.edit(serial, &additions);
        if has_cycle(&proposed) {
            closing_edge(&denied(outcome), &read::committed(&world), &additions);
        } else {
            committed(outcome);
            edges = proposed;
        }
        assert_eq!(read::committed(&world).edges, edges);
    }
    world.close();
}
