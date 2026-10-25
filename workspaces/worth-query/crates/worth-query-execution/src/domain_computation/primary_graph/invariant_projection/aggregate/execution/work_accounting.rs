//! Sole work and budget accounting owner for aggregate projection.

use super::super::{
    denial, WorthQueryInvariantAggregateDenial, WorthQueryInvariantAggregateDenialKind,
};
use crate::domain_computation::primary_graph::invariant_projection::work::WorthQueryInvariantProjectionWorkBudget;
use crate::domain_computation::primary_graph::invariant_projection::WorthQueryInvariantProjectionWork;

pub(in super::super) struct AggregateWorkAccounting<'work> {
    work: &'work mut WorthQueryInvariantProjectionWork,
    budget: &'work mut WorthQueryInvariantProjectionWorkBudget,
    cold_lookup_recorded: bool,
}

impl<'work> AggregateWorkAccounting<'work> {
    pub(super) fn new(
        work: &'work mut WorthQueryInvariantProjectionWork,
        budget: &'work mut WorthQueryInvariantProjectionWorkBudget,
    ) -> Self {
        Self {
            work,
            budget,
            cold_lookup_recorded: false,
        }
    }

    pub(in super::super) fn admit_cache_lookup(
        &mut self,
        member: &str,
    ) -> Result<(), WorthQueryInvariantAggregateDenial> {
        self.admit(1, member)
    }

    pub(in super::super) fn complete_warm_lookup(&mut self) {
        self.work.record_aggregate_lookup(true, 0);
    }

    pub(in super::super) fn record_cold_lookup(&mut self, rebuild_rows: usize) {
        assert!(
            !self.cold_lookup_recorded,
            "cold aggregate lookup recorded twice"
        );
        self.cold_lookup_recorded = true;
        self.work.record_aggregate_lookup(false, rebuild_rows);
    }

    pub(in super::super) const fn remaining(&self) -> Option<usize> {
        self.budget.remaining()
    }

    pub(in super::super) fn reject_initial_adjacency(
        &mut self,
        limit: worth_relational::facade::runtime::AdjacencyTruthReadLimitExceeded,
        member: &str,
    ) -> WorthQueryInvariantAggregateDenial {
        if self.budget.consume(limit.consumed_work_units()) {
            self.record_cold_lookup(limit.relation_records_examined());
            self.work
                .record_adjacency(limit.relation_records_examined());
        }
        self.budget.mark_exceeded();
        denial(
            WorthQueryInvariantAggregateDenialKind::WorkBudgetExceeded,
            member,
        )
    }

    pub(in super::super) fn reject_bounded_adjacency(
        &mut self,
        limit: worth_relational::facade::runtime::AdjacencyTruthReadLimitExceeded,
        member: &str,
    ) -> WorthQueryInvariantAggregateDenial {
        if self.budget.consume(limit.consumed_work_units()) {
            self.work
                .record_adjacency(limit.relation_records_examined());
        }
        self.budget.mark_exceeded();
        denial(
            WorthQueryInvariantAggregateDenialKind::WorkBudgetExceeded,
            member,
        )
    }

    pub(in super::super) fn complete_adjacency(
        &mut self,
        consumed: usize,
        examined: usize,
        member: &str,
    ) -> Result<(), WorthQueryInvariantAggregateDenial> {
        if !self.budget.consume(consumed) {
            return Err(denial(
                WorthQueryInvariantAggregateDenialKind::WorkBudgetExceeded,
                member,
            ));
        }
        self.work.record_adjacency(examined);
        Ok(())
    }

    pub(in super::super) fn admit_source_field(
        &mut self,
        member: &str,
    ) -> Result<(), WorthQueryInvariantAggregateDenial> {
        self.admit(1, member)?;
        self.work.record_field();
        Ok(())
    }

    fn admit(
        &mut self,
        amount: usize,
        member: &str,
    ) -> Result<(), WorthQueryInvariantAggregateDenial> {
        if !self.budget.can_afford(amount) {
            return Err(denial(
                WorthQueryInvariantAggregateDenialKind::WorkBudgetExceeded,
                member,
            ));
        }
        assert!(self.budget.consume(amount), "projection work was admitted");
        Ok(())
    }
}
