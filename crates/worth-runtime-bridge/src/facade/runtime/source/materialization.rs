use super::*;

impl RuntimeBridge {
    /// Materializes a planned source packet set through the configured source adapter.
    ///
    /// This is the main advanced source-materialization door; validation and
    /// replay helpers stay specialist.
    pub fn materialize_source(
        &self,
        planned_packet_set: &PlannedSourceReadPacketSet,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<MaterializedTruthViewPacketSet, BridgeDeliveryError> {
        let adapter = self.source_adapter.as_ref().ok_or_else(|| {
            let error = BridgeDeliveryError::new(
                BridgeDeliveryErrorKind::SourceContractMismatch,
                "Runtime cannot materialize a source packet set without a configured source adapter.",
            );
            self.record_source_failure(
                planned_packet_set.contract().declaration(),
                SourceFailureClass::SourceMaterializationRejected,
                error.kind(),
                error.to_string(),
            );
            error
        })?;

        adapter
            .materialize_packets(planned_packet_set, execution)
            .and_then(|materialized| {
                self.validate_materialized_source_packet_set(planned_packet_set, &materialized)?;
                Ok(materialized)
            })
            .inspect_err(|delivery_error| {
                for planned in planned_packet_set.packets() {
                    self.record_historical_evaluation_failure(
                        planned.declaration(),
                        crate::historical::failures::historical_failure_class_for_delivery_error(
                            delivery_error,
                        ),
                        delivery_error.to_string(),
                        crate::historical::failures::historical_failure_counters_for_delivery_error(
                            planned.declaration(),
                            delivery_error,
                        ),
                    );
                }
                self.record_source_failure(
                    planned_packet_set.contract().declaration(),
                    validation::source_failure_class_for_materialization_error(
                        delivery_error.kind(),
                    ),
                    delivery_error.kind(),
                    delivery_error.to_string(),
                );
            })
    }

    /// Materializes one source-backed truth-view observation.
    pub fn materialize_source_packet(
        &self,
        contract: &AdmittedSourceContract,
        read_packet: SnapshotReadPacket,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<MaterializedTruthViewObservation, BridgeDeliveryError> {
        let planned_packet_set = self.plan_source_packet_set(contract, read_packet, execution)?;
        let adapter = self.source_adapter.as_ref().ok_or_else(|| {
            let error = BridgeDeliveryError::new(
                BridgeDeliveryErrorKind::SourceContractMismatch,
                "Runtime cannot materialize a source packet without a configured source adapter.",
            );
            self.record_source_failure(
                contract.declaration(),
                SourceFailureClass::SourceMaterializationRejected,
                error.kind(),
                error.to_string(),
            );
            error
        })?;
        let planned = planned_packet_set.first().clone();
        adapter
            .materialize_packet(planned.clone(), execution)
            .and_then(|materialized| {
                self.validate_materialized_source_observation(&planned, materialized)
            })
            .inspect_err(|delivery_error| {
                self.record_historical_evaluation_failure(
                    planned.declaration(),
                    crate::historical::failures::historical_failure_class_for_delivery_error(
                        delivery_error,
                    ),
                    delivery_error.to_string(),
                    crate::historical::failures::historical_failure_counters_for_delivery_error(
                        planned.declaration(),
                        delivery_error,
                    ),
                );
                self.record_source_failure(
                    contract.declaration(),
                    validation::source_failure_class_for_materialization_error(
                        delivery_error.kind(),
                    ),
                    delivery_error.kind(),
                    delivery_error.to_string(),
                );
            })
    }

    /// Materializes a batch of source-backed truth-view observations.
    ///
    /// ```no_run
    /// use worth_runtime_bridge::facade::{
    ///     AdmittedSourceContract, RuntimeBridge, SnapshotReadPacket,
    /// };
    ///
    /// fn materialize_source_batch(
    ///     bridge: &RuntimeBridge,
    ///     contract: &AdmittedSourceContract,
    ///     packets: Vec<SnapshotReadPacket>,
    ///     execution: worth_execution::ExecutionRequest<'_, '_>,
    /// ) -> Result<(), Box<dyn std::error::Error>> {
    ///     let packet_set = bridge.materialize_source_packet_batch(contract, packets, execution)?;
    ///     let _record = bridge.canonicalize_source_materialization_packet_set_record(&packet_set)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn materialize_source_packet_batch(
        &self,
        contract: &AdmittedSourceContract,
        read_packets: Vec<SnapshotReadPacket>,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<MaterializedTruthViewPacketSet, BridgeDeliveryError> {
        let planned_packet_set =
            self.plan_source_packet_batch(contract, read_packets, execution)?;
        self.materialize_source(&planned_packet_set, execution)
    }
}
