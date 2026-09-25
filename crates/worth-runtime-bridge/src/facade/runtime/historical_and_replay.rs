use super::*;

static NEXT_SIGNAL_RUNTIME_KEY: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

pub(super) fn fresh_signal_runtime_key() -> u64 {
    NEXT_SIGNAL_RUNTIME_KEY.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

impl RuntimeBridge {
    /// Delivers the full continuity workflow for one retained route record.
    ///
    /// This is an advanced diagnostics/proof entrypoint that reconstructs,
    /// resolves, lowers, and canonicalizes continuity in one bridge job.
    ///
    /// ```no_run
    /// use worth_runtime_bridge::facade::{BridgeRouteRecord, RuntimeBridge};
    ///
    /// fn deliver_continuity(
    ///     bridge: &RuntimeBridge,
    ///     route_record: &BridgeRouteRecord,
    /// ) -> Result<(), Box<dyn std::error::Error>> {
    ///     let _result = bridge.deliver_continuity(route_record)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn deliver_continuity(
        &self,
        route_record: &BridgeRouteRecord,
    ) -> Result<crate::diagnostics::BridgeDeliveredContinuityResult, BridgeContinuityError> {
        let requests = self.plan_continuity_requests(route_record)?;
        let packet = self.plan_historical_lineage_packet(&requests)?;
        let resolved = self.resolve_lineage_continuity(&packet)?;
        let artifact = self.lower_continuity_artifact(&resolved);
        let canonical_record =
            self.canonicalize_continuity_record(route_record, &requests, &artifact);

        Ok(crate::diagnostics::BridgeDeliveredContinuityResult::new(
            artifact,
            canonical_record,
        ))
    }

    /// Resolves historical lineage material into a continuity outcome set.
    ///
    /// This remains an advanced lane because continuity is a real bridge proof
    /// workflow, but ordinary route/evaluate callers should not need to invoke
    /// it directly.
    pub fn resolve_lineage_continuity(
        &self,
        packet: &BridgeHistoricalLineagePacket,
    ) -> Result<ResolvedLineageContinuitySet, BridgeContinuityError> {
        crate::continuity::ResolvedLineageContinuitySet::from_historical_packet(packet)
    }

    /// Lowers a resolved continuity outcome set into a bridge artifact.
    pub fn lower_continuity_artifact(
        &self,
        resolved: &ResolvedLineageContinuitySet,
    ) -> BridgeContinuityArtifact {
        crate::continuity::BridgeContinuityArtifact::from_resolved(resolved)
    }

    /// Canonicalizes and records a continuity artifact for replay and diagnostics.
    pub fn canonicalize_continuity_record(
        &self,
        route_record: &BridgeRouteRecord,
        requests: &BridgeEligibleContinuityRequestSet,
        artifact: &BridgeContinuityArtifact,
    ) -> BridgeCanonicalContinuityRecord {
        let record = crate::diagnostics::BridgeCanonicalContinuityRecord::new(
            route_record.clone(),
            requests.digest(),
            artifact.continuity_resolution_digest(),
            artifact.continuity_identity().clone(),
            artifact.remapped_subscription_slice_identity().clone(),
            artifact.remapped_slices().clone(),
            std::sync::Arc::from(artifact.continuity_outcomes().to_vec()),
            *artifact.counters(),
        );
        self.diagnostics.record_continuity(record.clone());
        record
    }

    /// Replays a canonical continuity record and verifies semantic stability.
    ///
    /// Use this when certification or retained diagnostics need to prove that a
    /// continuity artifact can be reconstructed from canonical bridge records
    /// alone.
    pub fn replay_canonical_continuity_record(
        &self,
        record: &BridgeCanonicalContinuityRecord,
    ) -> Result<BridgeContinuityReplaySummary, BridgeReplayError> {
        let record = record.decode()?;
        let _continuity_replay_mismatch_counters =
            record.counters().with_continuity_replay_mismatch();
        let replay_context = BridgeErrorContext::replay(
            record.route_identity().clone(),
            record.source_snapshot().clone(),
        )
        .with_subscription_slice_identity(record.remapped_subscription_slice_identity().clone());

        let requests = self
            .plan_continuity_requests(record.route_record())
            .map_err(|error| {
                BridgeReplayError::new(
                    BridgeReplayErrorKind::ContinuityRequestMismatch,
                    format!("Bridge continuity replay failed to reconstruct the planned continuity requests: {error}"),
                )
                .with_context(replay_context.clone())
            })?;
        if requests.digest() != record.continuity_request_digest() {
            return Err(BridgeReplayError::new(
                BridgeReplayErrorKind::ContinuityRequestMismatch,
                format!(
                    "Bridge continuity replay reconstructed request digest `{}` but original digest was `{}`.",
                    requests.digest(),
                    record.continuity_request_digest()
                ),
            )
            .with_context(replay_context.clone()));
        }

        let packet = self
            .plan_historical_lineage_packet(&requests)
            .map_err(|error| {
                BridgeReplayError::new(
                    BridgeReplayErrorKind::ContinuityResolutionMismatch,
                    format!("Bridge continuity replay failed to reconstruct the historical lineage packet: {error}"),
                )
                .with_context(replay_context.clone())
            })?;
        let resolved = self.resolve_lineage_continuity(&packet).map_err(|error| {
            BridgeReplayError::new(
                BridgeReplayErrorKind::ContinuityResolutionMismatch,
                format!(
                    "Bridge continuity replay failed to resolve continuity canonically: {error}"
                ),
            )
            .with_context(replay_context.clone())
        })?;
        let artifact = self.lower_continuity_artifact(&resolved);
        if artifact.continuity_resolution_digest() != record.continuity_resolution_digest() {
            return Err(BridgeReplayError::new(
                BridgeReplayErrorKind::ContinuityResolutionMismatch,
                format!(
                    "Bridge continuity replay reconstructed resolution digest `{}` but original digest was `{}`.",
                    artifact.continuity_resolution_digest(),
                    record.continuity_resolution_digest()
                ),
            )
            .with_context(replay_context.clone()));
        }
        if artifact.continuity_identity() != record.continuity_artifact_identity()
            || artifact.remapped_subscription_slice_identity()
                != record.remapped_subscription_slice_identity()
        {
            return Err(BridgeReplayError::new(
                BridgeReplayErrorKind::ContinuityArtifactMismatch,
                format!(
                    "Bridge continuity replay reconstructed artifact `{}` / `{}` but original artifact was `{}` / `{}`.",
                    artifact.continuity_identity().as_str(),
                    artifact.remapped_subscription_slice_identity().as_str(),
                    record.continuity_artifact_identity().as_str(),
                    record.remapped_subscription_slice_identity().as_str()
                ),
            )
            .with_context(replay_context));
        }

        Ok(artifact)
    }

    pub(crate) fn new(
        policy: BridgeRuntimePolicy,
        committed_patch_source: Arc<dyn CommittedPatchSource>,
        snapshot_read_source: Arc<dyn SnapshotReadSource>,
        signal_sink: Arc<dyn InvalidationSink>,
        truth_branch_head_source: Option<Arc<dyn TruthBranchHeadSource>>,
        continuity_lineage_source: Option<Arc<dyn ContinuityLineageSource>>,
        writeback_authority: Option<Arc<dyn TruthWritebackAuthority>>,
        snapshot_reader_pool: Option<Arc<dyn SnapshotReaderPool>>,
        source_registry: AdmittedSourceRegistry,
        source_adapter: Option<Arc<dyn BridgeSourceAdapter>>,
        structural_registry: AdmittedStructuralRegistry,
        merge_registry: AdmittedMergeRegistry,
        diagnostic_sink: Option<Arc<dyn DiagnosticSink>>,
        mapping_registry: FrozenMappingRegistry,
        aspect_registry: FrozenAspectMappingRegistry,
        subscription_family_registry: FrozenSubscriptionFamilyRegistry,
        semantic_dependency_registry: crate::correspondence::AdmittedSemanticDependencyRegistry,
    ) -> Self {
        let diagnostics = BridgeDiagnosticsFacade::new(policy);
        let diagnostic_sink = diagnostic_sink.unwrap_or_else(|| Arc::new(diagnostics.clone()));
        let authoritative_source_profile = committed_patch_source.authoritative_source_profile();
        let signal_runtime_key = fresh_signal_runtime_key();
        Self {
            diagnostic_sink,
            diagnostics,
            policy,
            committed_patch_source,
            authoritative_source_profile,
            snapshot_read_source,
            snapshot_reader_pool,
            signal_sink,
            truth_branch_head_source,
            continuity_lineage_source,
            writeback_authority,
            source_registry,
            source_adapter,
            structural_registry,
            merge_registry,
            mapping_registry,
            aspect_registry,
            subscription_family_registry,
            signal_runtime_key,
            signal_runtime_custody: crate::source::BridgeSignalRuntimeCustody::new(
                signal_runtime_key,
            ),
            signal_aspect_lowering_owner: worth_signal::facade::SignalAspectLoweringOwner::fresh(),
            execution_basis_reservations: Default::default(),
            correspondence_allocations: Default::default(),
            semantic_dependency_registry,
        }
    }
}
