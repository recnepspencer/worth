use std::collections::{BTreeMap, BTreeSet};

use super::{UiStaticBackdropDeclaration, UiStaticBackdropPlacement};

const SURFACE_CONTENT: &str = "@surface-content";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiStaticOverlayRelationGraph {
    relations: Box<[UiStaticOverlayRelation]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiStaticOverlayRelation {
    before: Box<str>,
    after: Box<str>,
    kind: UiStaticOverlayRelationKind,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UiStaticOverlayRelationKind {
    Precedes,
    ImmediatelyPrecedes,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiStaticOverlayRelationGraphDenial {
    BackdropCapacityExceeded,
    DuplicateParticipant,
    MissingAnchor,
    SelfRelation,
    Cycle,
    ConflictingImmediateAdjacency,
    ForeignSurfaceAnchor,
    AmbiguousOrder,
}

impl UiStaticOverlayRelationGraph {
    pub fn admit(
        portals: &[String],
        backdrops: &[UiStaticBackdropDeclaration],
    ) -> Result<Self, UiStaticOverlayRelationGraphDenial> {
        if backdrops.len() > crate::UI_APPEARANCE_BACKDROP_RELATION_CAPACITY {
            return Err(UiStaticOverlayRelationGraphDenial::BackdropCapacityExceeded);
        }
        let mut nodes = BTreeSet::from([SURFACE_CONTENT.to_owned()]);
        for portal in portals {
            if !nodes.insert(portal.clone()) {
                return Err(UiStaticOverlayRelationGraphDenial::DuplicateParticipant);
            }
        }
        for backdrop in backdrops {
            if !nodes.insert(backdrop.identity().to_owned()) {
                return Err(UiStaticOverlayRelationGraphDenial::DuplicateParticipant);
            }
        }
        let mut edges = BTreeMap::<String, BTreeSet<String>>::new();
        let mut predecessors = BTreeMap::<String, String>::new();
        let mut successors = BTreeMap::<String, String>::new();
        let mut relations = Vec::new();
        for backdrop in backdrops {
            if foreign_surface_anchor(backdrop, backdrops) {
                return Err(UiStaticOverlayRelationGraphDenial::ForeignSurfaceAnchor);
            }
            let (before, after, kind) = relation(backdrop);
            if before == after {
                return Err(UiStaticOverlayRelationGraphDenial::SelfRelation);
            }
            if !nodes.contains(&before) || !nodes.contains(&after) {
                return Err(UiStaticOverlayRelationGraphDenial::MissingAnchor);
            }
            if kind == UiStaticOverlayRelationKind::ImmediatelyPrecedes
                && (successors.insert(before.clone(), after.clone()).is_some()
                    || predecessors.insert(after.clone(), before.clone()).is_some())
            {
                return Err(UiStaticOverlayRelationGraphDenial::ConflictingImmediateAdjacency);
            }
            edges
                .entry(before.clone())
                .or_default()
                .insert(after.clone());
            relations.push(UiStaticOverlayRelation {
                before: before.into_boxed_str(),
                after: after.into_boxed_str(),
                kind,
            });
        }
        ensure_acyclic(&nodes, &edges)?;
        ensure_unambiguous(backdrops, &edges)?;
        relations.sort_by(|left, right| {
            left.before
                .cmp(&right.before)
                .then(left.after.cmp(&right.after))
                .then(left.kind.cmp(&right.kind))
        });
        Ok(Self {
            relations: relations.into_boxed_slice(),
        })
    }

    pub fn relations(&self) -> &[UiStaticOverlayRelation] {
        &self.relations
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = b"worth-ui:overlay-relations:v2".to_vec();
        bytes.extend_from_slice(&(self.relations.len() as u64).to_le_bytes());
        for relation in &self.relations {
            bytes.push(match relation.kind {
                UiStaticOverlayRelationKind::Precedes => 1,
                UiStaticOverlayRelationKind::ImmediatelyPrecedes => 2,
            });
            text(&mut bytes, &relation.before);
            text(&mut bytes, &relation.after);
        }
        bytes
    }
}

impl UiStaticOverlayRelation {
    pub fn before(&self) -> &str {
        &self.before
    }

    pub fn after(&self) -> &str {
        &self.after
    }

    pub const fn kind(&self) -> UiStaticOverlayRelationKind {
        self.kind
    }
}

fn foreign_surface_anchor(
    backdrop: &UiStaticBackdropDeclaration,
    backdrops: &[UiStaticBackdropDeclaration],
) -> bool {
    let Some(anchor) = (match backdrop.placement() {
        UiStaticBackdropPlacement::ImmediatelyBeforeBackdrop(anchor)
        | UiStaticBackdropPlacement::ImmediatelyAfterBackdrop(anchor) => Some(anchor.as_ref()),
        _ => None,
    }) else {
        return false;
    };
    backdrops
        .iter()
        .find(|candidate| candidate.identity() == anchor)
        .is_some_and(|candidate| candidate.surface() != backdrop.surface())
}

fn relation(
    backdrop: &UiStaticBackdropDeclaration,
) -> (String, String, UiStaticOverlayRelationKind) {
    match backdrop.placement() {
        UiStaticBackdropPlacement::AboveSurfaceContent => (
            SURFACE_CONTENT.to_owned(),
            backdrop.identity().to_owned(),
            UiStaticOverlayRelationKind::Precedes,
        ),
        UiStaticBackdropPlacement::ImmediatelyBeforePortal(anchor) => (
            backdrop.identity().to_owned(),
            anchor.to_string(),
            UiStaticOverlayRelationKind::ImmediatelyPrecedes,
        ),
        UiStaticBackdropPlacement::ImmediatelyAfterPortal(anchor) => (
            anchor.to_string(),
            backdrop.identity().to_owned(),
            UiStaticOverlayRelationKind::ImmediatelyPrecedes,
        ),
        UiStaticBackdropPlacement::ImmediatelyBeforeBackdrop(anchor) => (
            backdrop.identity().to_owned(),
            anchor.to_string(),
            UiStaticOverlayRelationKind::ImmediatelyPrecedes,
        ),
        UiStaticBackdropPlacement::ImmediatelyAfterBackdrop(anchor) => (
            anchor.to_string(),
            backdrop.identity().to_owned(),
            UiStaticOverlayRelationKind::ImmediatelyPrecedes,
        ),
    }
}

fn ensure_acyclic(
    nodes: &BTreeSet<String>,
    edges: &BTreeMap<String, BTreeSet<String>>,
) -> Result<(), UiStaticOverlayRelationGraphDenial> {
    let mut incoming = nodes
        .iter()
        .map(|node| (node, 0_usize))
        .collect::<BTreeMap<_, _>>();
    for successors in edges.values() {
        for successor in successors {
            *incoming
                .get_mut(successor)
                .ok_or(UiStaticOverlayRelationGraphDenial::MissingAnchor)? += 1;
        }
    }
    let mut removed = BTreeSet::new();
    while removed.len() < nodes.len() {
        let available = incoming
            .iter()
            .filter_map(|(node, count)| (*count == 0 && !removed.contains(*node)).then_some(*node))
            .collect::<Vec<_>>();
        if available.is_empty() {
            return Err(UiStaticOverlayRelationGraphDenial::Cycle);
        }
        for node in available {
            removed.insert(node);
            if let Some(successors) = edges.get(node) {
                for successor in successors {
                    *incoming.get_mut(successor).expect("validated successor") -= 1;
                }
            }
        }
    }
    Ok(())
}

fn ensure_unambiguous(
    backdrops: &[UiStaticBackdropDeclaration],
    edges: &BTreeMap<String, BTreeSet<String>>,
) -> Result<(), UiStaticOverlayRelationGraphDenial> {
    for (index, left) in backdrops.iter().enumerate() {
        for right in backdrops.iter().skip(index + 1) {
            if left.surface() != right.surface() {
                continue;
            }
            if !reachable(left.identity(), right.identity(), edges)
                && !reachable(right.identity(), left.identity(), edges)
            {
                return Err(UiStaticOverlayRelationGraphDenial::AmbiguousOrder);
            }
        }
    }
    Ok(())
}

fn text(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

fn reachable(source: &str, target: &str, edges: &BTreeMap<String, BTreeSet<String>>) -> bool {
    let mut pending = vec![source];
    let mut visited = BTreeSet::new();
    while let Some(node) = pending.pop() {
        if node == target {
            return true;
        }
        if visited.insert(node) {
            if let Some(successors) = edges.get(node) {
                pending.extend(successors.iter().map(String::as_str));
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_bytes_preserve_relation_kind() {
        let graph = |kind| UiStaticOverlayRelationGraph {
            relations: Box::new([UiStaticOverlayRelation {
                before: "lower".into(),
                after: "upper".into(),
                kind,
            }]),
        };
        let ordinary = graph(UiStaticOverlayRelationKind::Precedes);
        let immediate = graph(UiStaticOverlayRelationKind::ImmediatelyPrecedes);

        assert_eq!(ordinary.relations()[0].before(), "lower");
        assert_eq!(ordinary.relations()[0].after(), "upper");
        assert_eq!(
            ordinary.relations()[0].kind(),
            UiStaticOverlayRelationKind::Precedes
        );
        assert_ne!(ordinary.canonical_bytes(), immediate.canonical_bytes());
    }
}
