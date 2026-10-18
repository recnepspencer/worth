//! Sealing consumes same-snapshot validation or compares every carried fact.

use super::retained::*;
use super::{CompletedComputationRetention, SealedComputationRetention};
use crate::domain_computation::primary_graph::application_attempt::SealedComputationFacts;

impl CompletedComputationRetention {
    /// New index storage is admitted before seal builds it, with the run's
    /// existing request reservation held until lineage retention takes custody.
    pub(in crate::domain_computation::primary_graph) fn reserve_fact_index(
        &mut self,
        bytes: Option<u64>,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        Option<worth_execution::ExecutionMemoryReservation>,
        crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial,
    > {
        use crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial as Resource;
        match self {
            Self::Produced(run) => {
                run.tree_memory
                    .grow(bytes.ok_or(Resource::CapacityOverflow)?)?;
                Ok(None)
            }
            // No retained run can hold temporary attribution. Its caller's
            // explicit allocation policy owns that storage until completion.
            Self::Absent(_) => match (bytes, policy) {
                (Some(bytes), worth_execution::ExecutionAllocationPolicy::Execution(lease)) => {
                    lease
                        .reserve_memory(bytes)
                        .map(Some)
                        .map_err(super::super::super::execution_denial::memory_denial)
                }
                _ => Ok(None),
            },
        }
    }
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn seal(
        self,
        facts: Option<SealedComputationFacts>,
    ) -> Result<SealedComputationRetention, ()> {
        self.seal_at_optional(facts, None)
    }
    pub(in crate::domain_computation::primary_graph) fn seal_at(
        self,
        facts: Option<SealedComputationFacts>,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<SealedComputationRetention, ()> {
        self.seal_at_optional(facts, Some(snapshot))
    }
    fn seal_at_optional(
        self,
        facts: Option<SealedComputationFacts>,
        snapshot: Option<&worth_relational::facade::snapshots::SnapshotHandle>,
    ) -> Result<SealedComputationRetention, ()> {
        match self {
            Self::Produced(run) => run.seal_at(
                facts.expect("one completed run retains its fact attribution"),
                snapshot,
            ),
            Self::Absent(reason) => Ok(SealedComputationRetention::Absent(reason)),
        }
    }
}

impl CompletedComputationRun {
    /// Unqualified carriage compares original prior facts. Native-qualified
    /// carriage is consumed only at the exact reader snapshot that checked it.
    fn seal_at(
        self,
        facts: SealedComputationFacts,
        snapshot: Option<&worth_relational::facade::snapshots::SnapshotHandle>,
    ) -> Result<SealedComputationRetention, ()> {
        if let Some(carried) = &self.carried
            && !(snapshot.is_some() && snapshot == carried.validated_at.as_ref())
        {
            let unchanged = carried
                .prior
                .facts
                .facts()
                .inspect(|_| {
                    super::super::super::super::fact_verification_observation::carried_seal()
                })
                .filter(|(_, _, readers)| readers.reads().any(|read| carried.carried(read)))
                .all(|(key, fact, _)| facts.fact(key) == Some(fact));
            if !unchanged {
                return Err(());
            }
        }
        let checked_calls = facts.bind_call_shape(
            self.call_sequence.shape(),
            self.carried
                .as_ref()
                .and_then(|carried| carried.prior.checked_calls.as_ref()),
        );
        let bytes = self
            .typed_bytes
            .zip(facts.charged_bytes())
            .and_then(|(typed, facts)| typed.checked_add(facts));
        Ok(SealedComputationRetention::Produced(SealedComputationRun {
            state: RetainedComputation {
                basis: self.basis,
                typed: self.typed,
                facts,
                checked_calls,
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
