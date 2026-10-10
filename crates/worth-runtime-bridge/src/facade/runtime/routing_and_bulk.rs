use super::*;

impl RuntimeBridge {
    /// Creates a new runtime bridge builder.
    pub fn builder() -> RuntimeBridgeBuilder {
        RuntimeBridgeBuilder::new()
    }

    /// Returns the runtime policy frozen into this bridge instance.
    ///
    /// Reach for this when you need to explain or verify runtime-wide replay,
    /// diagnostics, or execution guarantees.
    pub fn policy(&self) -> &BridgeRuntimePolicy {
        &self.policy
    }

    /// Specialist ingress step that turns a route request into a committed-patch envelope.
    pub fn ingest_committed_patch(
        &self,
        request: BridgeRouteRequest,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, BridgeRouteError> {
        Ok(
            crate::input::source::ingest_committed_patch(self, request, execution)?
                .envelope()
                .clone(),
        )
    }

    /// Plans one already-ingested committed-patch envelope with default mapping context.
    pub fn plan_envelope(
        &self,
        envelope: BridgeCommittedPatchEnvelope,
    ) -> Result<BridgePlannedRoute, BridgeRouteError> {
        self.plan_envelope_with_mapping_context(envelope, BridgeMappingContext::default())
    }

    /// Plans one already-ingested committed-patch envelope with explicit mapping context.
    pub fn plan_envelope_with_mapping_context(
        &self,
        envelope: BridgeCommittedPatchEnvelope,
        mapping_context: BridgeMappingContext,
    ) -> Result<BridgePlannedRoute, BridgeRouteError> {
        crate::routing::planning::plan_ingested_patch(
            self,
            crate::routing::IngestedBridgePatch::new(
                envelope,
                mapping_context,
                crate::routing::scope::RouteScope::begin(),
            ),
        )
    }

    /// Plans one already-ingested envelope under explicit mapping context and route policy.
    pub fn plan_envelope_with_mapping_context_and_route_policy(
        &self,
        envelope: BridgeCommittedPatchEnvelope,
        mapping_context: BridgeMappingContext,
        route_policy: &BridgeRoutePlanningPolicy,
    ) -> Result<BridgePlannedRoute, BridgeRouteError> {
        self.ensure_route_planning_policy_coherent(route_policy)?;
        crate::routing::planning::plan_ingested_patch(
            self,
            crate::routing::IngestedBridgePatch::new(
                envelope,
                mapping_context,
                crate::routing::scope::RouteScope::begin()
                    .with_route_planning_policy(route_policy.clone()),
            ),
        )
    }

    /// Plans one committed patch through the routing substrate without delivering it.
    ///
    /// Prefer [`RuntimeBridge::route`] for ordinary work. This is the advanced
    /// door for callers that need to inspect or stage the planned route.
    pub fn plan_committed_patch(
        &self,
        request: BridgeRouteRequest,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgePlannedRoute, BridgeRouteError> {
        self.plan_committed_patch_with_mapping_context(
            request,
            BridgeMappingContext::default(),
            execution,
        )
    }

    /// Plans one committed patch with explicit mapping context.
    pub fn plan_committed_patch_with_mapping_context(
        &self,
        request: BridgeRouteRequest,
        mapping_context: BridgeMappingContext,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgePlannedRoute, BridgeRouteError> {
        let ingested = crate::input::source::ingest_committed_patch(self, request, execution)?;
        crate::routing::planning::plan_ingested_patch(
            self,
            ingested.with_mapping_context(mapping_context),
        )
    }

    /// Plans one committed patch with an explicit route policy.
    pub fn plan_committed_patch_with_route_policy(
        &self,
        request: BridgeRouteRequest,
        route_policy: &BridgeRoutePlanningPolicy,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgePlannedRoute, BridgeRouteError> {
        self.plan_committed_patch_with_mapping_context_and_route_policy(
            request,
            BridgeMappingContext::default(),
            route_policy,
            execution,
        )
    }

    /// Plans one committed patch with both explicit mapping context and route policy.
    pub fn plan_committed_patch_with_mapping_context_and_route_policy(
        &self,
        request: BridgeRouteRequest,
        mapping_context: BridgeMappingContext,
        route_policy: &BridgeRoutePlanningPolicy,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgePlannedRoute, BridgeRouteError> {
        self.ensure_route_planning_policy_coherent(route_policy)?;
        let ingested = crate::input::source::ingest_committed_patch(self, request, execution)?;
        crate::routing::planning::plan_ingested_patch(
            self,
            ingested
                .with_mapping_context(mapping_context)
                .with_route_scope(
                    crate::routing::scope::RouteScope::begin()
                        .with_route_planning_policy(route_policy.clone()),
                ),
        )
    }

    pub(crate) fn plan_committed_patch_with_mapping_context_and_route_policy_digest_for_replay(
        &self,
        request: BridgeRouteRequest,
        mapping_context: BridgeMappingContext,
        route_policy_digest: &str,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgePlannedRoute, BridgeRouteError> {
        let ingested = crate::input::source::ingest_committed_patch(self, request, execution)?;
        crate::routing::planning::plan_ingested_patch(
            self,
            ingested
                .with_mapping_context(mapping_context)
                .with_route_scope(
                    crate::routing::scope::RouteScope::begin()
                        .with_route_planning_policy_digest(route_policy_digest.to_owned()),
                ),
        )
    }

    pub(crate) fn plan_committed_patch_with_mapping_context_and_route_policy_for_replay(
        &self,
        request: BridgeRouteRequest,
        mapping_context: BridgeMappingContext,
        route_policy: &BridgeRoutePlanningPolicy,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgePlannedRoute, BridgeRouteError> {
        let ingested = crate::input::source::ingest_committed_patch(self, request, execution)?;
        crate::routing::planning::plan_ingested_patch(
            self,
            ingested
                .with_mapping_context(mapping_context)
                .with_route_scope(
                    crate::routing::scope::RouteScope::begin()
                        .with_route_planning_policy(route_policy.clone()),
                ),
        )
    }

    /// Plans a bulk bridge workload under the runtime's default route policy.
    ///
    /// This is an advanced execution-planning surface used by bulk and
    /// certification workflows.
    pub fn plan_bulk_workload(
        &self,
        request: BridgeBulkWorkloadRequest,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeBulkWorkloadPlan, BridgeRouteError> {
        crate::routing::planning::plan_bulk_workload(self, request, execution)
    }

    /// Plans a bulk bridge workload under an explicit route policy.
    pub fn plan_bulk_workload_with_route_policy(
        &self,
        request: BridgeBulkWorkloadRequest,
        route_policy: &BridgeRoutePlanningPolicy,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeBulkWorkloadPlan, BridgeRouteError> {
        self.ensure_route_planning_policy_coherent(route_policy)?;
        crate::routing::planning::plan_bulk_workload_with_route_policy(
            self,
            request,
            route_policy,
            execution,
        )
    }

    /// Canonicalizes and records a bulk workload plan for replay and diagnostics.
    pub fn canonicalize_bulk_workload_plan(
        &self,
        plan: &BridgeBulkWorkloadPlan,
    ) -> BridgeCanonicalBulkPlanRecord {
        let record = crate::routing::BridgeCanonicalBulkPlanRecord::from_bulk_workload_plan(plan);
        self.diagnostics.record_bulk(record.clone());
        record
    }

    /// Delivers one previously planned route to the configured compute sink.
    pub fn deliver_invalidation(
        &self,
        route: BridgePlannedRoute,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeRouteResult, BridgeDeliveryError> {
        crate::delivery::deliver_planned_route(self, route, execution)
    }

    /// Prepares a planned route for later delivery.
    pub fn prepare_delivery(&self, route: BridgePlannedRoute) -> BridgePreparedDeliveryRequest {
        crate::delivery::prepare_planned_route_for_delivery(route)
    }

    /// Delivers a previously prepared route.
    pub fn deliver_prepared(
        &self,
        prepared: BridgePreparedDeliveryRequest,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeRouteResult, BridgeDeliveryError> {
        crate::delivery::deliver_prepared_route(self, prepared, execution)
    }

    /// Delivers a previously planned bulk workload.
    pub fn deliver_bulk_workload_plan(
        &self,
        plan: BridgeBulkWorkloadPlan,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeBulkWorkloadResult, BridgeDeliveryError> {
        crate::delivery::deliver_bulk_workload_plan(self, plan, execution)
    }

    /// Prepares a signal evaluation request from a planned route.
    pub fn prepare_signal_evaluation(
        &self,
        route: BridgePlannedRoute,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeSignalEvaluationRequest, BridgeDeliveryError> {
        crate::delivery::prepare_signal_evaluation(self, route, execution)
    }

    /// Replays and verifies a canonical route record.
    pub fn replay_canonical_record(
        &self,
        record: &BridgeCanonicalRouteRecord,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeReplaySummary, BridgeReplayError> {
        let route_record = record.decode()?;
        crate::routing::replay_route_record(self, &route_record, execution)
    }

    /// Replays and verifies a canonical bulk workload record.
    pub fn replay_canonical_bulk_plan_record(
        &self,
        record: &BridgeCanonicalBulkPlanRecord,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeBulkWorkloadPlan, BridgeReplayError> {
        if !self.policy.allow_replay_artifacts() {
            return Err(BridgeReplayError::new(
                BridgeReplayErrorKind::ReplayArtifactsDisabled,
                "Bridge replay artifacts are disabled by runtime policy.",
            ));
        }

        let record = record.decode()?;
        let replayed =
            self.plan_bulk_workload(record.request().clone(), execution)
                .map_err(|error| {
                    BridgeReplayError::new(
                BridgeReplayErrorKind::BulkPlanReplayMismatch,
                format!("Bridge bulk replay failed to reconstruct the planned workload: {error}"),
            )
                })?;

        if replayed.workload_identity() != record.workload_identity()
            || replayed.canonical_request().digest() != record.canonical_request_digest()
            || replayed.normalized_summary().digest() != record.normalized_summary_digest()
            || replayed.canonical_planning_identity() != record.canonical_planning_identity()
            || replayed.admission_profile_identity() != record.admission_profile_identity()
            || replayed.packet_set().digest() != record.packet_set_digest()
            || replayed.execution_plan().digest() != record.execution_plan_digest()
            || replayed.execution_plan().reduced_artifact().digest()
                != record.reduced_artifact_digest()
        {
            return Err(BridgeReplayError::new(
                BridgeReplayErrorKind::BulkPlanReplayMismatch,
                format!(
                    "Bridge bulk replay reconstructed workload `{}` / plan `{}` / execution `{}` but the canonical record expected `{}` / `{}` / `{}`.",
                    replayed.workload_identity().as_str(),
                    replayed.canonical_planning_identity().as_str(),
                    replayed.execution_plan().digest(),
                    record.workload_identity().as_str(),
                    record.canonical_planning_identity().as_str(),
                    record.execution_plan_digest()
                ),
            ));
        }

        Ok(replayed)
    }

    /// Opens the standard diagnostics door for this bridge.
    ///
    /// The returned wrapper keeps the everyday, job-shaped helpers in front
    /// while still exposing retained diagnostic artifacts when needed.
    pub fn diagnostics(&self) -> BridgeDiagnostics<'_> {
        BridgeDiagnostics::new(&self.diagnostics)
    }

    /// Projects a route planning policy from a lowered execution policy.
    pub fn project_route_planning_policy(
        &self,
        lowered: &LoweredBridgeExecutionPolicy,
    ) -> Result<BridgeRoutePlanningPolicy, BridgeRouteError> {
        let route_policy = lowered.route_planning_policy();
        self.ensure_route_planning_policy_coherent(&route_policy)?;
        Ok(route_policy)
    }
}

impl RuntimeBridge {
    fn ensure_route_planning_policy_coherent(
        &self,
        route_policy: &BridgeRoutePlanningPolicy,
    ) -> Result<(), BridgeRouteError> {
        if route_policy.diagnostics_tier() > self.policy.diagnostics_tier() {
            return Err(BridgeRouteError::new(
                BridgeRouteErrorKind::RoutePolicyMismatch,
                format!(
                    "Route planning policy `{}` requires diagnostics tier `{:?}` but runtime baseline admits only `{:?}`.",
                    route_policy.digest(),
                    route_policy.diagnostics_tier(),
                    self.policy.diagnostics_tier()
                ),
            ));
        }
        if route_policy.route_artifacts() && !self.policy.record_route_artifacts() {
            return Err(BridgeRouteError::new(
                BridgeRouteErrorKind::RoutePolicyMismatch,
                format!(
                    "Route planning policy `{}` requires route artifacts but runtime baseline disables route artifact retention.",
                    route_policy.digest()
                ),
            ));
        }
        if route_policy.replay_artifacts() && !self.policy.allow_replay_artifacts() {
            return Err(BridgeRouteError::new(
                BridgeRouteErrorKind::RoutePolicyMismatch,
                format!(
                    "Route planning policy `{}` requires replay artifacts but runtime baseline disables replay retention.",
                    route_policy.digest()
                ),
            ));
        }
        Ok(())
    }
}
