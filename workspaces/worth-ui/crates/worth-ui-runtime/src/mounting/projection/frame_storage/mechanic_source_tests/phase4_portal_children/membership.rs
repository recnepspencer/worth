use super::*;
use crate::mounting::projection::lowering::portal_changes::children_in_transition;
use crate::runtime::persistent_index::{begin_all_test_observation, test_work};

#[test]
fn portal_children_select_only_the_exact_surface_neighborhood_at_scale() {
    for size in [64, 4096] {
        let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let other = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let owner = UiMountedInstanceIdentity::mint_unbound().unwrap();
        let other_owner = UiMountedInstanceIdentity::mint_unbound().unwrap();
        let mut records = vec![
            relation_node(owner, 0, surface, Some("owner.a"), None),
            relation_node(other_owner, 1, other, Some("owner.a"), None),
        ];
        let mut local = Vec::new();
        let mut foreign = Vec::new();
        for index in 2..size {
            let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
            let (node_surface, relation) = match index {
                2 | 3 => {
                    local.push(instance);
                    (surface, Some("owner.a"))
                }
                4 | 5 => {
                    foreign.push(instance);
                    (other, Some("owner.a"))
                }
                _ => (surface, Some("unrelated.owner")),
            };
            records.push(relation_node(instance, index, node_surface, None, relation));
        }
        let semantic = UiMountedSemanticProjection::initial(records, vec![]);
        begin_all_test_observation();
        let (actual, charged) = semantic.portal_children_for_owners(&[owner]);
        let observed = test_work();
        local.sort_unstable();
        assert_eq!(actual, local);
        assert_eq!(
            observed.iterated_entries(),
            2,
            "lookup may visit only the selected child bucket"
        );
        assert_eq!(
            charged,
            observed.lookup_probes() + observed.iterated_entries()
        );
        assert!(
            charged < 32,
            "work follows tree height and selected children: {charged}"
        );
        foreign.sort_unstable();
        assert_eq!(
            semantic.portal_children_for_owners(&[other_owner]).0,
            foreign
        );
        assert_eq!(
            semantic.portal_children_for_owners(&[owner, owner]).0,
            local
        );
        let mut candidate = semantic.clone();
        let instance = candidate
            .nodes_in_order()
            .last()
            .unwrap()
            .receipt
            .mounted_instance();
        let mut changed_node = candidate.node(instance).unwrap().clone();
        changed_node.portal_child_owner =
            Some(crate::capability::ComponentId::new("owner.a").unwrap());
        begin_all_test_observation();
        let mutation = candidate.insert_node(changed_node);
        assert_eq!(
            test_work().iterated_entries(),
            0,
            "a membership edit cannot rebuild a bucket"
        );
        assert!(mutation.key_probes() + mutation.node_copies() < 200);
        local.push(instance);
        local.sort_unstable();
        assert_eq!(candidate.portal_children_for_owners(&[owner]).0, local);
        assert_eq!(semantic.portal_children_for_owners(&[owner]).0.len(), 2);
    }
}

