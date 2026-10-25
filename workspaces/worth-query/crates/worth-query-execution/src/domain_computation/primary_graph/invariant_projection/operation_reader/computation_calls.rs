//! What one owner call of a partitioned computation charged the reader, and
//! how a later run charges it again without making the call.
//!
//! An outcome never depends on reuse. A carried call charges the work it did
//! where the call would have charged it, and passes only when the remaining
//! work would have passed every preflight the call made. Otherwise the call
//! is made, and fails as it would have.
//!
//! Only the partitioned computation's comparator carries, observes or enters
//! a retained call: each takes the `Comparator` only that module can name a
//! value of.

use std::sync::Arc;

use worth_relational::facade::identity::EntityId;

use super::decision_reads::DecisionReadOutcome;
use super::WorthQueryApplicationOperationInvariantProjectionReader;
use crate::domain_computation::primary_graph::application_attempt::{
    observe_admitted_fact, ComputationRead, ObservedRetained, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationFactKey,
};
use crate::domain_computation::primary_graph::application_contribution::Comparator;
use crate::domain_computation::primary_graph::invariant_projection::work::WorthQueryInvariantProjectionWorkBudget;

mod ranges;
mod sequence;
pub(in crate::domain_computation::primary_graph) use sequence::{
    CallSequenceDenial, CallShape, CallShapeTransition, CallSummary, CallVisit,
    ComputationCallSequence,
};

/// The work one owner call charged and the entities it reached.
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct ComputationCallCharge {
    accounting: Option<ComputationCallAccounting>,
    reached: Arc<[EntityId]>,
}

#[derive(Clone, Copy, Debug)]
struct ComputationCallAccounting {
    /// The remaining work the call needed at its start to pass every
    /// preflight it made.
    demand: usize,
    /// The work it consumed from the budget.
    consumed: usize,
}

impl ComputationCallCharge {
    /// The bytes retaining the charge holds beyond its inline size, or `None`
    /// when the count overflows.
    pub(in crate::domain_computation::primary_graph) fn additional_bytes(&self) -> Option<u64> {
        let bytes = self.reached.len().checked_mul(size_of::<EntityId>())?;
        u64::try_from(bytes).ok()
    }
}

/// The work the reader had charged when a computation's run began, so a run
/// abandoned before any result can be made again from its start.
#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) struct ComputationRunStart {
    work_budget: WorthQueryInvariantProjectionWorkBudget,
}

impl<Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    pub(in crate::domain_computation::primary_graph) fn computation_has_accounting(
        &self,
        _: &Comparator,
    ) -> bool {
        self.reader.execution.has_accounting()
    }
    /// Where a computation's run begins.
    pub(in crate::domain_computation::primary_graph) const fn run_start(
        &self,
        _: &Comparator,
    ) -> ComputationRunStart {
        ComputationRunStart {
            work_budget: self.reader.work_budget,
        }
    }

    /// Restores the private charge policy so the full retry has a fresh run's
    /// bill. Diagnostic observations retain reads already performed before retry.
    pub(in crate::domain_computation::primary_graph) fn restart(
        &mut self,
        _: &Comparator,
        start: ComputationRunStart,
    ) {
        self.reader.work_budget = start.work_budget;
        self.decision_facts.materialize_carried();
    }

    /// Makes one owner call, recording every fact key it reads as that call's,
    /// and what it charged. The charge is `None` only when its arithmetic
    /// fails, and then the call cannot be carried.
    pub(in crate::domain_computation::primary_graph) fn measured<Output>(
        &mut self,
        read: ComputationRead,
        call: impl FnOnce(&mut Self) -> Output,
    ) -> (Output, Option<ComputationCallCharge>) {
        let accounted = self.reader.execution.has_accounting();
        self.decision_facts.begin_fresh_call(read);
        if accounted {
            self.reader.work_budget.begin_call();
        }
        self.reader.realized_scope.begin_call();
        let output = self.attributed(read, call);
        let reached = self.reader.realized_scope.end_call();
        let accounting = if accounted {
            let Some((demand, consumed)) = self.reader.work_budget.end_call() else {
                return (output, None);
            };
            Some(ComputationCallAccounting { demand, consumed })
        } else {
            None
        };
        let charge = Some(ComputationCallCharge {
            accounting,
            reached: reached.into(),
        });
        (output, charge)
    }

    /// Charges a call an earlier run made over the same facts, as if it were
    /// made now. Returns false, charging nothing, when the call would not
    /// pass here: the caller makes it instead.
    pub(in crate::domain_computation::primary_graph) fn carry(
        &mut self,
        _: &Comparator,
        charge: &ComputationCallCharge,
    ) -> bool {
        if self.reader.execution.has_accounting() {
            let Some(accounting) = charge.accounting else {
                return false;
            };
            if !self
                .reader
                .work_budget
                .carry(accounting.demand, accounting.consumed)
            {
                return false;
            }
        }
        self.reader.realized_scope.extend(&charge.reached);
        true
    }

    pub(in crate::domain_computation::primary_graph) fn computation_changes(
        &self,
        _: &Comparator,
        prior: &crate::domain_computation::primary_graph::application_contribution::ComputationPrior,
    ) -> Option<
        crate::domain_computation::primary_graph::output_lineage::invalidation::ComputationChanges,
    > {
        prior.changes_at_snapshot(self.reader.runtime, self.reader.snapshot)
    }
    pub(in crate::domain_computation::primary_graph) fn computation_snapshot(
        &self,
        _: &Comparator,
    ) -> worth_relational::facade::snapshots::SnapshotHandle {
        self.reader.snapshot.clone()
    }

    /// The immutable installed backing which also issued this reader's decode
    /// session. Identity alone grants no source, snapshot or session custody.
    pub(in crate::domain_computation::primary_graph) fn computation_value_bindings(
        &self,
        _: &Comparator,
    ) -> worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog {
        self.reader.value_bindings.clone()
    }

    /// The fact `key` names at this projection's snapshot, comparable only
    /// with the fact retained under it. Comparing a retained fact charges
    /// nothing: a full run would not compare it. The comparator compares only
    /// a state whose summed worst-case observation the computation's declared
    /// work admits.
    pub(in crate::domain_computation::primary_graph) fn observe_retained(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        comparator: &Comparator,
        key: &WorthQueryApplicationFactKey,
    ) -> Result<ObservedRetained, WorthQueryApplicationAttemptDenial> {
        observe_admitted_fact(
            execution,
            &*self.reader.runtime,
            self.reader.snapshot,
            self.reader.layout,
            key,
        )
        .map(|observed| ObservedRetained::new(comparator, observed))
    }

    /// Enters a fact a carried call read as an ordinary admitted read of that
    /// call, so seal observes it again.
    pub(in crate::domain_computation::primary_graph) fn enter_carried(
        &mut self,
        _: &Comparator,
        key: WorthQueryApplicationFactKey,
        read: ComputationRead,
    ) {
        self.attributed(read, |reader| {
            reader
                .decision_facts
                .record(key, DecisionReadOutcome::Observed);
        });
    }
}
