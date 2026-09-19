use std::collections::{BTreeMap, BTreeSet};

use super::{
    UiOverlayAnchor, UiOverlayParticipantSurface, UiOverlayRelation,
    UiOverlayRelationAdmissionDenial,
};

pub(super) fn different_surfaces(
    before: UiOverlayAnchor,
    after: UiOverlayAnchor,
    surfaces: &[UiOverlayParticipantSurface],
) -> bool {
    let before_surface = surface_for(before, surfaces);
    let after_surface = surface_for(after, surfaces);
    before_surface
        .zip(after_surface)
        .is_some_and(|(left, right)| left != right)
}

pub(super) fn backdrops_from_relations(
    relations: &[UiOverlayRelation],
    surfaces: &[UiOverlayParticipantSurface],
) -> Vec<(
    UiOverlayAnchor,
    Option<super::super::UiSemanticSurfaceDeclarationIdentity>,
)> {
    let mut anchors = BTreeSet::new();
    for relation in relations {
        for anchor in [relation.lower, relation.upper] {
            if matches!(anchor, UiOverlayAnchor::Backdrop(_)) {
                anchors.insert(anchor);
            }
        }
    }
    anchors
        .into_iter()
        .map(|anchor| (anchor, surface_for(anchor, surfaces)))
        .collect()
}

pub(super) fn ensure_unambiguous(
    backdrops: &[(
        UiOverlayAnchor,
        Option<super::super::UiSemanticSurfaceDeclarationIdentity>,
    )],
    edges: &BTreeMap<UiOverlayAnchor, BTreeSet<UiOverlayAnchor>>,
    relations: &[UiOverlayRelation],
) -> Result<(), UiOverlayRelationAdmissionDenial> {
    let portal_chains = portal_anchored_chains(relations);
    for (index, (left, left_surface)) in backdrops.iter().enumerate() {
        for (right, right_surface) in backdrops.iter().skip(index + 1) {
            if left_surface != right_surface {
                continue;
            }
            if !reachable(*left, *right, edges) && !reachable(*right, *left, edges) {
                // Distinct Portal groups acquire their order from the live,
                // owner-issued stack. Declaration identity supplies no order.
                let distinct_portal_groups = portal_chains
                    .get(left)
                    .zip(portal_chains.get(right))
                    .is_some_and(|(left, right)| left != right);
                if !distinct_portal_groups {
                    return Err(UiOverlayRelationAdmissionDenial::AmbiguousOrder);
                }
            }
        }
    }
    Ok(())
}

fn portal_anchored_chains(
    relations: &[UiOverlayRelation],
) -> BTreeMap<UiOverlayAnchor, UiOverlayAnchor> {
    let mut adjacent = BTreeMap::<_, BTreeSet<_>>::new();
    let mut portals = BTreeSet::new();
    for relation in relations
        .iter()
        .filter(|relation| relation.kind == super::UiOverlayRelationKind::ImmediatelyPrecedes)
    {
        adjacent
            .entry(relation.lower)
            .or_default()
            .insert(relation.upper);
        adjacent
            .entry(relation.upper)
            .or_default()
            .insert(relation.lower);
        for anchor in [relation.lower, relation.upper] {
            if matches!(anchor, UiOverlayAnchor::Portal(_)) {
                portals.insert(anchor);
            }
        }
    }
    let mut owners = BTreeMap::new();
    let mut visited = BTreeSet::new();
    for portal in portals {
        let mut pending = vec![portal];
        let mut component = Vec::new();
        while let Some(anchor) = pending.pop() {
            if !visited.insert(anchor) {
                continue;
            }
            component.push(anchor);
            if let Some(neighbors) = adjacent.get(&anchor) {
                pending.extend(neighbors.iter().copied());
            }
        }
        if component
            .iter()
            .filter(|anchor| matches!(anchor, UiOverlayAnchor::Portal(_)))
            .count()
            == 1
        {
            owners.extend(component.into_iter().map(|anchor| (anchor, portal)));
        }
    }
    owners
}

pub(super) fn ensure_acyclic(
    nodes: &BTreeSet<UiOverlayAnchor>,
    edges: &BTreeMap<UiOverlayAnchor, BTreeSet<UiOverlayAnchor>>,
) -> Result<(), UiOverlayRelationAdmissionDenial> {
    let mut incoming = nodes
        .iter()
        .map(|node| (*node, 0_usize))
        .collect::<BTreeMap<_, _>>();
    for successors in edges.values() {
        for successor in successors {
            *incoming
                .get_mut(successor)
                .ok_or(UiOverlayRelationAdmissionDenial::MissingAnchor)? += 1;
        }
    }
    let mut removed = BTreeSet::new();
    while removed.len() != nodes.len() {
        let available = incoming
            .iter()
            .filter_map(|(node, count)| (*count == 0 && !removed.contains(node)).then_some(*node))
            .collect::<Vec<_>>();
        if available.is_empty() {
            return Err(UiOverlayRelationAdmissionDenial::Cycle);
        }
        for selected in available {
            removed.insert(selected);
            if let Some(successors) = edges.get(&selected) {
                for successor in successors {
                    *incoming.get_mut(successor).expect("validated successor") -= 1;
                }
            }
        }
    }
    Ok(())
}

fn surface_for(
    anchor: UiOverlayAnchor,
    surfaces: &[UiOverlayParticipantSurface],
) -> Option<super::super::UiSemanticSurfaceDeclarationIdentity> {
    surfaces
        .iter()
        .find(|participant| participant.anchor == anchor)
        .map(|participant| participant.surface)
        .or(match anchor {
            UiOverlayAnchor::SurfaceContentOn(surface) => Some(surface),
            UiOverlayAnchor::SurfaceContent
            | UiOverlayAnchor::Portal(_)
            | UiOverlayAnchor::Backdrop(_) => None,
        })
}

fn reachable(
    source: UiOverlayAnchor,
    target: UiOverlayAnchor,
    edges: &BTreeMap<UiOverlayAnchor, BTreeSet<UiOverlayAnchor>>,
) -> bool {
    let mut pending = vec![source];
    let mut visited = BTreeSet::new();
    while let Some(node) = pending.pop() {
        if node == target {
            return true;
        }
        if visited.insert(node) {
            if let Some(successors) = edges.get(&node) {
                pending.extend(successors.iter().copied());
            }
        }
    }
    false
}
