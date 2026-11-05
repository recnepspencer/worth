//! Derived group postings in the existing installed registry.
use crate::identity::data::KindId;
use crate::validation::candidate_source::SourcePreparationGroup;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default, Debug)]
pub(crate) struct SourceProgramPostings {
    entities: BTreeMap<KindId, BTreeSet<usize>>,
    relations: BTreeMap<KindId, BTreeSet<usize>>,
}

impl SourceProgramPostings {
    pub(crate) fn compile(
        groups: &[SourcePreparationGroup],
        control: crate::schema::SchemaCompilationControl<'_, '_>,
    ) -> Result<Self, crate::validation::CustomInvariantCompilationDenial> {
        use crate::validation::CustomInvariantCompilationDenial as Denial;
        let mut postings = Self::default();
        for (slot, group) in groups.iter().enumerate() {
            for source in group.programs() {
                control.checkpoint().map_err(Denial::Execution)?;
                for kind in [source.scope_kind(), source.row_kind()] {
                    postings.entities.entry(kind).or_default().insert(slot);
                }
                for &(relation, from, to) in source.physical_sources() {
                    control.checkpoint().map_err(Denial::Execution)?;
                    postings.relations.entry(relation).or_default().insert(slot);
                    for kind in [from, to] {
                        postings.entities.entry(kind).or_default().insert(slot);
                    }
                }
            }
        }
        Ok(postings)
    }

    pub(crate) fn affected(
        &self,
        entities: impl Iterator<Item = KindId>,
        relations: impl Iterator<Item = KindId>,
    ) -> BTreeSet<usize> {
        let mut affected = BTreeSet::new();
        for kind in entities {
            if let Some(slots) = self.entities.get(&kind) {
                affected.extend(slots);
            }
        }
        for kind in relations {
            if let Some(slots) = self.relations.get(&kind) {
                affected.extend(slots);
            }
        }
        affected
    }
}
