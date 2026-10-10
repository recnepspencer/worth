use super::*;

impl RuntimeBridge {
    /// Replays and verifies a retained source materialization record.
    pub fn replay_source_materialization_record(
        &self,
        record: &SourceMaterializationRecord,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<SourceMaterializationRecord, BridgeReplayError> {
        let contract = self
            .source_registry
            .contract_for_identity(record.source_contract_identity())
            .ok_or_else(|| {
                BridgeReplayError::new(
                    BridgeReplayErrorKind::PlanningContractMismatch,
                    format!(
                        "Bridge source replay could not find source contract `{}` in the runtime source registry.",
                        record.source_contract_identity()
                    ),
                )
                .with_context(BridgeErrorContext::default())
            })?;

        let adapter_capability_digest = self
            .source_adapter
            .as_ref()
            .map(|adapter| adapter.declared_capabilities().digest().to_owned())
            .ok_or_else(|| {
                BridgeReplayError::new(
                    BridgeReplayErrorKind::PlanningContractMismatch,
                    "Bridge source replay requires a configured source adapter.",
                )
                .with_context(BridgeErrorContext::default())
            })?;

        if adapter_capability_digest != record.adapter_capability_digest() {
            return Err(BridgeReplayError::new(
                BridgeReplayErrorKind::PlanningContractMismatch,
                format!(
                    "Bridge source replay reconstructed adapter capabilities `{}` but original source record expected `{}`.",
                    adapter_capability_digest,
                    record.adapter_capability_digest()
                ),
            )
            .with_context(BridgeErrorContext::default()));
        }

        let observation = self
            .plan_source_packet_set_from_packets(contract, record.read_packets().to_vec(), execution)
            .and_then(|planned_packet_set| self.materialize_source(&planned_packet_set, execution))
            .map_err(|error| {
                BridgeReplayError::new(
                    BridgeReplayErrorKind::HistoricalEvaluationAuthorityMismatch,
                    format!(
                        "Bridge source replay could not materialize the planned source packet set: {error}"
                    ),
                )
                .with_context(BridgeErrorContext::default())
            })?;

        let replayed = SourceMaterializationRecord::from_packet_set(
            contract,
            &observation,
            Arc::<str>::from(adapter_capability_digest),
        );

        if replayed != *record {
            return Err(BridgeReplayError::new(
                BridgeReplayErrorKind::HistoricalEvaluationAuthorityMismatch,
                format!(
                    "Bridge source replay reconstructed record `{}` but original source record was `{}`.",
                    replayed.record_identity().as_str(),
                    record.record_identity().as_str()
                ),
            )
            .with_context(BridgeErrorContext::default()));
        }

        Ok(replayed)
    }
}
