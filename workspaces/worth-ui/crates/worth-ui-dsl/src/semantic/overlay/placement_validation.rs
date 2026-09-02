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
) -> Result<(), UiOverlayRelationAdmissionDenial> {
    for (index, (left, left_surface)) in backdrops.iter().enumerate() {
        for (right, right_surface) in backdrops.iter().skip(index + 1) {
            if left_surface != right_surface {
                continue;
            }
            if !reachable(*left, *right, edges) && !reachable(*right, *left, edges) {
                return Err(UiOverlayRelationAdmissionDenial::AmbiguousOrder);
            }
        }
    }
    Ok(())
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
        .or_else(|| match anchor {
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
