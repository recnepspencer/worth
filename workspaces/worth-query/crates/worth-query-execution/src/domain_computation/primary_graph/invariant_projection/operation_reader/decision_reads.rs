//! The decision facts one operation projection read, and which of them a
//! partitioned computation read in which owner call.

use std::collections::BTreeSet;

use super::WorthQueryApplicationOperationInvariantProjectionReader;
use crate::domain_computation::primary_graph::application_attempt::{
    ComputationFactAttribution, ComputationRead, WorthQueryApplicationFactKey,
};
use crate::domain_computation::primary_graph::application_contribution::{
    CompletedComputationRetention, ComputationDeposit, ComputationPrior, PriorAbsence, Suppression,
};

/// Every decision fact key of one projection.
///
/// A key read while an owner call of a partitioned computation holds the
/// reader is recorded with that call, and one the handler read itself is
/// recorded as the handler's. A key both read is both.
#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct DecisionReads {
    handler: BTreeSet<WorthQueryApplicationFactKey>,
    /// The first read that failed. Its outcome depends on what its fact does
    /// not hold, so a handler or owner call that swallowed the failure has an
    /// outcome nothing retained, and the projection seals nothing.
    failed: Option<WorthQueryApplicationFactKey>,
    computation: ComputationFactAttribution,
    reading: Option<ComputationRead>,
    runs: ComputationRuns,
    /// What the producer that runs the projection retained, for the first
    /// partitioned computation to take.
    prior: ComputationPriorCustody,
    /// Where that computation leaves its completed run, when a producer runs
    /// the projection.
    deposit: Option<ComputationDeposit>,
}

/// A reader either has no producer, owns the handed prior, or has consumed it.
#[derive(Default)]
enum ComputationPriorCustody {
    #[default]
    NoProducer,
    Handed(ComputationPrior),
    Taken,
}

pub(in crate::domain_computation::primary_graph) enum ComputationPriorUnavailable {
    NoProducerPrior,
    PriorAlreadyTaken,
}
impl ComputationPriorUnavailable {
    pub(in crate::domain_computation::primary_graph) fn suppression(&self) -> Suppression {
        match self {
            Self::NoProducerPrior => Suppression::NoProducerPrior,
            Self::PriorAlreadyTaken => Suppression::PriorAlreadyTaken,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn full_cause(&self) -> crate::domain_computation::primary_graph::application_contribution::WorthQueryPartitionedComputationFullCause{
        PriorAbsence::Suppressed(self.suppression()).full_cause()
    }
}

/// What became of one decision read. A field, entity or relation read
/// records an absent value as its content, so only a traversal can fail.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DecisionReadOutcome {
    Observed,
    Failed,
}

/// How many partitioned computations read through the projection's reader.
#[derive(Default)]
enum ComputationRuns {
    #[default]
    None,
    One,
    Several,
}

impl DecisionReads {
    /// The reads of a projection a producer runs, handed what its selected
    /// record retained.
    pub(in crate::domain_computation::primary_graph) fn with_prior(
        prior: ComputationPrior,
    ) -> Self {
        Self {
            prior: ComputationPriorCustody::Handed(prior),
            deposit: Some(ComputationDeposit::new()),
            ..Self::default()
        }
    }

    /// The one way a decision read is recorded, on every outcome of the read.
    pub(super) fn record(
        &mut self,
        key: WorthQueryApplicationFactKey,
        outcome: DecisionReadOutcome,
    ) {
        if outcome == DecisionReadOutcome::Failed && self.failed.is_none() {
            self.failed = Some(key.clone());
        }
        match self.reading {
            Some(read) => self.computation.record(key, read),
            None => {
                self.handler.insert(key);
            }
        }
    }

