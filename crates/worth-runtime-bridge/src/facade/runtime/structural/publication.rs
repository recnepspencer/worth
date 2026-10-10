use super::*;

impl RuntimeBridge {
    /// Publishes an advisory structural remap artifact from a reduced match set.
    ///
    /// This is the advanced publication door for advisory structural remap
    /// workflows after planning and reduction have already established the
    /// candidate set.
    pub fn publish_structural_remap_artifact(
        &self,
        reduced_match_set: &ReducedStructuralMatchSet,
    ) -> Result<PublishedStructuralRemapArtifact, BridgeDeliveryError> {
        PublishedStructuralRemapArtifact::from_reduced_match_set(reduced_match_set.clone())
            .ok_or_else(|| {
                BridgeDeliveryError::new(
                    BridgeDeliveryErrorKind::StructuralPlanRejected,
                    format!(
                        "Reduced structural match set `{}` does not describe an advisory remap publication outcome.",
                        reduced_match_set.digest()
                    ),
                )
            })
    }

    /// Publishes a branch-comparison artifact from a reduced match set.
    ///
    /// ```no_run
    /// use worth_runtime_bridge::facade::{
    ///     AdmittedStructuralComparisonContract, RuntimeBridge, SnapshotReadPacket,
    /// };
    ///
    /// fn publish_branch_comparison(
    ///     bridge: &RuntimeBridge,
    ///     contract: &AdmittedStructuralComparisonContract,
    ///     packet: SnapshotReadPacket,
    ///     execution: worth_execution::ExecutionRequest<'_, '_>,
    /// ) -> Result<(), Box<dyn std::error::Error>> {
    ///     let planned = bridge.plan_structural_branch_comparison_from_read_packet(contract, packet, execution)?;
    ///     let reduced = bridge.reduce_structural_match_set(&planned)?;
    ///     let artifact = bridge.publish_branch_comparison_artifact(&reduced)?;
    ///     let _record = bridge.canonicalize_structural_branch_comparison_record(
    ///         contract,
    ///         &planned,
    ///         &reduced,
    ///         &artifact,
    ///     );
    ///     Ok(())
    /// }
    /// ```
    pub fn publish_branch_comparison_artifact(
        &self,
        reduced_match_set: &ReducedStructuralMatchSet,
    ) -> Result<PublishedBranchComparisonArtifact, BridgeDeliveryError> {
        PublishedBranchComparisonArtifact::from_reduced_match_set(reduced_match_set.clone())
            .ok_or_else(|| {
                BridgeDeliveryError::new(
                    BridgeDeliveryErrorKind::StructuralPlanRejected,
                    format!(
                        "Reduced structural match set `{}` does not describe a branch comparison publication outcome.",
                        reduced_match_set.digest()
                    ),
                )
            })
    }

    /// Canonicalizes and records a structural remap artifact.
    ///
    /// Use this when an advanced structural remap workflow needs retained
    /// diagnostics or replay-safe comparison artifacts.
    pub fn canonicalize_structural_remap_record(
        &self,
        contract: &AdmittedStructuralComparisonContract,
        planned_packet_set: &PlannedStructuralMatchPacketSet,
        reduced_match_set: &ReducedStructuralMatchSet,
        artifact: &PublishedStructuralRemapArtifact,
    ) -> BridgeCanonicalStructuralRemapRecord {
        let counters = BridgeStructuralCounters::from_structural_outcome(
            contract,
            planned_packet_set,
            reduced_match_set,
        );
        let record = BridgeCanonicalStructuralRemapRecord::new(BridgeStructuralRemapRecord::new(
            contract.clone(),
            planned_packet_set.clone(),
            reduced_match_set.clone(),
            artifact.clone(),
            counters,
        ));
        self.diagnostics.record_structural_remap(record.clone());
        record
    }

    /// Canonicalizes and records a structural branch comparison artifact.
    pub fn canonicalize_structural_branch_comparison_record(
        &self,
        contract: &AdmittedStructuralComparisonContract,
        planned_packet_set: &PlannedStructuralMatchPacketSet,
        reduced_match_set: &ReducedStructuralMatchSet,
        artifact: &PublishedBranchComparisonArtifact,
    ) -> BridgeCanonicalStructuralBranchComparisonRecord {
        let counters = BridgeStructuralCounters::from_structural_outcome(
            contract,
            planned_packet_set,
            reduced_match_set,
        );
        let record = BridgeCanonicalStructuralBranchComparisonRecord::new(
            BridgeStructuralBranchComparisonRecord::new(
                contract.clone(),
                planned_packet_set.clone(),
                reduced_match_set.clone(),
                artifact.clone(),
                counters,
            ),
        );
        self.diagnostics
            .record_structural_branch_comparison(record.clone());
        record
    }
}
