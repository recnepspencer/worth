use crate::facade::{
    BridgeDeliveryError, BridgeDeliveryErrorKind, BridgeTruthViewSelector,
    HistoricalEvaluationDeclaration, RelationalCommittedPatchRequest, RuntimeBridge,
    SnapshotReadPacket,
};
use crate::historical::failures::{
    historical_failure_class_for_delivery_error, historical_failure_class_for_policy_rejection,
    historical_failure_counters_for_delivery_error,
    historical_failure_counters_for_policy_rejection,
};
use crate::snapshot::{
    BridgeTruthViewAuthorityBasis, BridgeTruthViewKind, BridgeTruthViewPolicyResolution,
    PlannedTruthViewPacket,
};

impl RuntimeBridge {
    pub fn plan_truth_view_packet(
        &self,
        declaration: HistoricalEvaluationDeclaration,
        read_packet: SnapshotReadPacket,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<PlannedTruthViewPacket, BridgeDeliveryError> {
        let resolved_policy = match self.resolve_truth_view_policy(&declaration) {
            BridgeTruthViewPolicyResolution::Admitted(policy) => policy,
            BridgeTruthViewPolicyResolution::Rejected(rejection) => {
                self.record_historical_evaluation_failure(
                    &declaration,
                    historical_failure_class_for_policy_rejection(rejection.kind()),
                    rejection.detail(),
                    historical_failure_counters_for_policy_rejection(
                        &declaration,
                        rejection.kind(),
                    ),
                );
                return Err(BridgeDeliveryError::new(
                    BridgeDeliveryErrorKind::HistoricalPolicyRejected,
                    format!(
                        "Historical evaluation declaration `{}` was rejected during truth-view policy resolution: {}",
                        rejection.declaration_identity().as_str(),
                        rejection.detail()
                    ),
                ));
            }
        };
        let authority_basis =
            match self.resolve_truth_view_authority_basis(declaration.selector(), execution) {
                Ok(authority_basis) => authority_basis,
                Err(error) => {
                    self.record_historical_evaluation_failure(
                        &declaration,
                        historical_failure_class_for_delivery_error(&error),
                        error.to_string(),
                        historical_failure_counters_for_delivery_error(&declaration, &error),
                    );
                    return Err(error);
                }
            };
        Ok(PlannedTruthViewPacket::new(
            declaration,
            resolved_policy,
            authority_basis,
            read_packet,
        ))
    }

    pub(super) fn resolve_truth_view_authority_basis(
        &self,
        selector: &BridgeTruthViewSelector,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeTruthViewAuthorityBasis, BridgeDeliveryError> {
        match selector.view_kind() {
            BridgeTruthViewKind::CommittedSnapshot | BridgeTruthViewKind::BranchSnapshot => {
                Ok(BridgeTruthViewAuthorityBasis::from_selector(selector))
            }
            BridgeTruthViewKind::HistoricalCommit | BridgeTruthViewKind::BranchCommit => {
                let commit_identity = selector.commit_identity().ok_or_else(|| {
                    BridgeDeliveryError::new(
                        BridgeDeliveryErrorKind::HistoricalSelectorMissingCommit,
                        format!(
                            "Truth-view selector `{}` did not carry a required commit identity.",
                            selector.selector_identity().as_str()
                        ),
                    )
                })?;
                let envelope = self
                    .committed_patch_source
                    .load_committed_patch(RelationalCommittedPatchRequest::on_branch(
                        commit_identity.clone(),
                        selector.branch_identity().clone(),
                    ), execution)
                    .map_err(|error| {
                        BridgeDeliveryError::new(
                            error.historical_delivery_kind(),
                            format!(
                                "Truth-view selector `{}` could not resolve committed envelope for `{}`: {error}",
                                selector.selector_identity().as_str(),
                                commit_identity.as_str()
                            ),
                        )
                    })?;
                self.authority_basis_from_envelope(selector, &envelope)
            }
            BridgeTruthViewKind::BranchHead => {
                let source = self.truth_branch_head_source.as_ref().ok_or_else(|| {
                    BridgeDeliveryError::new(
                        BridgeDeliveryErrorKind::MissingBranchHeadSource,
                        format!(
                            "Truth-view selector `{}` requires a configured truth branch-head source.",
                            selector.selector_identity().as_str()
                        ),
                    )
                })?;
                let envelope = source
                    .load_branch_head_patch(selector.branch_identity(), execution)
                    .map_err(|error| {
                        BridgeDeliveryError::new(
                            error.historical_delivery_kind(),
                            format!(
                                "Truth-view selector `{}` could not resolve branch head for `{}`: {error}",
                                selector.selector_identity().as_str(),
                                selector.branch_identity().as_str()
                            ),
                        )
                    })?;
                self.authority_basis_from_envelope(selector, &envelope)
            }
        }
    }

    fn authority_basis_from_envelope(
        &self,
        selector: &BridgeTruthViewSelector,
        envelope: &crate::input::envelope::BridgeCommittedPatchEnvelope,
    ) -> Result<BridgeTruthViewAuthorityBasis, BridgeDeliveryError> {
        if envelope.branch_identity() != selector.branch_identity() {
            return Err(BridgeDeliveryError::new(
                BridgeDeliveryErrorKind::HistoricalBranchMismatch,
                format!(
                    "Truth-view selector `{}` targeted branch `{}` but resolved authority was on branch `{}`.",
                    selector.selector_identity().as_str(),
                    selector.branch_identity().as_str(),
                    envelope.branch_identity().as_str()
                ),
            ));
        }

        if let Some(selector_commit_identity) = selector.commit_identity() {
            if envelope.commit_identity() != selector_commit_identity {
                return Err(BridgeDeliveryError::new(
                    BridgeDeliveryErrorKind::HistoricalCommitMismatch,
                    format!(
                        "Truth-view selector `{}` targeted commit `{}` but resolved authority bound commit `{}`.",
                        selector.selector_identity().as_str(),
                        selector_commit_identity.as_str(),
                        envelope.commit_identity().as_str()
                    ),
                ));
            }
        }

        if let Some(selector_snapshot_identity) = selector.snapshot_identity() {
            if envelope.snapshot_identity() != selector_snapshot_identity {
                return Err(BridgeDeliveryError::new(
                    BridgeDeliveryErrorKind::SnapshotIdentityMismatch,
                    format!(
                        "Truth-view selector `{}` targeted snapshot `{}` but resolved authority bound snapshot `{}`.",
                        selector.selector_identity().as_str(),
                        selector_snapshot_identity.as_str(),
                        envelope.snapshot_identity().as_str()
                    ),
                ));
            }
        }

        Ok(BridgeTruthViewAuthorityBasis::from_resolved_envelope(
            selector,
            envelope.commit_identity().clone(),
            envelope.snapshot_identity().clone(),
        ))
    }
}
