use super::*;

impl RuntimeBridge {
    /// Plans one source-backed truth-view packet.
    ///
    /// This is an advanced bridge control surface. Most callers should prefer
    /// the standard evaluation path unless they need explicit source packet
    /// planning before materialization or replay.
    pub fn plan_source_packet(
        &self,
        contract: &AdmittedSourceContract,
        read_packet: SnapshotReadPacket,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<PlannedTruthViewPacket, BridgeDeliveryError> {
        self.plan_source_packet_set(contract, read_packet, execution)
            .map(|planned| planned.first().clone())
    }

    /// Plans one source-backed packet set from a single read packet.
    ///
    /// This is the main advanced packet-planning door for source-backed reads.
    ///
    /// ```no_run
    /// use worth_runtime_bridge::facade::{
    ///     AdmittedSourceContract, RuntimeBridge, SnapshotReadPacket,
    /// };
    ///
    /// fn plan_one_source_packet(
    ///     bridge: &RuntimeBridge,
    ///     contract: &AdmittedSourceContract,
    ///     packet: SnapshotReadPacket,
    ///     execution: worth_execution::ExecutionRequest<'_, '_>,
    /// ) -> Result<(), Box<dyn std::error::Error>> {
    ///     let planned = bridge.plan_source_packet_set(contract, packet, execution)?;
    ///     let materialized = bridge.materialize_source(&planned, execution)?;
    ///     let _record = bridge.canonicalize_source_materialization_packet_set_record(&materialized)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn plan_source_packet_set(
        &self,
        contract: &AdmittedSourceContract,
        read_packet: SnapshotReadPacket,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<PlannedSourceReadPacketSet, BridgeDeliveryError> {
        self.plan_source_packet_set_from_packets(contract, vec![read_packet], execution)
    }

    /// Plans one source-backed packet set from many read packets.
    ///
    /// Use this when one advanced workflow needs to materialize a batch of
    /// truth-view reads against the same admitted source contract.
    pub fn plan_source_packet_batch(
        &self,
        contract: &AdmittedSourceContract,
        read_packets: Vec<SnapshotReadPacket>,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<PlannedSourceReadPacketSet, BridgeDeliveryError> {
        self.plan_source_packet_set_from_packets(contract, read_packets, execution)
    }

    pub(super) fn plan_source_packet_set_from_packets(
        &self,
        contract: &AdmittedSourceContract,
        read_packets: Vec<SnapshotReadPacket>,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<PlannedSourceReadPacketSet, BridgeDeliveryError> {
        let validated_declaration = ValidatedSourceDeclaration::from_contract(contract);
        let declaration = HistoricalEvaluationDeclaration::new(
            validated_declaration.declaration().selector().clone(),
            if contract
                .required_capabilities()
                .contains(BridgeSourceCapability::ReplayContinuityRead)
            {
                BridgeReplayMode::Required
            } else {
                BridgeReplayMode::Disabled
            },
            BridgeDiagnosticsTier::Standard,
            BridgeDeliveryIntent::PrepareSignalEvaluation,
        );
        let packets = read_packets
            .into_iter()
            .map(|read_packet| {
                self.plan_truth_view_packet(declaration.clone(), read_packet, execution)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(PlannedSourceReadPacketSet::new(
            contract.clone(),
            validated_declaration,
            packets,
        ))
    }
}
