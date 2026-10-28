//! Checked reader charge and exact realized-scope carriage of native ranges.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::{CheckedCallShape, NativeCallCarriage},
    application_contribution::{
        QueryRequestExecution, WorthQueryManagedComputationResourceDenial as Resource,
    },
};
use std::ops::Range;
impl<Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    pub(in crate::domain_computation::primary_graph) fn can_carry_call_range(
        &self,
        _: &Comparator,
        summary: &CallSummary,
    ) -> bool {
        if !self.reader.execution.has_accounting() {
            return true;
        }
        let Some(accounting) = summary.accounting else {
            return false;
        };
        let mut budget = self.reader.work_budget;
        budget.carry(accounting.demand, accounting.consumed)
    }
    pub(in crate::domain_computation::primary_graph) fn begin_call_ranges(
        &mut self,
        _: &Comparator,
        binding: CheckedCallShape,
    ) {
        self.decision_facts.computation.begin_ranges(binding);
    }
    pub(in crate::domain_computation::primary_graph) fn select_call_shape_transition(
        &mut self,
        _: &Comparator,
        transition: Arc<CallShapeTransition>,
    ) -> bool {
        self.decision_facts
            .computation
            .select_shape_transition(transition)
    }
    pub(in crate::domain_computation::primary_graph) fn carry_call_range(
        &mut self,
        key: &Comparator,
        sequence: &Arc<ComputationCallSequence>,
        range: Range<usize>,
        summary: &CallSummary,
        execution: &QueryRequestExecution<'_>,
    ) -> Result<bool, Resource> {
        self.carry_mapped_call_range(key, sequence, range.clone(), range, summary, execution)
    }
    pub(in crate::domain_computation::primary_graph) fn carry_mapped_call_range(
        &mut self,
        key: &Comparator,
        sequence: &Arc<ComputationCallSequence>,
        source: Range<usize>,
        target: Range<usize>,
        summary: &CallSummary,
        execution: &QueryRequestExecution<'_>,
    ) -> Result<bool, Resource> {
        if !self.can_carry_call_range(key, summary) {
            return Ok(false);
        }
        if !self
            .decision_facts
            .computation
            .can_carry_mapped_range(&source, &target)
        {
            return Ok(false);
        }
        self.reader
            .realized_scope
            .carry_range(sequence, source.clone(), execution)?;
        assert!(
            self.decision_facts
                .computation
                .carry_mapped_range(source, target),
            "scope admission retains the checked range"
        );
        if self.reader.execution.has_accounting() {
            let accounting = summary
                .accounting
                .expect("range preflight checked accounting");
            assert!(self
                .reader
                .work_budget
                .carry(accounting.demand, accounting.consumed));
        }
        crate::domain_computation::primary_graph::fact_verification_observation::computation_call_carry();
        Ok(true)
    }
    pub(in crate::domain_computation::primary_graph) fn close_call_ranges(
        &mut self,
        _: &Comparator,
        execution: &QueryRequestExecution<'_>,
    ) -> Result<Arc<NativeCallCarriage>, Resource> {
        let memory = execution
            .has_accounting()
            .then(|| {
                let bytes = self
                    .decision_facts
                    .computation
                    .range_close_bytes()
                    .ok_or(Resource::CapacityOverflow)?;
                execution.reserve(bytes)
            })
            .transpose()?;
        Ok(self.decision_facts.computation.close_ranges(memory))
    }
}
