//! Build mixed-intent proposals and their literal candidate images.
use super::{
    model::{Edge, Model, Node, Rng},
    Tally,
};
use std::collections::BTreeSet;
use worth_relational::facade::{
    identity::{KindId, PartitionId},
    symbols::ClientKey,
    transactions::*,
};

pub(super) fn build(
    model: &Model,
    seed: u64,
    split: bool,
    step: usize,
    rng: &mut Rng,
    tally: &mut Tally,
) -> (Model, WorkerIntentBatch, Vec<String>) {
    let mut proposal = Proposal {
        model: model.clone(),
        intents: Vec::new(),
        touched_nodes: BTreeSet::new(),
        touched_edges: BTreeSet::new(),
        log: Vec::new(),
    };
    let intents = 1 + rng.next() % 3;
    for ordinal in 0..intents {
        let live = proposal.model.live_nodes();
        let key =
            |what: &str| ClientKey::raw(format!("mixed-{seed}-{split}-{step}-{ordinal}-{what}"));
        let action = if live.is_empty() { 6 } else { rng.next() % 9 };
        match action {
            0 | 7 => proposal.retire_relation(action == 7, key("restore"), rng, tally),
            1 => proposal.retarget(&live, rng),
            2 | 3 => proposal.insert_relation(&live, key("edge"), rng, tally),
            4 | 8 => proposal.edit_entity(&live, key("replacement"), rng, tally),
            _ => proposal.create_entity(split, key("entity"), rng, tally),
        }
    }
    let batch = proposal
        .intents
        .into_iter()
        .fold(WorkerIntentBatch::new("mixed-laws"), |batch, intent| {
            batch.push(intent)
        });
    (proposal.model, batch, proposal.log)
}

