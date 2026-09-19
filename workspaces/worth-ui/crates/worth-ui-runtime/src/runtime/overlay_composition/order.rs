use std::collections::{BTreeMap, BTreeSet};

use worth_ui_dsl::{UiBackdropIdentity, UiPortalDeclarationId};

use super::planner::UiOverlayCompositionDenial;
use super::relation_graph::{
    UiCompiledOverlayRelation, UiCompiledOverlayRelationGraph, UiOverlayAnchor,
    UiOverlayRelationKind,
};
use super::snapshot::{
    UiBackdropInstanceIdentity, UiOverlayBackdropInstanceScope, UiOverlayBackdropRow,
    UiOverlayPortalRow, UiOverlayStackParticipant,
};
use super::topological_order::unique_topological_order;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) enum OrderNode {
    Content,
    Portal(crate::runtime::portal::UiPortalIdentity),
    Backdrop(UiBackdropInstanceIdentity),
}

pub(super) fn compile_order(
    portals: &[UiOverlayPortalRow],
    backdrops: &[UiOverlayBackdropRow],
    relations: &UiCompiledOverlayRelationGraph,
) -> Result<(Vec<UiOverlayStackParticipant>, usize), UiOverlayCompositionDenial> {
    let mut nodes = BTreeSet::from([OrderNode::Content]);
    let mut portal_by_declaration =
        BTreeMap::<UiPortalDeclarationId, Vec<&UiOverlayPortalRow>>::new();
    let mut portal_by_identity = BTreeMap::new();
    for portal in portals {
        nodes.insert(OrderNode::Portal(portal.portal()));
        portal_by_declaration
            .entry(portal.declaration())
            .or_default()
            .push(portal);
        portal_by_identity.insert(portal.portal(), portal.clone());
    }
    let mut backdrop_by_declaration =
        BTreeMap::<UiBackdropIdentity, Vec<&UiOverlayBackdropRow>>::new();
    let mut backdrop_by_identity = BTreeMap::new();
    for backdrop in backdrops {
        nodes.insert(OrderNode::Backdrop(backdrop.identity()));
        backdrop_by_declaration
            .entry(backdrop.declaration())
            .or_default()
            .push(backdrop);
        backdrop_by_identity.insert(backdrop.identity(), backdrop.clone());
    }

    let mut edges = BTreeSet::new();
    for pair in portals.windows(2) {
        add_edge(
            &mut edges,
            &mut BTreeSet::new(),
            OrderNode::Portal(pair[0].portal()),
            OrderNode::Portal(pair[1].portal()),
            UiOverlayRelationKind::Precedes,
        )?;
    }
    if let Some(first) = portals.first() {
        add_edge(
            &mut edges,
            &mut BTreeSet::new(),
            OrderNode::Content,
            OrderNode::Portal(first.portal()),
            UiOverlayRelationKind::Precedes,
        )?;
    }
    let mut immediate = BTreeSet::new();
    for relation in relations.relations() {
        expand_relation(
            *relation,
            &portal_by_declaration,
            &backdrop_by_declaration,
            &mut edges,
            &mut immediate,
        )?;
    }
    let ordered = unique_topological_order(&nodes, &edges, &immediate)?;
    let mut position = BTreeMap::new();
    for (index, node) in ordered.iter().enumerate() {
        position.insert(*node, index);
    }
    if immediate.iter().any(|(lower, upper)| {
        position
            .get(lower)
            .zip(position.get(upper))
            .is_none_or(|(lower, upper)| upper.saturating_sub(*lower) != 1)
    }) {
        return Err(UiOverlayCompositionDenial::ImmediateAdjacencyViolated);
    }
    let participants = ordered
        .into_iter()
        .filter_map(|node| match node {
            OrderNode::Content => None,
            OrderNode::Portal(portal) => Some(UiOverlayStackParticipant::Portal(
                portal_by_identity
                    .get(&portal)
                    .expect("ordered Portal remains materialized")
                    .clone(),
            )),
            OrderNode::Backdrop(identity) => Some(UiOverlayStackParticipant::Backdrop(
                backdrop_by_identity
                    .get(&identity)
                    .expect("ordered backdrop remains materialized")
                    .clone(),
            )),
        })
        .collect();
    Ok((participants, edges.len()))
}

