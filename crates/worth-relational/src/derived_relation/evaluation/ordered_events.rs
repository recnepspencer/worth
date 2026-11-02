//! Integrate the complete event acquisition into the one evaluator's support.
use super::memo::Key;
use super::*;
use crate::derived_relation::ordered_events::{self, Recipe};

impl<Port: SourceRead> Evaluation<'_, Port> {
    pub(super) fn ordered_events(&mut self, key: Key, recipe: Recipe) -> Result<(), Denial> {
        let anchor = key
            .anchor
            .entity()
            .ok_or(DefinitionDenial::InvalidMemberDomain)?;
        let acquired = ordered_events::acquire(
            self.registry,
            &key.definition,
            recipe,
            anchor,
            key.reverse,
            self.read,
        )?;
        for data in acquired.support {
            for (entity, kind) in data.entities {
                self.read.checkpoint(1)?;
                if !self.entities.contains_key(&entity) {
                    self.read
                        .retain(size_of::<(EntityId, KindId)>() + 4 * size_of::<usize>())?;
                    self.entities.insert(entity, kind);
                }
            }
            for record in data.relations {
                self.read.checkpoint(1)?;
                if !self.relations.contains_key(&record.relation_id) {
                    self.read
                        .retain(size_of::<StructuralRelationRecord>() + 4 * size_of::<usize>())?;
                    self.relations.insert(record.relation_id, record);
                }
            }
            for probe in data.adjacency {
                self.read.checkpoint(1)?;
                if !self.adjacency.contains(&probe) {
                    self.read
                        .retain(size_of::<Adjacency>() + 4 * size_of::<usize>())?;
                    self.adjacency.insert(probe);
                }
            }
        }
        self.save(key, acquired.members)
    }
}
