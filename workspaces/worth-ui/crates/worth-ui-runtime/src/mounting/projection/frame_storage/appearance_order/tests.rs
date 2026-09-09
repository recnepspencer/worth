use super::row::PartitionKey;
use super::*;

fn row(rank: u32) -> Row {
    Row {
        key: PartitionKey {
            binding: worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap(),
            space: worth_ui_host_contract::UiMountedCoordinateSpace::Viewport as u8,
            portal: None,
            rank,
        },
        family: 0,
        bounds: [0.0, 0.0, 20.0, 20.0],
    }
}

fn update(surface: UiSemanticSurfaceIdentity, row: Row) -> Update {
    Update {
        surface,
        instance: UiMountedInstanceIdentity::mint_unbound().unwrap(),
        rows: vec![row],
    }
}

fn apply(
    index: &mut UiMountedAppearanceOrderIndex,
    updates: &[Update],
) -> Result<(), UiMountedAppearanceOrderDenial> {
    index.apply(updates, &mut UiMountedSpatialWork::default())
}

#[test]
fn retained_neighbors_conflict_and_failed_batch_preserves_both_predecessors() {
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let first = update(surface, row(65_536));
    let mut second = update(surface, first.rows[0]);
    second.rows[0].key.rank = u32::MAX;
    let mut index = UiMountedAppearanceOrderIndex::default();
    apply(&mut index, &[first.clone(), second.clone()]).unwrap();
    let mut collision = second.clone();
    collision.rows[0].key.rank = 65_536;
    assert!(
        matches!(apply(&mut index, &[collision]), Err(UiMountedAppearanceOrderDenial::Ambiguous { first: a, second: b, .. })
        if a == first.instance && b == second.instance)
    );
    // If failure leaked a removal, one of these new overlapping peers would pass.
    for retained in [&first, &second] {
        let third = update(surface, retained.rows[0]);
        assert!(matches!(
            apply(&mut index, &[third]),
            Err(UiMountedAppearanceOrderDenial::Ambiguous { .. })
        ));
    }
    let mut swapped_first = first.clone();
    let mut swapped_second = second.clone();
    swapped_first.rows[0].key.rank = u32::MAX;
    swapped_second.rows[0].key.rank = 65_536;
    apply(&mut index, &[swapped_second, swapped_first]).unwrap();
    let mut remove = first.clone();
    remove.rows.clear();
    apply(&mut index, &[remove]).unwrap();
    apply(&mut index, &[update(surface, second.rows[0])]).unwrap();
}

#[test]
fn own_outline_is_exempt_but_cross_node_outline_and_surface_conflict() {
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let mut first = update(surface, row(7));
    let mut outline = first.rows[0];
    outline.family = 1;
    outline.bounds = [-5.0, -5.0, 25.0, 25.0];
    first.rows.push(outline);
    let mut index = UiMountedAppearanceOrderIndex::default();
    apply(&mut index, &[first.clone()]).unwrap();
    let mut neighbor = update(surface, first.rows[0]);
    neighbor.rows[0].bounds = [20.0, 0.0, 24.0, 20.0];
    assert!(matches!(
        apply(&mut index, &[neighbor.clone()]),
        Err(UiMountedAppearanceOrderDenial::Ambiguous { .. })
    ));
    neighbor.rows[0].bounds = [25.0, 0.0, 30.0, 20.0];
    apply(&mut index, &[neighbor]).unwrap();
    // Suppression removes both family entries, so this returning peer is lawful.
    let mut suppressed = first.clone();
    suppressed.rows.clear();
    apply(&mut index, &[suppressed]).unwrap();
    apply(&mut index, &[update(surface, outline)]).unwrap();
}

