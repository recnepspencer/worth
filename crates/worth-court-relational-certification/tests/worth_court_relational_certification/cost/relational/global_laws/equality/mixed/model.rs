//! Literal records and exhaustive kind-filtered DFS; no SDK adjacency.
use std::collections::BTreeSet;
use worth_relational::facade::{
    identity::{EntityId, PartitionId, RelationId},
    transactions::*,
};
#[derive(Clone)]
pub(super) struct Node {
    pub(super) opaque: bool,
    pub(super) id: Option<EntityId>,
    pub(super) key: Option<CreatedEntityRef>,
    pub(super) kind: u32,
    pub(super) partition: PartitionId,
}
#[derive(Clone)]
pub(super) struct Edge {
    pub(super) id: Option<RelationId>,
    pub(super) key: Option<CreatedRelationRef>,
    pub(super) kind: u32,
    pub(super) s: usize,
    pub(super) t: usize,
}
#[derive(Clone, Default)]
pub(super) struct Model {
    pub(super) nodes: Vec<Option<Node>>,
    pub(super) edges: Vec<Option<Edge>>,
}
impl Model {
    pub(super) fn valid(&self) -> bool {
        let live: Vec<_> = self
            .edges
            .iter()
            .flatten()
            .filter(|e| e.kind == 9)
            .collect();
        self.nodes.iter().enumerate().all(|(i, n)| match n {
            Some(n) if n.kind == 1 => {
                let mut seen = BTreeSet::from([i]);
                let mut pending = vec![i];
                let mut ok = false;
                while let Some(v) = pending.pop() {
                    for e in &live {
                        if e.s == v && seen.insert(e.t) {
                            if self.nodes[e.t].as_ref().unwrap().kind == 3 {
                                ok = true;
                            }
                            pending.push(e.t);
                        }
                    }
                }
                ok
            }
            _ => true,
        })
    }
    pub(super) fn reference(&self, i: usize) -> EntityReference {
        let n = self.nodes[i].as_ref().unwrap();
        match (&n.id, &n.key) {
            (Some(id), _) => EntityReference::Existing(*id),
            (None, Some(key)) => EntityReference::Created(key.clone()),
            _ => unreachable!(),
        }
    }
    pub(super) fn live_nodes(&self) -> Vec<usize> {
        (0..self.nodes.len())
            .filter(|&i| self.nodes[i].as_ref().is_some_and(|n| !n.opaque))
            .collect()
    }
    pub(super) fn has_pair(&self, kind: u32, s: usize, t: usize) -> bool {
        self.edges
            .iter()
            .flatten()
            .any(|e| e.kind == kind && e.s == s && e.t == t)
    }
    pub(super) fn cascade(&mut self, node: usize) {
        self.nodes[node] = None;
        for e in self.edges.iter_mut() {
            if e.as_ref().is_some_and(|e| e.s == node || e.t == node) {
                *e = None;
            }
        }
    }
}

pub(super) struct Rng(pub(super) u64);
impl Rng {
    pub(super) fn next(&mut self) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as usize
    }
}

impl Model {
    pub(super) fn from_world(
        world: &worth_testkit_relational::maintenance::relation_laws::World,
    ) -> Self {
        let mut model = Self::default();
        for (i, id) in world.nodes.iter().enumerate() {
            model.nodes.push(Some(Node {
                opaque: false,
                id: Some(*id),
                key: None,
                kind: match i % 4 {
                    0 => 1,
                    1 => 2,
                    _ => 3,
                },
                partition: id.partition_id,
            }));
        }
        for (id, s, t) in &world.edges {
            model.edges.push(Some(Edge {
                id: Some(*id),
                key: None,
                kind: 9,
                s: *s,
                t: *t,
            }));
        }
        model
    }
    pub(super) fn bind_created(
        &mut self,
        result: &worth_relational::facade::transactions::CommitResult,
    ) {
        for node in self.nodes.iter_mut().flatten() {
            if node.id.is_none() && !node.opaque {
                node.id = Some(
                    result
                        .created_entity(node.key.as_ref().unwrap())
                        .expect("created entity"),
                );
            }
        }
        for edge in self.edges.iter_mut().flatten() {
            if edge.id.is_none() {
                edge.id = Some(
                    result
                        .created_relation(edge.key.as_ref().unwrap())
                        .expect("created relation"),
                );
            }
        }
    }
}
