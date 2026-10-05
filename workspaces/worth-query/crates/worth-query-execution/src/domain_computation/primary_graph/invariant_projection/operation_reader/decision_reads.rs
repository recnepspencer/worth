//! The decision facts one operation projection read, and which of them a
//! partitioned computation read in which owner call.

use std::collections::BTreeSet;

use super::WorthQueryApplicationOperationInvariantProjectionReader;
use crate::domain_computation::primary_graph::application_attempt::{
    ComputationFactAttribution, ComputationRead, WorthQueryApplicationFactKey,
};

/// Every decision fact key of one projection.
///
/// A key read while an owner call of a partitioned computation holds the
/// reader is recorded with that call. Every other key is the handler's.
#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct DecisionReads {
    handler: BTreeSet<WorthQueryApplicationFactKey>,
    computation: ComputationFactAttribution,
    reading: Option<ComputationRead>,
    runs: ComputationRuns,
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
    pub(super) fn insert(&mut self, key: WorthQueryApplicationFactKey) {
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
    /// A key the handler read itself is the handler's. There is no attribution
    /// unless exactly one partitioned computation ran: partition identities
    /// name the partitions of one computation only.
    pub(in crate::domain_computation::primary_graph) fn into_expected(
        self,
    ) -> (
        BTreeSet<WorthQueryApplicationFactKey>,
        Option<ComputationFactAttribution>,
    ) {
        let Self {
            mut handler,
            mut computation,
            runs,
            ..
        } = self;
        computation.yield_to_handler(&handler);
        handler.extend(computation.keys().cloned());
        (
            handler,
            matches!(runs, ComputationRuns::One).then_some(computation),
        )
    }
}

impl<Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    /// A partitioned computation begins reading through this reader.
    pub(in crate::domain_computation::primary_graph) fn begin_computation_reads(&mut self) {
        self.decision_facts.runs = match self.decision_facts.runs {
            ComputationRuns::None => ComputationRuns::One,
            ComputationRuns::One | ComputationRuns::Several => ComputationRuns::Several,
        };
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