    /// The keys the read attempt re-observes: every key read, by the handler
    /// or by an owner call. With them, what the computation read.
    ///
    /// A key the handler read beside an owner call stays that call's: the
    /// handler reads its own facts again on every attempt. There is no
    /// attribution unless exactly one partitioned computation ran: partition
    /// identities name the partitions of one computation only.
    /// The deposit is there only when exactly one partitioned computation
    /// ran under a producer. A projection with a failed read has no keys to
    /// re-observe: the key of the first failed read is returned instead.
    pub(in crate::domain_computation::primary_graph) fn into_expected(
        self,
    ) -> Result<
        (
            BTreeSet<WorthQueryApplicationFactKey>,
            (
                Option<ComputationFactAttribution>,
                CompletedComputationRetention,
            ),
        ),
        WorthQueryApplicationFactKey,
    > {
        let Self {
            mut handler,
            failed,
            computation,
            runs,
            deposit,
            ..
        } = self;
        if let Some(failed) = failed {
            return Err(failed);
        }
        handler.extend(computation.keys().cloned());
        let result = match runs {
            ComputationRuns::None => (
                None,
                CompletedComputationRetention::Absent(PriorAbsence::NotProduced),
            ),
            ComputationRuns::One => (
                Some(computation),
                deposit.expect("begin creates the run deposit").take(),
            ),
            ComputationRuns::Several => {
                // Attribution identifies one computation only. Drop its state here
                // with the precise reason, rather than pretending no run happened.
                drop(deposit);
                (
                    None,
                    CompletedComputationRetention::Absent(PriorAbsence::Suppressed(
                        Suppression::Several,
                    )),
                )
            }
        };
        Ok((handler, result))
    }
}

impl<Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    /// A partitioned computation begins reading through this reader.
    pub(in crate::domain_computation::primary_graph) fn begin_computation_reads(&mut self) {
        let deposit = self
            .decision_facts
            .deposit
            .get_or_insert_with(ComputationDeposit::new);
        // Preparing is a run, even if its work or later execution stops.
        deposit.write(CompletedComputationRetention::Absent(PriorAbsence::Stopped));
        self.decision_facts.runs = match self.decision_facts.runs {
            ComputationRuns::None => ComputationRuns::One,
            ComputationRuns::One | ComputationRuns::Several => ComputationRuns::Several,
        };
    }

    /// What the producer retained, taken by the first partitioned computation
    /// to begin. Ordinary mutation readers are reachable here: the installed
    /// producer operation can also be invoked without a producer admission.
    /// Ordinary readers get NoProducerPrior. Once a producer prior was taken,
    /// later invocations get NoPriorHanded.
    pub(in crate::domain_computation::primary_graph) fn take_computation_prior(
        &mut self,
    ) -> Result<ComputationPrior, ComputationPriorUnavailable> {
        match &mut self.decision_facts.prior {
            ComputationPriorCustody::NoProducer => {
                Err(ComputationPriorUnavailable::NoProducerPrior)
            }
            ComputationPriorCustody::Taken => Err(ComputationPriorUnavailable::PriorAlreadyTaken),
            handed @ ComputationPriorCustody::Handed(_) => {
                let ComputationPriorCustody::Handed(prior) =
                    std::mem::replace(handed, ComputationPriorCustody::Taken)
                else {
                    unreachable!()
                };
                Ok(prior)
            }
        }
    }

    /// Where a completed run is left for seal, when a producer runs the
    /// projection.
    pub(in crate::domain_computation::primary_graph) fn computation_deposit(
        &self,
    ) -> ComputationDeposit {
        self.decision_facts
            .deposit
            .clone()
            .expect("prepare begins computation reads and creates the deposit before accessing it")
    }

    /// Runs one owner call, recording every fact key it reads as that call's.
    pub(in crate::domain_computation::primary_graph) fn attributed<Output>(
        &mut self,
        read: ComputationRead,
        call: impl FnOnce(&mut Self) -> Output,
    ) -> Output {
        self.decision_facts.reading = Some(read);
        let output = call(self);
        self.decision_facts.reading = None;
        output
    }
}

#[cfg(test)]
mod tests {
    use worth_relational::facade::identity::{EntityId, PartitionId};

    use super::{DecisionReadOutcome, DecisionReads};
    use crate::domain_computation::primary_graph::application_attempt::{
        ComputationRead, WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationFactKey,
    };

    fn adjacency(slot: u64) -> WorthQueryApplicationFactKey {
        WorthQueryApplicationFactKey::Adjacency {
            relation: "owner".to_owned(),
            anchor: EntityId::new(PartitionId::main(), slot, 1),
            direction: WorthQueryApplicationAdjacencyDirection::Incoming,
            maximum_work_units: 2,
        }
    }

    #[test]
    fn a_failed_read_an_owner_call_swallowed_seals_nothing() {
        let mut reads = DecisionReads::default();
        reads.record(adjacency(1), DecisionReadOutcome::Observed);
        reads.reading = Some(ComputationRead::Membership);
        reads.record(adjacency(2), DecisionReadOutcome::Failed);
        reads.reading = None;
        reads.record(adjacency(3), DecisionReadOutcome::Failed);
        assert_eq!(reads.into_expected().err(), Some(adjacency(2)));
    }

    #[test]
    fn observed_reads_seal_every_key() {
        let mut reads = DecisionReads::default();
        reads.record(adjacency(1), DecisionReadOutcome::Observed);
        let (keys, _) = reads.into_expected().ok().unwrap();
        assert_eq!(keys.into_iter().collect::<Vec<_>>(), [adjacency(1)]);
    }
}
