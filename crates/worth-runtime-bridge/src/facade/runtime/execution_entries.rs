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
        self.route_with_scope(
            request.into(),
            crate::routing::scope::RouteScope::begin(),
            execution,
        )
    }

    /// Routes and delivers one authoritative change under the admitted route policy.
    pub fn route_with_policy(
        &self,
        request: impl Into<BridgeRouteRequest>,
        route_policy: &BridgeRoutePlanningPolicy,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeRoute, BridgeStandardRouteError> {
        self.ensure_route_planning_policy_coherent(route_policy)?;
        if route_policy.execution_class() != BridgeExecutionPolicyClass::DeterministicCanonical {
            return Err(BridgeRouteError::new(
                BridgeRouteErrorKind::RoutePolicyMismatch,
                "The authoritative standard route requires deterministic canonical execution.",
            )
            .into());
        }
        self.route_with_scope(
            request.into(),
            crate::routing::scope::RouteScope::begin()
                .with_route_planning_policy(route_policy.clone()),
            execution,
        )
    }

    fn route_with_scope(
        &self,
        request: BridgeRouteRequest,
        scope: crate::routing::scope::RouteScope,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeRoute, BridgeStandardRouteError> {
        let ingested = crate::input::source::ingest_committed_patch(self, request, execution)?;
        let planned = crate::routing::planning::plan_ingested_patch(
            self,
            ingested.with_route_scope(
                scope.for_standard_delivery(self.policy.record_route_artifacts()),
            ),
        )?;
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