#[test]
fn portal_membership_tracks_reparent_surface_move_and_removal_without_mutating_predecessor() {
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let other = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let owner_a = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let owner_b = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let other_b = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let child = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let mut semantic = UiMountedSemanticProjection::initial(
        vec![
            relation_node(owner_a, 0, surface, Some("owner.a"), None),
            relation_node(owner_b, 1, surface, Some("owner.b"), None),
            relation_node(other_b, 2, other, Some("owner.b"), None),
            relation_node(child, 3, surface, None, None),
        ],
        vec![],
    );
    let empty_membership_bytes = semantic.retained_structural_bytes().unwrap();
    semantic.insert_node(relation_node(child, 3, surface, None, Some("owner.a")));
    assert!(semantic.retained_structural_bytes().unwrap() > empty_membership_bytes);
    let original = semantic.clone();
    let bytes = semantic.retained_structural_bytes();
    let no_op = semantic.insert_node(semantic.node(child).unwrap().clone());
    assert_eq!(bytes, semantic.retained_structural_bytes());
    assert!(no_op.node_copies() < 8);
    semantic.insert_node(relation_node(child, 3, surface, None, Some("owner.b")));
    assert!(semantic.portal_children_for_owners(&[owner_a]).0.is_empty());
    assert_eq!(semantic.portal_children_for_owners(&[owner_b]).0, [child]);
    semantic.insert_node(relation_node(child, 3, other, None, Some("owner.b")));
    assert!(semantic.portal_children_for_owners(&[owner_b]).0.is_empty());
    assert_eq!(semantic.portal_children_for_owners(&[other_b]).0, [child]);
    assert_eq!(original.portal_children_for_owners(&[owner_a]).0, [child]);
    assert!(original.portal_children_for_owners(&[other_b]).0.is_empty());
    semantic.insert_node(relation_node(child, 3, surface, None, None));
    assert_eq!(
        semantic.retained_structural_bytes().unwrap(),
        empty_membership_bytes
    );
    semantic.insert_node(relation_node(child, 3, surface, None, Some("owner.a")));
    let before_remove = semantic.clone();
    semantic.remove_node(child);
    assert!(semantic.portal_children_for_owners(&[owner_a]).0.is_empty());
    assert_eq!(
        before_remove.portal_children_for_owners(&[owner_a]).0,
        [child]
    );
}

#[test]
fn portal_transition_selects_retired_owner_children_and_new_surface_children() {
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let other = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let owner = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let old_child = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let new_child = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let previous = UiMountedSemanticProjection::initial(
        vec![
            relation_node(owner, 0, surface, Some("owner.a"), None),
            relation_node(old_child, 1, surface, None, Some("owner.a")),
            relation_node(new_child, 2, other, None, Some("owner.a")),
        ],
        vec![],
    );
    let mut successor = previous.clone();
    successor.remove_node(owner);
    assert_eq!(
        children_in_transition(Some(&previous), &successor, &[owner]).0,
        [old_child]
    );
    successor.insert_node(relation_node(owner, 0, other, Some("owner.a"), None));
    let mut expected = vec![old_child, new_child];
    expected.sort_unstable();
    assert_eq!(
        children_in_transition(Some(&previous), &successor, &[owner]).0,
        expected
    );
    assert_eq!(
        children_in_transition(None, &successor, &[owner]).0,
        [new_child]
    );
}

#[test]
fn portal_owner_delta_keeps_unchanged_participants_out_and_preserves_order_changes() {
    use crate::mounting::projection::lowering::portal_changes::changed_owners;
    let owner = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let child = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let first = portal_overlay(frame, owner, surface, binding);
    let second = portal_overlay_for_graph(frame, child, surface, binding, 4_152);
    let (fonts, _) = worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
    let previous = projection_frame_with_identity(
        frame,
        portal_semantic_projection(owner, child, surface, binding),
        surface,
        binding,
        owner,
        child,
        Arc::new(fonts),
        Default::default(),
        vec![first, second],
        1,
    );
    assert!(changed_owners(Some(&previous), &[first, second]).is_empty());
    let replacement = portal_overlay(
        UiMountedFrameIdentity::mint_unbound().unwrap(),
        owner,
        surface,
        binding,
    );
    assert_eq!(
        changed_owners(Some(&previous), &[replacement, second]),
        [owner]
    );
    assert_eq!(changed_owners(Some(&previous), &[first]), [child]);
    let mut both = vec![owner, child];
    both.sort_unstable();
    assert_eq!(changed_owners(Some(&previous), &[second, first]), both);
    assert_eq!(changed_owners(Some(&previous), &[]), both);
    assert_eq!(changed_owners(None, &[first, second]), both);
}

fn relation_node(
    instance: UiMountedInstanceIdentity,
    index: usize,
    surface: UiSemanticSurfaceIdentity,
    component: Option<&str>,
    child_owner: Option<&str>,
) -> UiMountedProjectionNodeRecord {
    node(
        instance,
        4_151 + index as u64,
        surface,
        bounds([8.0, 12.0, 20.0, 20.0]),
        component.map(|value| crate::capability::ComponentId::new(value).unwrap()),
        child_owner.map(|value| crate::capability::ComponentId::new(value).unwrap()),
        false,
    )
}
