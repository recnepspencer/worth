use super::*;

impl RuntimeBridge {
    /// Materializes one structural fingerprint from a single-basis structural contract.
    ///
    /// This is an advanced structural helper that turns truth-view material
    /// into a fingerprint suitable for advisory remap planning or replay proof.
    pub fn materialize_structural_fingerprint(
        &self,
        contract: &AdmittedStructuralComparisonContract,
        read_packet: SnapshotReadPacket,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<StructuralFingerprint, BridgeDeliveryError> {
        let declaration = contract.validated_declaration().declaration();
        let selector = match declaration.truth_view_basis() {
            StructuralTruthViewBasis::Single { selector, .. } => selector.clone(),
            StructuralTruthViewBasis::BranchPair { .. } => {
                return Err(BridgeDeliveryError::new(
                    BridgeDeliveryErrorKind::StructuralPlanRejected,
                    format!(
                        "Structural contract `{}` requires a branch-pair basis and cannot materialize a single structural fingerprint.",
                        contract.contract_identity().as_str()
                    ),
                ))
            }
        };

        let observation = self.materialize_truth_view_observation(
            self.plan_truth_view_packet(
                HistoricalEvaluationDeclaration::new(
                    selector,
                    BridgeReplayMode::Enabled,
                    BridgeDiagnosticsTier::Standard,
                    BridgeDeliveryIntent::PrepareSignalEvaluation,
                ),
                read_packet,
                execution,
            )?,
            execution,
        )?;

        StructuralFingerprint::from_observation(contract, &observation, execution).map_err(
            |error| {
                BridgeDeliveryError::new(
                    error.delivery_kind(BridgeDeliveryErrorKind::SnapshotReadContractViolation),
                    format!(
                        "Structural fingerprint materialization could not validate reads: {error}"
                    ),
                )
            },
        )
    }

    /// Materializes the left and right structural fingerprints for a branch-pair contract.
    pub fn materialize_structural_branch_fingerprints(
        &self,
        contract: &AdmittedStructuralComparisonContract,
        read_packet: SnapshotReadPacket,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(StructuralFingerprint, StructuralFingerprint), BridgeDeliveryError> {
        let declaration = contract.validated_declaration().declaration();
        let (left_selector, right_selector) = match declaration.truth_view_basis() {
            StructuralTruthViewBasis::BranchPair {
                left_selector,
                right_selector,
                ..
            } => (left_selector.clone(), right_selector.clone()),
            StructuralTruthViewBasis::Single { .. } => {
                return Err(BridgeDeliveryError::new(
                    BridgeDeliveryErrorKind::StructuralPlanRejected,
                    format!(
                    "Structural contract `{}` does not admit branch-pair structural comparison.",
                    contract.contract_identity().as_str()
                ),
                ))
            }
        };

        let left = self.materialize_truth_view_observation(
            self.plan_truth_view_packet(
                HistoricalEvaluationDeclaration::new(
                    left_selector,
                    BridgeReplayMode::Enabled,
                    BridgeDiagnosticsTier::Standard,
                    BridgeDeliveryIntent::PrepareSignalEvaluation,
                ),
                read_packet.clone(),
                execution,
            )?,
            execution,
        )?;
        let right = self.materialize_truth_view_observation(
            self.plan_truth_view_packet(
                HistoricalEvaluationDeclaration::new(
                    right_selector,
                    BridgeReplayMode::Enabled,
                    BridgeDiagnosticsTier::Standard,
                    BridgeDeliveryIntent::PrepareSignalEvaluation,
                ),
                read_packet,
                execution,
            )?,
            execution,
        )?;

        let left =
            StructuralFingerprint::from_observation(contract, &left, execution).map_err(
                |error| {
                    BridgeDeliveryError::new(
                error.delivery_kind(BridgeDeliveryErrorKind::SnapshotReadContractViolation),
                format!("Structural branch comparison could not validate left-side reads: {error}"),
            )
                },
            )?;
        let right = StructuralFingerprint::from_observation(contract, &right, execution).map_err(
            |error| {
                BridgeDeliveryError::new(
                    error.delivery_kind(BridgeDeliveryErrorKind::SnapshotReadContractViolation),
                    format!(
                        "Structural branch comparison could not validate right-side reads: {error}"
                    ),
                )
            },
        )?;

        Ok((left, right))
    }
}