#[test]
fn surface_binding_coordinate_and_portal_partitions_are_independent() {
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let first = update(surface, row(0));
    let mut index = UiMountedAppearanceOrderIndex::default();
    apply(&mut index, &[first.clone()]).unwrap();
    let mut peers = Vec::new();
    let mut other = update(
        UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
        first.rows[0],
    );
    peers.push(other);
    other = update(surface, first.rows[0]);
    other.rows[0].key.binding =
        worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap();
    peers.push(other);
    other = update(surface, first.rows[0]);
    other.rows[0].key.space = worth_ui_host_contract::UiMountedCoordinateSpace::Window as u8;
    peers.push(other);
    other = update(surface, first.rows[0]);
    other.rows[0].key.portal = Some(UiMountedInstanceIdentity::mint_unbound().unwrap());
    peers.push(other);
    apply(&mut index, &peers).unwrap();
    let mut moved = peers.last().unwrap().clone();
    moved.rows[0].key.portal = None;
    assert!(matches!(
        apply(&mut index, &[moved]),
        Err(UiMountedAppearanceOrderDenial::Ambiguous { .. })
    ));
    index.forget_surface(surface);
    apply(&mut index, &[first]).unwrap();
    assert!(matches!(
        apply(&mut index, &[update(peers[0].surface, peers[0].rows[0])]),
        Err(UiMountedAppearanceOrderDenial::Ambiguous { .. })
    ));
}

#[test]
fn local_order_edit_queries_only_its_spatial_neighborhood() {
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let basis = row(0);
    let mut index = UiMountedAppearanceOrderIndex::default();
    let mut updates = Vec::new();
    for i in 0..4_096 {
        let mut node = update(surface, basis);
        let x = f64::from(i / 64) * 32.0;
        let y = f64::from(i % 64) * 32.0;
        node.rows[0].bounds = [x, y, x + 20.0, y + 20.0];
        updates.push(node);
    }
    apply(&mut index, &updates).unwrap();
    let retained_bytes = index.retained_bytes();
    assert!(retained_bytes > 0);
    let mut unchanged_work = UiMountedSpatialWork::default();
    index
        .apply(&[updates[2_048].clone()], &mut unchanged_work)
        .unwrap();
    assert_eq!(unchanged_work.node_visits, 0);
    assert_eq!(unchanged_work.node_copies, 0);
    assert_eq!(unchanged_work.map_node_copies, 0);
    assert_eq!(index.retained_bytes(), retained_bytes);
    let mut overflow = update(surface, basis);
    overflow.rows[0].bounds = [4_096.0, 0.0, 4_116.0, 20.0];
    assert_eq!(
        apply(&mut index, &[overflow]),
        Err(UiMountedAppearanceOrderDenial::CapacityExceeded)
    );
    assert_eq!(index.retained_bytes(), retained_bytes);
    assert_eq!(index.node_count, updates.len());
    let mut changed = updates[2_048].clone();
    changed.rows[0].bounds[2] += 1.0;
    let mut work = UiMountedSpatialWork::default();
    index.apply(&[changed], &mut work).unwrap();
    assert!(work.region_tests < 32, "{work:?}");
    assert!(work.node_visits < 128, "{work:?}");
    assert!(work.map_key_probes < 128, "{work:?}");
    index.forget_surface(surface);
    assert_eq!(index.node_count, 0);
    assert_eq!(
        index.retained_bytes(),
        UiMountedAppearanceOrderIndex::default().retained_bytes()
    );
}

#[test]
fn surface_and_mechanic_reservations_preserve_the_committed_index() {
    let mut index = UiMountedAppearanceOrderIndex::default();
    let basis = row(0);
    let updates = (0..64)
        .map(|_| update(UiSemanticSurfaceIdentity::mint_unbound().unwrap(), basis))
        .collect::<Vec<_>>();
    apply(&mut index, &updates).unwrap();
    let bytes = index.retained_bytes();
    let extra = update(UiSemanticSurfaceIdentity::mint_unbound().unwrap(), basis);
    assert_eq!(
        apply(&mut index, &[extra]),
        Err(UiMountedAppearanceOrderDenial::SurfaceCapacityExceeded)
    );
    let mut excessive = updates[0].clone();
    excessive.rows = vec![basis; 3];
    assert_eq!(
        apply(&mut index, &[excessive]),
        Err(UiMountedAppearanceOrderDenial::MechanicCapacityExceeded)
    );
    assert_eq!(index.retained_bytes(), bytes);
    assert_eq!(index.node_count, 64);
    assert!(matches!(
        apply(&mut index, &[update(updates[0].surface, basis)]),
        Err(UiMountedAppearanceOrderDenial::Ambiguous { .. })
    ));
}
