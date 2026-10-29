//! Derived postings in the existing installed custom-rule registry.
use crate::identity::data::KindId;
use crate::validation::data::CustomInvariantRegistration;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default, Debug, Clone)]
pub(crate) struct SourceProgramPostings {
    entities: BTreeMap<KindId, BTreeSet<usize>>,
    relations: BTreeMap<KindId, BTreeSet<usize>>,
}

impl SourceProgramPostings {
    pub(crate) fn compile(registrations: &[CustomInvariantRegistration]) -> Self {
        let mut postings = Self::default();
        for (slot, registration) in registrations.iter().enumerate() {
            for source in registration.managed_sources.iter() {
                for kind in [source.scope_kind(), source.row_kind()] {
                    postings.entities.entry(kind).or_default().insert(slot);
                }
                for &(relation, from, to) in source.0.path.physical_sources() {
                    postings.relations.entry(relation).or_default().insert(slot);
                    for kind in [from, to] {
                        postings.entities.entry(kind).or_default().insert(slot);
                    }
                }
            }
        }
        postings
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