fn expand_relation(
    relation: UiCompiledOverlayRelation,
    portals: &BTreeMap<UiPortalDeclarationId, Vec<&UiOverlayPortalRow>>,
    backdrops: &BTreeMap<UiBackdropIdentity, Vec<&UiOverlayBackdropRow>>,
    edges: &mut BTreeSet<(OrderNode, OrderNode)>,
    immediate: &mut BTreeSet<(OrderNode, OrderNode)>,
) -> Result<(), UiOverlayCompositionDenial> {
    match (relation.lower(), relation.upper()) {
        (UiOverlayAnchor::SurfaceContent, UiOverlayAnchor::Backdrop(identity)) => {
            for target in backdrops.get(&identity).into_iter().flatten() {
                add_edge(
                    edges,
                    immediate,
                    OrderNode::Content,
                    OrderNode::Backdrop(target.identity()),
                    relation.kind(),
                )?;
            }
        }
        (UiOverlayAnchor::Backdrop(identity), UiOverlayAnchor::Portal(portal)) => {
            for source in backdrops.get(&identity).into_iter().flatten() {
                let target = matching_portal(source, portal, portals)?;
                add_edge(
                    edges,
                    immediate,
                    OrderNode::Backdrop(source.identity()),
                    OrderNode::Portal(target.portal()),
                    relation.kind(),
                )?;
            }
        }
        (UiOverlayAnchor::Portal(portal), UiOverlayAnchor::Backdrop(identity)) => {
            for target in backdrops.get(&identity).into_iter().flatten() {
                let source = matching_portal(target, portal, portals)?;
                add_edge(
                    edges,
                    immediate,
                    OrderNode::Portal(source.portal()),
                    OrderNode::Backdrop(target.identity()),
                    relation.kind(),
                )?;
            }
        }
        (UiOverlayAnchor::Backdrop(source), UiOverlayAnchor::Backdrop(target)) => {
            for source_row in backdrops.get(&source).into_iter().flatten() {
                let target_row = matching_backdrop(source_row, target, backdrops)?;
                add_edge(
                    edges,
                    immediate,
                    OrderNode::Backdrop(source_row.identity()),
                    OrderNode::Backdrop(target_row.identity()),
                    relation.kind(),
                )?;
            }
        }
        _ => return Err(UiOverlayCompositionDenial::Cycle),
    }
    Ok(())
}

fn add_edge(
    edges: &mut BTreeSet<(OrderNode, OrderNode)>,
    immediate: &mut BTreeSet<(OrderNode, OrderNode)>,
    lower: OrderNode,
    upper: OrderNode,
    kind: UiOverlayRelationKind,
) -> Result<(), UiOverlayCompositionDenial> {
    if lower == upper {
        return Err(UiOverlayCompositionDenial::Cycle);
    }
    edges.insert((lower, upper));
    if kind == UiOverlayRelationKind::ImmediatelyPrecedes {
        immediate.insert((lower, upper));
    }
    Ok(())
}

fn matching_portal(
    source: &UiOverlayBackdropRow,
    target: UiPortalDeclarationId,
    portals: &BTreeMap<UiPortalDeclarationId, Vec<&UiOverlayPortalRow>>,
) -> Result<UiOverlayPortalRow, UiOverlayCompositionDenial> {
    let candidates = portals.get(&target).map_or(&[][..], Vec::as_slice);
    if let UiOverlayBackdropInstanceScope::Portal(portal) = source.identity().scope() {
        return candidates
            .iter()
            .find(|candidate| candidate.portal() == portal)
            .map(|candidate| (*candidate).clone())
            .ok_or(UiOverlayCompositionDenial::MissingPortalAnchor {
                backdrop: source.declaration(),
                portal: target,
            });
    }
    match candidates {
        [] => Err(UiOverlayCompositionDenial::MissingPortalAnchor {
            backdrop: source.declaration(),
            portal: target,
        }),
        [candidate] => Ok((*candidate).clone()),
        _ => Err(UiOverlayCompositionDenial::AmbiguousPortalAnchor {
            backdrop: source.declaration(),
            portal: target,
        }),
    }
}

fn matching_backdrop(
    source: &UiOverlayBackdropRow,
    target: UiBackdropIdentity,
    backdrops: &BTreeMap<UiBackdropIdentity, Vec<&UiOverlayBackdropRow>>,
) -> Result<UiOverlayBackdropRow, UiOverlayCompositionDenial> {
    let candidates = backdrops.get(&target).map_or(&[][..], Vec::as_slice);
    let candidate = match source.identity().scope() {
        UiOverlayBackdropInstanceScope::SurfaceSingleton => {
            if candidates.len() > 1 {
                return Err(UiOverlayCompositionDenial::AmbiguousBackdropAnchor {
                    backdrop: source.declaration(),
                    anchor: target,
                });
            }
            candidates.first().copied()
        }
        UiOverlayBackdropInstanceScope::Portal(portal) => candidates
            .iter()
            .find(|candidate| {
                candidate.identity().scope() == UiOverlayBackdropInstanceScope::Portal(portal)
            })
            .copied(),
    };
    let Some(candidate) = candidate else {
        return if candidates.is_empty() {
            Err(UiOverlayCompositionDenial::MissingBackdropAnchor {
                backdrop: source.declaration(),
                anchor: target,
            })
        } else {
            Err(UiOverlayCompositionDenial::CrossScopeBackdropAnchor {
                backdrop: source.declaration(),
                anchor: target,
            })
        };
    };
    Ok(candidate.clone())
}
