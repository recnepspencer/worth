//! One iterative evaluator for proposed structural views and exact pinned reads.
use super::read::{DerivedRelationAdjacency as Adjacency, DerivedRelationData as Data};
use super::read::{DerivedRelationReadDenial as Denial, DerivedRelationReads};
use super::DerivedRelationNode as Node;
use super::{DerivedRelationDenial as DefinitionDenial, DerivedRelationHandle as Handle};
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::schema::data::RelationalSchemaRegistry;
use crate::validation::data::StructuralRelationRecord;
use std::collections::{BTreeMap, BTreeSet};

pub(super) trait SourceRead {
    fn checkpoint(&mut self, units: usize) -> Result<(), Denial>;
    fn retain(&mut self, bytes: usize) -> Result<(), Denial>;
    fn entity_kind(&mut self, id: EntityId) -> Result<KindId, Denial>;
    fn adjacency(&mut self, probe: Adjacency) -> Result<Vec<StructuralRelationRecord>, Denial>;
    fn reads(&self) -> DerivedRelationReads;
}

#[derive(Clone, Debug)]
struct Key {
    definition: Handle,
    node: usize,
    anchor: EntityId,
    reverse: bool,
}
impl Key {
    // This memo belongs to one admitted registry whose declared definitions are
    // unique. Entry ownership is validated before any key reaches evaluation.
    fn ordering_key(&self) -> (&super::DerivedRelationIdentity, usize, EntityId, bool) {
        (
            &self.definition.parts().identity,
            self.node,
            self.anchor,
            self.reverse,
        )
    }
}
impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.ordering_key() == other.ordering_key()
    }
}
impl Eq for Key {}
impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Key {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ordering_key().cmp(&other.ordering_key())
    }
}
enum Frame {
    Enter(Key),
    Alias {
        key: Key,
        source: Key,
        exclude_self: bool,
    },
    Compose {
        key: Key,
        first: Key,
        second: usize,
    },
    Union {
        key: Key,
        sources: Vec<Key>,
    },
}

struct Evaluation<'a, Port> {
    registry: &'a RelationalSchemaRegistry,
    read: &'a mut Port,
    pending: Vec<Frame>,
    values: BTreeMap<Key, BTreeSet<EntityId>>,
    entities: BTreeMap<EntityId, KindId>,
    relations: BTreeMap<RelationId, StructuralRelationRecord>,
    adjacency: BTreeSet<Adjacency>,
}

pub(super) fn evaluate(
    registry: &RelationalSchemaRegistry,
    handle: &Handle,
    root: EntityId,
    read: &mut impl SourceRead,
) -> Result<Data, Denial> {
    if !registry.admits_derived_relation(handle) {
        return Err(DefinitionDenial::ForeignDefinition.into());
    }
    let key = Key {
        definition: handle.clone(),
        node: handle.parts().nodes.len() - 1,
        anchor: root,
        reverse: false,
    };
    let mut evaluation = Evaluation {
        registry,
        read,
        pending: Vec::new(),
        values: BTreeMap::new(),
        entities: BTreeMap::new(),
        relations: BTreeMap::new(),
        adjacency: BTreeSet::new(),
    };
    evaluation.entity(root, handle.parts().from)?;
    evaluation.push(Frame::Enter(key.clone()))?;
    while let Some(frame) = evaluation.pending.pop() {
        evaluation.read.checkpoint(1)?;
        evaluation.step(frame)?;
    }
    evaluation.read.checkpoint(evaluation.values.len())?;
    evaluation.memo_key_work(&key)?;
    let endpoints = evaluation
        .values
        .remove(&key)
        .expect("complete root evaluation");
    let output_bytes = endpoints
        .len()
        .checked_mul(size_of::<EntityId>())
        .and_then(|n| {
            n.checked_add(
                evaluation
                    .entities
                    .len()
                    .checked_mul(size_of::<(EntityId, KindId)>())?,
            )
        })
        .and_then(|n| {
            n.checked_add(
                evaluation
                    .relations
                    .len()
                    .checked_mul(size_of::<StructuralRelationRecord>())?,
            )
        })
        .and_then(|n| {
            n.checked_add(
                evaluation
                    .adjacency
                    .len()
                    .checked_mul(size_of::<Adjacency>())?,
            )
        })
        .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
    evaluation.read.retain(output_bytes)?;
    Ok(Data {
        endpoints: endpoints.into_iter().collect(),
        entities: evaluation.entities.into_iter().collect(),
        relations: evaluation.relations.into_values().collect(),
        adjacency: evaluation.adjacency.into_iter().collect(),
        reads: evaluation.read.reads(),
    })
}

