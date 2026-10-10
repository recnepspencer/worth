use super::*;

impl RuntimeBridge {
    /// Routes one authoritative truth change through the standard path.
    ///
    /// This is the everyday front door for:
    ///
    /// - ingesting committed truth change
    /// - planning invalidation
    /// - delivering invalidation to the bound compute sink
    ///
    /// Prefer this over the lower-level ingest/plan/deliver sequence unless the
    /// job explicitly needs advanced control.
    pub fn route(
        &self,
        request: impl Into<BridgeRouteRequest>,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeRoute, BridgeStandardRouteError> {
        let planned = self.plan_committed_patch(request.into(), execution)?;
        let result = self.deliver_invalidation(planned.clone(), execution)?;
        Ok(BridgeRoute::new(planned, result))
    }

    /// Evaluates the current bridge-visible result for a routed target.
    ///
    /// This is the standard answer to "what should the compute side see now for
    /// the thing that was just routed?"
    pub fn evaluate_current(
        &self,
        target: BridgeEvaluationTarget,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeSignalEvaluationRequest, BridgeDeliveryError> {
        self.prepare_signal_evaluation(target.into_planned_route(), execution)
    }

    /// Evaluates an explicit truth view.
    ///
    /// Use this when branch head, branch snapshot, or historical commit basis
    /// is part of the job rather than an internal detail.
    pub fn evaluate(
        &self,
        request: BridgeTruthViewEvaluationRequest,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeTruthViewEvaluation, BridgeDeliveryError> {
        let planned =
            self.plan_truth_view_packet(request.declaration(), request.read_packet(), execution)?;
        let observation = self.materialize_truth_view_observation(planned, execution)?;
        let canonical_record = self.canonicalize_historical_evaluation_record(&observation);
        Ok(BridgeTruthViewEvaluation::new(
            observation,
            canonical_record,
        ))
    }
}
