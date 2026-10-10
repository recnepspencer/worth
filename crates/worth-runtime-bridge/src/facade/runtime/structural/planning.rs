use super::*;

impl RuntimeBridge {
    /// Plans a structural match packet set from already-classified candidates.
    pub fn plan_structural_match_packet_set(
        &self,
        contract: &AdmittedStructuralComparisonContract,
        candidates: Vec<StructuralMatchCandidate>,
    ) -> Result<PlannedStructuralMatchPacketSet, BridgeDeliveryError> {
        validation::validate_candidate_kinds(contract, &candidates)?;
        let validated = ValidatedStructuralIdentityDeclaration::from_contract(contract);
        Ok(PlannedStructuralMatchPacketSet::new(
            contract.clone(),
            validated,
            None,
            None,
            candidates,
        ))
    }

    /// Plans structural matching by first materializing fingerprints from read packets.
    ///
    /// This is the main advanced door for advisory remap-style structural work.
    pub fn plan_structural_match_packet_set_from_read_packets(
        &self,
        contract: &AdmittedStructuralComparisonContract,
        target_read_packet: SnapshotReadPacket,
        candidate_read_packets: Vec<SnapshotReadPacket>,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<PlannedStructuralMatchPacketSet, BridgeDeliveryError> {
        let target =
            self.materialize_structural_fingerprint(contract, target_read_packet, execution)?;
        let mut candidate_fingerprints = Vec::with_capacity(candidate_read_packets.len());
        for read_packet in candidate_read_packets {
            candidate_fingerprints.push(self.materialize_structural_fingerprint(
                contract,
                read_packet,
                execution,
            )?);
        }

        self.plan_structural_match_packet_set(
            contract,
            classify_advisory_candidates(&target, candidate_fingerprints),
        )
        .map(|planned| {
            PlannedStructuralMatchPacketSet::new(
                planned.contract().clone(),
                planned.validated_declaration().clone(),
                Some(target),
                None,
                planned.candidates().to_vec(),
            )
        })
    }

    /// Plans a branch-comparison structural packet set from one branch-pair read packet.
    ///
    /// ```no_run
    /// use worth_runtime_bridge::facade::{
    ///     AdmittedStructuralComparisonContract, RuntimeBridge, SnapshotReadPacket,
    /// };
    ///
    /// fn plan_branch_comparison(
    ///     bridge: &RuntimeBridge,
    ///     contract: &AdmittedStructuralComparisonContract,
    ///     packet: SnapshotReadPacket,
    ///     execution: worth_execution::ExecutionRequest<'_, '_>,
    /// ) -> Result<(), Box<dyn std::error::Error>> {
    ///     let planned = bridge.plan_structural_branch_comparison_from_read_packet(contract, packet, execution)?;
    ///     let reduced = bridge.reduce_structural_match_set(&planned)?;
    ///     let _artifact = bridge.publish_branch_comparison_artifact(&reduced)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn plan_structural_branch_comparison_from_read_packet(
        &self,
        contract: &AdmittedStructuralComparisonContract,
        read_packet: SnapshotReadPacket,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<PlannedStructuralMatchPacketSet, BridgeDeliveryError> {
        let (left, right) =
            self.materialize_structural_branch_fingerprints(contract, read_packet, execution)?;
        self.plan_structural_match_packet_set(contract, classify_branch_comparison(&left, &right))
            .map(|planned| {
                PlannedStructuralMatchPacketSet::new(
                    planned.contract().clone(),
                    planned.validated_declaration().clone(),
                    Some(left),
                    Some(right),
                    planned.candidates().to_vec(),
                )
            })
    }

    /// Reduces a planned structural packet set into a publication-ready outcome set.
    pub fn reduce_structural_match_set(
        &self,
        planned_packet_set: &PlannedStructuralMatchPacketSet,
    ) -> Result<ReducedStructuralMatchSet, BridgeDeliveryError> {
        validation::validate_candidate_kinds(
            planned_packet_set.contract(),
            planned_packet_set.candidates(),
        )?;
        Ok(ReducedStructuralMatchSet::from_planned_packet_set(
            planned_packet_set.clone(),
        ))
    }
}