impl<Port: SourceRead> Evaluation<'_, Port> {
    fn memo_key_work(&mut self, key: &Key) -> Result<(), Denial> {
        let identity = &key.definition.parts().identity;
        let key_bytes = identity
            .schema
            .0
            .len()
            .checked_add(identity.name.len())
            .and_then(|n| n.checked_add(size_of::<Key>()))
            .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
        // Supported BTree nodes compare at most eleven keys per level. Bit
        // width conservatively bounds search depth; each compared string prefix
        // is bounded by this query key's own bytes, without scanning the roster.
        let levels = (usize::BITS - self.values.len().max(1).leading_zeros()) as usize;
        let work = key_bytes
            .checked_mul(11)
            .and_then(|n| n.checked_mul(levels))
            .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
        self.read.checkpoint(work)
    }

    fn push(&mut self, frame: Frame) -> Result<(), Denial> {
        self.read.retain(size_of::<Frame>())?;
        self.pending.push(frame);
        Ok(())
    }
    fn save(&mut self, key: Key, values: BTreeSet<EntityId>) -> Result<(), Denial> {
        self.memo_key_work(&key)?;
        self.read
            .retain(size_of::<(Key, BTreeSet<EntityId>)>() + 4 * size_of::<usize>())?;
        self.values.insert(key, values);
        Ok(())
    }
    fn entity(&mut self, id: EntityId, expected: KindId) -> Result<(), Denial> {
        self.read.checkpoint(1)?;
        let kind = self.read.entity_kind(id)?;
        if kind != expected {
            return Err(DefinitionDenial::DiscontinuousEndpoints.into());
        }
        if !self.entities.contains_key(&id) {
            self.read
                .retain(size_of::<(EntityId, KindId)>() + 4 * size_of::<usize>())?;
            self.entities.insert(id, kind);
        }
        Ok(())
    }
    fn step(&mut self, frame: Frame) -> Result<(), Denial> {
        match frame {
            Frame::Enter(key) => self.enter(key),
            Frame::Alias {
                key,
                source,
                exclude_self,
            } => {
                let mut targets = BTreeSet::new();
                self.memo_key_work(&source)?;
                for endpoint in &self.values[&source] {
                    self.read.checkpoint(1)?;
                    if !exclude_self || *endpoint != key.anchor {
                        self.read
                            .retain(size_of::<EntityId>() + 4 * size_of::<usize>())?;
                        targets.insert(*endpoint);
                    }
                }
                self.save(key, targets)
            }
            Frame::Compose { key, first, second } => {
                let mut sources = Vec::new();
                self.memo_key_work(&first)?;
                for anchor in &self.values[&first] {
                    self.read.checkpoint(1)?;
                    self.read.retain(size_of::<Key>())?;
                    sources.push(Key {
                        node: second,
                        anchor: *anchor,
                        ..key.clone()
                    });
                }
                self.read.retain(sources.len() * size_of::<Key>())?;
                // Each second-leg evaluation runs before the union continuation.
                self.push(Frame::Union {
                    key,
                    sources: sources.clone(),
                })?;
                for source in sources {
                    self.push(Frame::Enter(source))?;
                }
                Ok(())
            }
            Frame::Union { key, sources } => {
                let mut targets = BTreeSet::new();
                for source in sources {
                    self.memo_key_work(&source)?;
                    for endpoint in &self.values[&source] {
                        self.read.checkpoint(1)?;
                        if !targets.contains(endpoint) {
                            self.read
                                .retain(size_of::<EntityId>() + 4 * size_of::<usize>())?;
                            targets.insert(*endpoint);
                        }
                    }
                }
                self.save(key, targets)
            }
        }
    }
    fn enter(&mut self, key: Key) -> Result<(), Denial> {
        self.memo_key_work(&key)?;
        if self.values.contains_key(&key) {
            return Ok(());
        }
        let definition = std::sync::Arc::clone(&key.definition.entry);
        self.read.checkpoint(1)?;
        match &definition.parts.nodes[key.node] {
            Node::Stored { relation, from, to } => self.stored(key, *relation, *from, *to),
            Node::Derived(identity) => {
                self.read
                    .checkpoint(identity.name.len() + identity.schema.0.len() + 1)?;
                let target = self
                    .registry
                    .derived_relation(identity)
                    .expect("registered named source");
                let source = Key {
                    node: target.parts().nodes.len() - 1,
                    definition: target,
                    ..key.clone()
                };
                self.alias(key, source, false)
            }
            Node::Reverse { input } => {
                let source = Key {
                    node: *input,
                    reverse: !key.reverse,
                    ..key.clone()
                };
                self.alias(key, source, false)
            }
            Node::WithoutSelfEdges { input } => {
                let source = Key {
                    node: *input,
                    ..key.clone()
                };
                self.alias(key, source, true)
            }
            Node::Compose { left, right } => {
                let (first, second) = if key.reverse {
                    (*right, *left)
                } else {
                    (*left, *right)
                };
                let first = Key {
                    node: first,
                    ..key.clone()
                };
                self.push(Frame::Compose {
                    key,
                    first: first.clone(),
                    second,
                })?;
                self.push(Frame::Enter(first))
            }
        }
    }
    fn alias(&mut self, key: Key, source: Key, exclude_self: bool) -> Result<(), Denial> {
        self.push(Frame::Alias {
            key,
            source: source.clone(),
            exclude_self,
        })?;
        self.push(Frame::Enter(source))
    }
    fn stored(
        &mut self,
        key: Key,
        relation: KindId,
        from: KindId,
        to: KindId,
    ) -> Result<(), Denial> {
        self.entity(key.anchor, if key.reverse { to } else { from })?;
        let probe = Adjacency {
            anchor: key.anchor,
            relation,
            outgoing: !key.reverse,
        };
        if !self.adjacency.contains(&probe) {
            self.read
                .retain(size_of::<Adjacency>() + 4 * size_of::<usize>())?;
            self.adjacency.insert(probe);
        }
        let mut targets = BTreeSet::new();
        for record in self.read.adjacency(probe)? {
            self.read.checkpoint(1)?;
            let (anchor, target) = if key.reverse {
                (record.target, record.source)
            } else {
                (record.source, record.target)
            };
            if record.kind_id != relation || anchor != key.anchor {
                return Err(DefinitionDenial::InvalidPhysicalSource(relation).into());
            }
            self.entity(record.source, from)?;
            self.entity(record.target, to)?;
            if !self.relations.contains_key(&record.relation_id) {
                self.read
                    .retain(size_of::<StructuralRelationRecord>() + 4 * size_of::<usize>())?;
                self.relations.insert(record.relation_id, record);
            }
            if !targets.contains(&target) {
                self.read
                    .retain(size_of::<EntityId>() + 4 * size_of::<usize>())?;
                targets.insert(target);
            }
        }
        self.save(key, targets)
    }
}