struct Proposal {
    model: Model,
    intents: Vec<MutationIntent>,
    touched_nodes: BTreeSet<usize>,
    touched_edges: BTreeSet<usize>,
    log: Vec<String>,
}
impl Proposal {
    fn relation(&self, rng: &mut Rng) -> Option<usize> {
        let candidates: Vec<_> = self
            .model
            .edges
            .iter()
            .enumerate()
            .filter_map(|(index, edge)| {
                edge.as_ref()
                    .filter(|edge| {
                        edge.kind == 9
                            && edge.id.is_some()
                            && !self.touched_edges.contains(&index)
                            && !self.touched_nodes.contains(&edge.s)
                            && !self.touched_nodes.contains(&edge.t)
                    })
                    .map(|_| index)
            })
            .collect();
        (!candidates.is_empty()).then(|| candidates[rng.next() % candidates.len()])
    }
    fn add_relation(&mut self, kind: u32, s: usize, t: usize, key: ClientKey) {
        let spec = RelationSpec {
            partition_id: self.model.nodes[s].as_ref().unwrap().partition,
            kind_id: KindId(kind),
            client_key: key.clone(),
            source: self.model.reference(s),
            target: self.model.reference(t),
            fields: Default::default(),
        };
        let reference = CreatedRelationRef {
            partition_id: spec.partition_id,
            kind_id: spec.kind_id,
            client_key: key,
            source: spec.source.clone(),
            target: spec.target.clone(),
        };
        self.intents
            .push(MutationIntent::Create(CreateIntent::Relation(spec)));
        self.model.edges.push(Some(Edge {
            id: None,
            key: Some(reference),
            kind,
            s,
            t,
        }));
        self.touched_edges.insert(self.model.edges.len() - 1);
    }
    fn retire_relation(&mut self, restore: bool, key: ClientKey, rng: &mut Rng, tally: &mut Tally) {
        let Some(index) = self.relation(rng) else {
            return;
        };
        let edge = self.model.edges[index].take().unwrap();
        self.intents
            .push(MutationIntent::Relation(RelationMutationIntent::Delete(
                DeleteRelationIntent {
                    relation_id: edge.id.unwrap(),
                },
            )));
        self.touched_edges.insert(index);
        self.log.push(format!("del {}->{}", edge.s, edge.t));
        if restore {
            self.add_relation(9, edge.s, edge.t, key);
            tally.same_commit_restores += 1;
            self.log.push(format!("restore {}->{}", edge.s, edge.t));
        }
    }
    fn retarget(&mut self, live: &[usize], rng: &mut Rng) {
        let Some(index) = self.relation(rng) else {
            return;
        };
        let edge = self.model.edges[index].clone().unwrap();
        let t = live[rng.next() % live.len()];
        let s = if rng.next().is_multiple_of(2) {
            live[rng.next() % live.len()]
        } else {
            edge.s
        };
        if t == s
            || self.touched_nodes.contains(&t)
            || self.touched_nodes.contains(&s)
            || self.model.has_pair(9, s, t)
            || self.model.nodes[t].as_ref().unwrap().id.is_none()
            || self.model.nodes[s].as_ref().unwrap().id.is_none()
        {
            return;
        }
        self.intents.push(MutationIntent::Relation(
            RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                relation_id: edge.id.unwrap(),
                kind_id: KindId(9),
                source: self.model.reference(s),
                target: self.model.reference(t),
            }),
        ));
        self.touched_edges.insert(index);
        self.model.edges[index].as_mut().unwrap().s = s;
        self.model.edges[index].as_mut().unwrap().t = t;
        self.log
            .push(format!("retarget {}->{} => {s}->{t}", edge.s, edge.t));
    }
    fn insert_relation(
        &mut self,
        live: &[usize],
        key: ClientKey,
        rng: &mut Rng,
        tally: &mut Tally,
    ) {
        let kind = if rng.next().is_multiple_of(3) { 10 } else { 9 };
        let (s, t) = (live[rng.next() % live.len()], live[rng.next() % live.len()]);
        if s == t || self.model.has_pair(kind, s, t) {
            return;
        }
        if [s, t].iter().any(|n| {
            self.model.nodes[*n].as_ref().unwrap().id.is_some() && self.touched_nodes.contains(n)
        }) {
            return;
        }
        self.add_relation(kind, s, t, key);
        if kind == 10 {
            tally.unrelated += 1;
        }
        self.log.push(format!("insert k{kind} {s}->{t}"));
    }
    fn edit_entity(&mut self, live: &[usize], key: ClientKey, rng: &mut Rng, tally: &mut Tally) {
        let index = live[rng.next() % live.len()];
        let node = self.model.nodes[index].clone().unwrap();
        let incident_touched = self.model.edges.iter().enumerate().any(|(e, edge)| {
            edge.as_ref()
                .is_some_and(|edge| edge.s == index || edge.t == index)
                && self.touched_edges.contains(&e)
        });
        if node.id.is_none() || self.touched_nodes.contains(&index) || incident_touched {
            return;
        }
        self.touched_nodes.insert(index);
        let replace = rng.next().is_multiple_of(2);
        self.model.cascade(index);
        if replace {
            let kind = 1 + (rng.next() % 3) as u32;
            self.intents
                .push(MutationIntent::Entity(EntityMutationIntent::Replace(
                    ReplaceEntityIntent {
                        entity_id: node.id.unwrap(),
                        replacement: EntitySpec {
                            partition_id: node.partition,
                            kind_id: KindId(kind),
                            client_key: key,
                            fields: Default::default(),
                        },
                    },
                )));
            // Replacements remain in the verdict model, but aren't selected for
            // later edits because Replace does not return a CreatedEntityRef.
            self.model.nodes.push(Some(Node {
                opaque: true,
                id: None,
                key: None,
                kind,
                partition: node.partition,
            }));
            self.touched_nodes.insert(self.model.nodes.len() - 1);
            tally.replaces += 1;
            self.log
                .push(format!("replace {index} (k{}) -> k{kind}", node.kind));
        } else {
            self.intents
                .push(MutationIntent::Entity(EntityMutationIntent::Delete(
                    DeleteEntityIntent {
                        entity_id: node.id.unwrap(),
                    },
                )));
            tally.entity_deletes += 1;
            self.log
                .push(format!("delete entity {index} (k{})", node.kind));
        }
    }
    fn create_entity(&mut self, split: bool, key: ClientKey, rng: &mut Rng, tally: &mut Tally) {
        let kind = 1 + (rng.next() % 3) as u32;
        let partition = if split {
            PartitionId(1 + (rng.next() % 4) as u32)
        } else {
            PartitionId::main()
        };
        let created = CreatedEntityRef {
            partition_id: partition,
            kind_id: KindId(kind),
            client_key: key.clone(),
        };
        self.intents
            .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: partition,
                kind_id: KindId(kind),
                client_key: key,
                fields: Default::default(),
            })));
        self.model.nodes.push(Some(Node {
            opaque: false,
            id: None,
            key: Some(created),
            kind,
            partition,
        }));
        tally.creates += 1;
        self.log
            .push(format!("create k{kind} as {}", self.model.nodes.len() - 1));
    }
}
