//! Sole work and budget accounting owner for aggregate projection.

use super::super::{
    denial, WorthQueryInvariantAggregateDenial, WorthQueryInvariantAggregateDenialKind,
};
use crate::domain_computation::primary_graph::invariant_projection::work::WorthQueryInvariantProjectionWorkBudget;
use crate::domain_computation::primary_graph::invariant_projection::WorthQueryInvariantProjectionWork;

pub struct AggregateWorkAccounting<'work> {
    work: &'work mut WorthQueryInvariantProjectionWork,
    budget: &'work mut WorthQueryInvariantProjectionWorkBudget,
    cold_lookup_recorded: bool,
}

impl<'work> AggregateWorkAccounting<'work> {
    pub fn new(
        work: &'work mut WorthQueryInvariantProjectionWork,
        budget: &'work mut WorthQueryInvariantProjectionWorkBudget,
    ) -> Self {
        Self {
            work,
            budget,
            cold_lookup_recorded: false,
        }
    }

    pub fn admit_cache_lookup(
        &mut self,
        member: &str,
    ) -> Result<(), WorthQueryInvariantAggregateDenial> {
        self.admit(1, member)
    }

    pub fn complete_warm_lookup(&mut self) {
        self.work.record_aggregate_lookup(true, 0);
    }

    pub fn record_cold_lookup(&mut self, rebuild_rows: usize) {
        assert!(
            !self.cold_lookup_recorded,
            "cold aggregate lookup recorded twice"
        );
        self.cold_lookup_recorded = true;
        self.work.record_aggregate_lookup(false, rebuild_rows);
    }

    pub const fn remaining(&self) -> Option<usize> {
        self.budget.remaining()
    }

    pub fn reject_initial_adjacency(
        &mut self,
        limit: worth_relational::facade::runtime::AdjacencyTruthReadLimitExceeded,
        member: &str,
    ) -> WorthQueryInvariantAggregateDenial {
        if self.budget.consume(limit.consumed_work_units()) {
            self.record_cold_lookup(limit.relation_records_examined());
            self.work
                .record_adjacencies(1, limit.relation_records_examined());
        }
        self.budget.mark_exceeded();
        denial(
            WorthQueryInvariantAggregateDenialKind::WorkBudgetExceeded,
            member,
        )
    }

    pub fn reject_bounded_adjacency(
        &mut self,
        limit: worth_relational::facade::runtime::AdjacencyTruthReadLimitExceeded,
        member: &str,
    ) -> WorthQueryInvariantAggregateDenial {
        if self.budget.consume(limit.consumed_work_units()) {
            self.work
                .record_adjacencies(1, limit.relation_records_examined());
        }
        self.budget.mark_exceeded();
        denial(
            WorthQueryInvariantAggregateDenialKind::WorkBudgetExceeded,
            member,
        )
    }

    pub fn complete_adjacency(
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
        self.work.record_adjacencies(1, examined);
        Ok(())
    }

    pub fn admit_source_field(
        &mut self,
        member: &str,
    ) -> Result<(), WorthQueryInvariantAggregateDenial> {
        self.admit(1, member)
    }

    pub fn record_source_field(&mut self) {
        self.work.record_fields(1);
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
