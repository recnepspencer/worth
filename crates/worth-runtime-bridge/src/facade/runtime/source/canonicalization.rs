use super::*;

impl RuntimeBridge {
    /// Canonicalizes and records one materialized source observation.
    ///
    /// This is the advanced bridge artifact door for a single source-backed
    /// materialized observation. Most callers should only reach for this when
    /// they need retained diagnostics or replay-safe source records.
    pub fn canonicalize_source_materialization_record(
        &self,
        contract: &AdmittedSourceContract,
        observation: &MaterializedTruthViewObservation,
    ) -> Result<SourceMaterializationRecord, BridgeDeliveryError> {
        let adapter_capability_digest = self
            .source_adapter
            .as_ref()
            .map(|adapter| Arc::<str>::from(adapter.declared_capabilities().digest()))
            .ok_or_else(|| {
                BridgeDeliveryError::new(
                    BridgeDeliveryErrorKind::SourceContractMismatch,
                    "Runtime cannot canonicalize a source materialization record without a configured source adapter.",
                )
            })?;

        if observation.planned().declaration().selector() != contract.declaration().selector() {
            let error = BridgeDeliveryError::new(
                BridgeDeliveryErrorKind::SourceContractMismatch,
                format!(
                    "Source contract `{}` does not match the materialized selector `{}`.",
                    contract.contract_identity().as_str(),
                    observation
                        .planned()
                        .declaration()
                        .selector()
                        .selector_identity()
                        .as_str()
                ),
            );
            self.record_source_failure(
                contract.declaration(),
                SourceFailureClass::TruthViewSelectionMismatch,
                error.kind(),
                error.to_string(),
            );
            return Err(error);
        }

        let record =
            SourceMaterializationRecord::new(contract, observation, adapter_capability_digest);
        self.diagnostics
            .record_source_materialization(record.clone());
        Ok(record)
    }

    /// Canonicalizes and records one materialized source packet set.
    ///
    /// ```no_run
    /// use worth_runtime_bridge::facade::{
    ///     AdmittedSourceContract, RuntimeBridge, SnapshotReadPacket,
    /// };
    ///
    /// fn canonicalize_source_batch(
    ///     bridge: &RuntimeBridge,
    ///     contract: &AdmittedSourceContract,
    ///     packets: Vec<SnapshotReadPacket>,
    ///     execution: worth_execution::ExecutionRequest<'_, '_>,
    /// ) -> Result<(), Box<dyn std::error::Error>> {
    ///     let planned = bridge.plan_source_packet_batch(contract, packets, execution)?;
    ///     let materialized = bridge.materialize_source(&planned, execution)?;
    ///     let _record = bridge.canonicalize_source_materialization_packet_set_record(&materialized)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn canonicalize_source_materialization_packet_set_record(
        &self,
        materialized_packet_set: &MaterializedTruthViewPacketSet,
    ) -> Result<SourceMaterializationRecord, BridgeDeliveryError> {
        let contract = materialized_packet_set.planned_packet_set().contract();
        let adapter_capability_digest = self
            .source_adapter
            .as_ref()
            .map(|adapter| Arc::<str>::from(adapter.declared_capabilities().digest()))
            .ok_or_else(|| {
                BridgeDeliveryError::new(
                    BridgeDeliveryErrorKind::SourceContractMismatch,
                    "Runtime cannot canonicalize a source materialization record without a configured source adapter.",
                )
            })?;

        self.validate_materialized_source_packet_set(
            materialized_packet_set.planned_packet_set(),
            materialized_packet_set,
        )?;

        let record = SourceMaterializationRecord::from_packet_set(
            contract,
            materialized_packet_set,
            adapter_capability_digest,
        );
        self.diagnostics
            .record_source_materialization(record.clone());
        Ok(record)
    }
}
