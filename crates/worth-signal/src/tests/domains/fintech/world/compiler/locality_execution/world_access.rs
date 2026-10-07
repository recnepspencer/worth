use crate::data::error::SignalError;
use crate::tests::domains::fintech::world::{
    FinancialLocalityDefinition, FinancialPerformedCanonicalWork, LocalitySemanticOutputId,
};

use super::{FinancialLocalityRedObservation, FinancialRestoreLifecycleEvidence};

impl super::super::CompiledFinancialWorld {
    pub(in crate::tests::domains::fintech) fn baseline_preparation_report(
        &self,
    ) -> worth_foundational::ExecutionReport {
        self.locality()
            .baseline_preparation_report
            .expect("completed baseline retains its request's charged-work report")
    }

    pub(in crate::tests::domains::fintech) fn locality_definition(
        &self,
    ) -> &FinancialLocalityDefinition {
        self.locality().locality_definition()
    }

    pub(in crate::tests::domains::fintech) fn begin_operation_observation(
        &mut self,
    ) -> Result<crate::facade::SignalObservationSession, SignalError> {
        self.locality_mut().begin_runtime_observation()
    }

    pub(in crate::tests::domains::fintech) fn finish_operation_observation(
        &self,
        session: crate::facade::SignalObservationSession,
    ) -> Result<crate::data::proof::SignalInvalidationExecutionReceipt, SignalError> {
        self.locality().finish_runtime_observation(&session)
    }

    pub(in crate::tests::domains::fintech) fn locality_graph_instance(&self) -> u64 {
        self.locality().graph_instance()
    }

    pub(in crate::tests::domains::fintech) fn locality_operational_digest_with_work(
        &self,
        performed_work: &FinancialPerformedCanonicalWork,
    ) -> Result<worth_foundational::facade::CanonicalDigestId, SignalError> {
        self.locality().operational_digest_with_work(performed_work)
    }

    pub(in crate::tests::domains::fintech) fn run_inherited_breadth_red_control(
        &mut self,
    ) -> Result<FinancialLocalityRedObservation, SignalError> {
        self.locality_mut().run_inherited_breadth_red_control()
    }

    pub(in crate::tests::domains::fintech) fn run_locality_action_trace(
        &mut self,
        trace_index: usize,
    ) -> Result<FinancialLocalityRedObservation, SignalError> {
        self.locality_mut().run_action_trace(trace_index)
    }

    pub(in crate::tests::domains::fintech) fn run_locality_action_trace_with_workers(
        &mut self,
        trace_index: usize,
        workers: usize,
    ) -> Result<FinancialLocalityRedObservation, SignalError> {
        self.locality_mut()
            .run_action_trace_with_workers(trace_index, workers)
    }

    pub(in crate::tests::domains::fintech) fn observe_locality_action_trace_with_workers(
        &mut self,
        trace_index: usize,
        workers: usize,
    ) -> Result<
        (
            FinancialLocalityRedObservation,
            crate::data::proof::SignalInvalidationExecutionReceipt,
        ),
        SignalError,
    > {
        let token = self
            .locality_mut()
            .runtime
            .begin_invalidation_execution_observation()
            .map_err(|denial| SignalError::invalid_input(denial.to_string()))?;
        let observation = self.run_locality_action_trace_with_workers(trace_index, workers)?;
        let receipt = self
            .locality()
            .runtime
            .finish_invalidation_execution_observation(&token)?;
        Ok((observation, receipt))
    }

    pub(in crate::tests::domains::fintech) fn set_locality_diagnostics_tier(
        &mut self,
        tier: crate::facade::DiagnosticsTier,
    ) {
        self.locality_mut()
            .runtime
            .graph_mut()
            .reset_runtime_policy_to_tier(tier);
    }

    pub(in crate::tests::domains::fintech) fn locality_retained_fact_counts(
        &self,
    ) -> (usize, usize) {
        let observer = self.locality().runtime.graph().observe();
        self.locality()
            .handles
            .values()
            .fold((0, 0), |(explanations, provenance), node| {
                (
                    explanations + usize::from(observer.explanation_fact(*node).is_some()),
                    provenance + usize::from(observer.provenance_fact(*node).is_some()),
                )
            })
    }

    pub(in crate::tests::domains::fintech) fn certify_restore_locality_lifecycle(
        &mut self,
    ) -> Result<FinancialRestoreLifecycleEvidence, SignalError> {
        self.locality_mut().certify_restore_lifecycle()
    }

    pub(in crate::tests::domains::fintech) fn committed_locality_financial_values(
        &self,
    ) -> Result<std::collections::BTreeMap<LocalitySemanticOutputId, i64>, SignalError> {
        self.locality().committed_financial_values()
    }
}
