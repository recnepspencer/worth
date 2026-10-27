//! Iterative recipe navigation, retaining both Boolean branches' support.
use super::memo::Key;
use super::{insert_member, member_work, DefinitionDenial, Denial, Evaluation, SourceRead};
use crate::derived_relation::DerivedRelationNode as Node;
use std::collections::BTreeSet;

pub(super) enum Frame {
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
    Combine {
        key: Key,
        sources: Vec<Key>,
        intersection: bool,
    },
}

impl<Port: SourceRead> Evaluation<'_, Port> {
    pub(super) fn push(&mut self, frame: Frame) -> Result<(), Denial> {
        self.read.retain(size_of::<Frame>())?;
        self.pending.push(frame);
        Ok(())
    }
    pub(super) fn step(&mut self, frame: Frame) -> Result<(), Denial> {
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
                    member_work(self.read, endpoint, 1)?;
                    if !exclude_self || *endpoint != key.anchor {
                        insert_member(self.read, &mut targets, endpoint.clone())?;
                    }
                }
                self.save(key, targets)
            }
            Frame::Compose { key, first, second } => {
                self.memo_key_work(&first)?;
                let count = self.values[&first].len();
                self.read.retain(
                    count
                        .checked_mul(size_of::<Key>())
                        .ok_or(DefinitionDenial::WorkBudgetExceeded)?,
                )?;
                let mut sources = Vec::with_capacity(count);
                for anchor in &self.values[&first] {
                    self.read.checkpoint(1)?;
                    sources.push(Key {
                        node: second,
                        anchor: anchor.clone(),
                        ..key.clone()
                    });
                }
                self.combine(key, sources, false)
            }
            Frame::Combine {
                key,
                sources,
                intersection,
            } => {
                let mut targets = BTreeSet::new();
                if let Some(first) = sources.first() {
                    self.memo_key_work(first)?;
                    for endpoint in &self.values[first] {
                        insert_member(self.read, &mut targets, endpoint.clone())?;
                    }
                }
                for source in sources.iter().skip(1) {
                    self.memo_key_work(source)?;
                    if intersection {
                        // Both source frames have completed, including empty
                        // adjacency reads, before set intersection begins.
                        let right = &self.values[source];
                        let mut retained = BTreeSet::new();
                        for endpoint in targets {
                            member_work(self.read, &endpoint, right.len())?;
                            if right.contains(&endpoint) {
                                insert_member(self.read, &mut retained, endpoint)?;
                            }
                        }
                        targets = retained;
                    } else {
                        for endpoint in &self.values[source] {
                            insert_member(self.read, &mut targets, endpoint.clone())?;
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
                let names = identity
                    .name
                    .len()
                    .checked_add(identity.schema.0.len())
                    .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
                let levels = (usize::BITS
                    - self.registry.derived_relations.len().max(1).leading_zeros())
                    as usize;
                self.read.checkpoint(
                    names
                        .checked_mul(11)
                        .and_then(|n| n.checked_mul(levels))
                        .ok_or(DefinitionDenial::WorkBudgetExceeded)?,
                )?;
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
            Node::CarrierLift { domain, case } => self.carrier_lift(key, domain, case),
            Node::Union { left, right } | Node::Intersection { left, right } => {
                self.read.retain(2 * size_of::<Key>())?;
                let sources = vec![
                    Key {
                        node: *left,
                        ..key.clone()
                    },
                    Key {
                        node: *right,
                        ..key.clone()
                    },
                ];
                let intersection =
                    matches!(&definition.parts.nodes[key.node], Node::Intersection { .. });
                self.combine(key, sources, intersection)
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
    fn combine(&mut self, key: Key, sources: Vec<Key>, intersection: bool) -> Result<(), Denial> {
        self.read.retain(
            sources
                .len()
                .checked_mul(size_of::<Key>())
                .ok_or(DefinitionDenial::WorkBudgetExceeded)?,
        )?;
        self.push(Frame::Combine {
            key,
            sources: sources.clone(),
            intersection,
        })?;
        for source in sources {
            self.push(Frame::Enter(source))?;
        }
        Ok(())
    }
}
