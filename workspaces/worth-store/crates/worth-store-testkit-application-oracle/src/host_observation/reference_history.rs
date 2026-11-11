//! Expected history is descriptive reference state, never a Home recovery cache.
use super::{
    comparison::{HostDivergence, ProjectionPart},
    projection::{HeadProjection, OccurrenceProjection},
};

#[derive(Default)]
pub struct ReferenceOccurrencePrefix {
    observed: Vec<OccurrenceProjection>,
}
impl ReferenceOccurrencePrefix {
    pub fn observe(&mut self, occurrences: &[OccurrenceProjection]) -> Result<(), HostDivergence> {
        for (index, occurrence) in occurrences.iter().enumerate() {
            if let Some(prior) = self
                .observed
                .iter()
                .find(|prior| prior.owner == occurrence.owner && prior.commit == occurrence.commit)
            {
                if prior != occurrence {
                    return Err(HostDivergence::Meaning {
                        part: ProjectionPart::ReferenceHistory,
                        index,
                    });
                }
            } else {
                self.observed.push(occurrence.clone());
            }
        }
        Ok(())
    }
    pub fn trace(
        &self,
        head: &HeadProjection,
    ) -> Result<Vec<OccurrenceProjection>, HostDivergence> {
        let mut chain = Vec::new();
        let mut next = Some(head.commit);
        while let Some(commit) = next {
            let failure = || HostDivergence::Meaning {
                part: ProjectionPart::ReferenceHistory,
                index: chain.len(),
            };
            // Repeating a node would exceed the finite set of observed facts.
            if chain.len() >= self.observed.len() {
                return Err(failure());
            }
            let original = self
                .observed
                .iter()
                .find(|occurrence| occurrence.owner == head.owner && occurrence.commit == commit)
                .ok_or_else(failure)?;
            next = original.parent;
            chain.push(original.clone());
        }
        Ok(chain)
    }
}
