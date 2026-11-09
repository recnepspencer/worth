//! The installed event recipe alone selects the predecessor and its step changes.
use super::*;
use crate::derived_relation::{
    evaluation::SourceRead, DerivedRelationAdjacency, DerivedRelationReads, StructuralSource,
};
use crate::identity::data::KindId;
use crate::validation::data::StructuralRelationRecord;
use std::collections::BTreeSet;

impl ManagedIndexSourceScope<'_> {
    pub fn ordered_predecessor(&self) -> Result<Option<EntityId>, Denial> {
        let Some(recipe) = self.program.0.ordered else {
            return Ok(None);
        };
        if !self.scope_exists()? {
            return Ok(None);
        }
        let records = self.event_records(recipe.predecessor)?;
        match records.as_slice() {
            [] => Ok(None),
            [record] if record.target != self.scope => Ok(Some(record.target)),
            _ => Err(Definition::InvalidEventChain.into()),
        }
    }

    /// No roster walk or predecessor reconstruction: only this event's own leaves.
    /// Current before/after membership tickets retain their genuine Native loans.
    pub fn prepare_ordered_step(
        &self,
    ) -> Result<
        Vec<(
            Option<ManagedIndexSourceOccurrence>,
            Option<PreparedManagedIndexSourceOccurrence>,
        )>,
        Denial,
    > {
        let recipe = self.program.0.ordered.ok_or(Denial::ForeignSource)?;
        let parent = self.ordered_predecessor()?.ok_or(Denial::ForeignSource)?;
        if !self.committed_available {
            return Err(Denial::RootUnavailable);
        }
        let before = self.rows_at(true, parent);
        let after = self.proposed();
        let mut rows = BTreeSet::new();
        for kind in [recipe.introduced, recipe.retired] {
            for record in self.event_records(kind)? {
                self.read_budget()
                    .checkpoint(1)
                    .map_err(ReadDenial::Structural)?;
                self.read_budget()
                    .claim_retained((size_of::<EntityId>() * 5) as u64)
                    .map_err(ReadDenial::Structural)?;
                rows.insert(record.target);
            }
        }
        self.read_budget()
            .claim_retained(
                (rows.len()
                    * size_of::<(
                        Option<ManagedIndexSourceOccurrence>,
                        Option<PreparedManagedIndexSourceOccurrence>,
                    )>()) as u64,
            )
            .map_err(ReadDenial::Structural)?;
        rows.into_iter()
            .map(|row| Ok((before.occurrence(row)?, after.prepare_occurrence(row)?)))
            .collect()
    }

    fn event_records(&self, relation: KindId) -> Result<Vec<StructuralRelationRecord>, Denial> {
        self.views
            .after
            .require_derived_relation(&self.program.0.path)?;
        let mut source = StructuralSource {
            view: &self.views.after,
            reads: DerivedRelationReads::default(),
        };
        Ok(source.adjacency(DerivedRelationAdjacency {
            anchor: self.scope,
            relation,
            outgoing: true,
        })?)
    }
}
