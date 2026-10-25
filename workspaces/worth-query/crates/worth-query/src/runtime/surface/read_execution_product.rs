use crate::runtime::{
    WorthQueryAdmittedGraphReadAccessPlan, WorthQueryGraphReadAccessComplexityCounters,
    WorthQueryGraphReadAccessPlanConsumption, WorthQueryGraphReadAccessReceiptSummary,
    WorthQueryIntentDecisionTraceEnvelope, WorthQueryIntentExecutionProvenance,
};

use super::WorthQueryReadReceipt;

pub(in crate::runtime) trait WorthQueryReadExecutionProduct {
    fn receipt(&self) -> &WorthQueryReadReceipt;
    fn receipt_mut(&mut self) -> &mut WorthQueryReadReceipt;

    fn attach_intent_admission_evidence(
        &mut self,
        decision_trace_envelope: WorthQueryIntentDecisionTraceEnvelope,
        execution_provenance: WorthQueryIntentExecutionProvenance,
    ) {
        self.receipt_mut().decision_trace_envelope = Some(decision_trace_envelope);
        self.receipt_mut().execution_provenance = Some(execution_provenance);
    }

    fn attach_graph_read_access_plan(
        &mut self,
        plan: WorthQueryAdmittedGraphReadAccessPlan,
        plan_consumption: WorthQueryGraphReadAccessPlanConsumption,
    ) {
        let graph_read_access_summary =
            WorthQueryGraphReadAccessReceiptSummary::from_execution_parts(
                self.receipt().read_graph_digest(),
                &plan,
                &plan_consumption,
            );
        let graph_read_access_complexity_counters =
            WorthQueryGraphReadAccessComplexityCounters::from_execution_parts(
                &plan,
                &plan_consumption,
            );
        let receipt = self.receipt_mut();
        receipt.graph_read_access_plan = Some(plan);
        receipt.graph_read_access_plan_consumption = Some(plan_consumption);
        receipt.graph_read_access_summary = Some(graph_read_access_summary);
        receipt.graph_read_access_complexity_counters = Some(graph_read_access_complexity_counters);
    }
}
