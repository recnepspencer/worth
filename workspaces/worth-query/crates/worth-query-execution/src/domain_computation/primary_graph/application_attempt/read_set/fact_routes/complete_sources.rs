//! Index complete source bodies once, retaining their actual physical support.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::StoreDenial;
pub(super) fn entry_count(fact: &Fact) -> Result<u64, StoreDenial> {
    let counts = match fact {
        Fact::StructuralWalk { observation } => (
            observation.native().visits().len(),
            observation.native().relations().len(),
        ),
        Fact::DerivedSource { observation } => (
            observation.native().data().entity_dependencies().len(),
            observation.native().data().physical_relations().len(),
        ),
        _ => (0, 0),
    };
    counts
        .0
        .checked_add(counts.1)
        .and_then(|n| u64::try_from(n).ok())
        .ok_or(StoreDenial::Representability)
}
impl FactRoutes {
    pub(super) fn index_complete_sources(
        &mut self,
        fact: &Fact,
        ordinal: usize,
        control: StorageControl<'_, '_>,
    ) -> Result<(), StoreDenial> {
        match fact {
            Fact::StructuralWalk { observation } => self.index_body(
                observation
                    .native()
                    .visits()
                    .iter()
                    .map(|v| v.entity.entity_id),
                observation
                    .native()
                    .relations()
                    .iter()
                    .map(|r| r.relation_id),
                ordinal,
                control,
            ),
            Fact::DerivedSource { observation } => self.index_body(
                observation
                    .native()
                    .data()
                    .entity_dependencies()
                    .iter()
                    .map(|(id, _)| *id),
                observation
                    .native()
                    .data()
                    .physical_relations()
                    .iter()
                    .map(|r| r.relation_id),
                ordinal,
                control,
            ),
            _ => Ok(()),
        }
    }
    fn index_body(
        &mut self,
        entities: impl Iterator<Item = EntityId>,
        relations: impl Iterator<Item = RelationId>,
        ordinal: usize,
        control: StorageControl<'_, '_>,
    ) -> Result<(), StoreDenial> {
        for entity in entities {
            control.check_live()?;
            self.complete_entities.entry(entity).or_insert(ordinal);
        }
        for relation in relations {
            control.check_live()?;
            self.relations.entry(relation).or_insert(ordinal);
        }
        Ok(())
    }
}
