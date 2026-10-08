//! Sealing compares every carried fact before retaining a completed run.

use super::retained::*;
use super::{CompletedComputationRetention, SealedComputationRetention};
use crate::domain_computation::primary_graph::application_attempt::SealedComputationFacts;

impl CompletedComputationRetention {
    pub(in crate::domain_computation::primary_graph) fn seal(
        self,
        facts: Option<SealedComputationFacts>,
    ) -> Result<SealedComputationRetention, ()> {
        match self {
            Self::Produced(run) => {
                run.seal(facts.expect("one completed run retains its fact attribution"))
            }
            Self::Absent(reason) => Ok(SealedComputationRetention::Absent(reason)),
        }
    }
}

impl CompletedComputationRun {
    /// Seals the run over the facts seal observed. Every fact a carried call
    /// read must be the fact the prior run read: the law that makes a carried
    /// call's answer the answer a full run computes. A call the run did not
    /// carry, such as a removed item's key, holds no fact. `Err` when one
    /// moved.
    pub(in crate::domain_computation::primary_graph) fn seal(
        self,
        facts: SealedComputationFacts,
    ) -> Result<SealedComputationRetention, ()> {
        if let Some(carried) = &self.carried {
            let unchanged = carried
                .prior
                .facts
                .facts()
                .filter(|(_, _, readers)| readers.reads().any(|read| carried.carried(read)))
                .all(|(key, fact, _)| facts.fact(key) == Some(fact));
            if !unchanged {
                return Err(());
            }
        }
        let bytes = self
            .typed_bytes
            .zip(facts.charged_bytes())
            .and_then(|(typed, facts)| typed.checked_add(facts));
        Ok(SealedComputationRetention::Produced(SealedComputationRun {
            state: RetainedComputation {
                basis: self.basis,
                typed: self.typed,
                facts,
                bytes,
            },
            cloned_from: self.carried.map(|carried| carried.prior),
            tree_memory: RunTreeMemory {
                _held: self.tree_memory,
                _routing: self.routing_memory,
            },
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::super::{PriorAbsence, Suppression};
    use super::*;

    /// Absence is explicit: an empty Produced seal cannot be constructed.
    /// Recording writes Unmeasured when it discards unmeasurable state;
    /// sealing carries the dropping site's reason unchanged.
    #[test]
    fn seal_preserves_the_reason_written_by_each_dropping_site() {
        for reason in [
            PriorAbsence::Unmeasured,
            PriorAbsence::Stopped,
            PriorAbsence::Suppressed(Suppression::Policy),
            PriorAbsence::Suppressed(Suppression::Several),
            PriorAbsence::Suppressed(Suppression::Collision),
            PriorAbsence::Suppressed(Suppression::NoProducerPrior),
            PriorAbsence::Suppressed(Suppression::PriorAlreadyTaken),
        ] {
            let expected = reason.clone();
            let sealed = CompletedComputationRetention::Absent(reason)
                .seal(None)
                .unwrap();
            assert!(
                matches!(sealed, SealedComputationRetention::Absent(actual) if actual == expected)
            );
        }
    }
}
