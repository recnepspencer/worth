use crate::facade::{
    BridgeCanonicalHistoricalEvaluationRecord, BridgeErrorContext,
    BridgeHistoricalEvaluationCounters, BridgeHistoricalEvaluationFailureClass,
    BridgeHistoricalEvaluationReplaySummary, BridgeReplayError, BridgeReplayErrorKind,
    RuntimeBridge,
};

impl RuntimeBridge {
    pub fn replay_canonical_historical_evaluation_record(
        &self,
        record: &BridgeCanonicalHistoricalEvaluationRecord,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeHistoricalEvaluationReplaySummary, BridgeReplayError> {
        let record = record.decode()?;
        let planned = self
            .plan_truth_view_packet(record.declaration().clone(), record.read_packet().clone(), execution)
            .map_err(|error| {
                let replay_error = BridgeReplayError::new(
                    BridgeReplayErrorKind::HistoricalEvaluationDeclarationMismatch,
                    format!(
                        "Bridge historical evaluation replay could not reconstruct the planned truth-view packet: {error}"
                    ),
                )
                .with_context(BridgeErrorContext::default());
                self.record_historical_evaluation_failure(
                    record.declaration(),
                    BridgeHistoricalEvaluationFailureClass::HistoricalReplayMismatch,
                    replay_error.to_string(),
                    BridgeHistoricalEvaluationCounters::from_successful_materialization(
                        record.declaration(),
                        record.decision_log().materialization_path(),
                    )
                    .with_historical_replay_mismatch(),
                );
                replay_error
            })?;

        if planned.resolved_policy().digest() != record.decision_log().resolved_policy_digest() {
            let error = BridgeReplayError::new(
                BridgeReplayErrorKind::HistoricalEvaluationPolicyMismatch,
                format!(
                    "Bridge historical evaluation replay reconstructed policy `{}` but original policy was `{}`.",
                    planned.resolved_policy().digest(),
                    record.decision_log().resolved_policy_digest()
                ),
            )
            .with_context(BridgeErrorContext::default());
            self.record_historical_evaluation_failure(
                record.declaration(),
                BridgeHistoricalEvaluationFailureClass::HistoricalReplayMismatch,
                error.to_string(),
                BridgeHistoricalEvaluationCounters::from_successful_materialization(
                    record.declaration(),
                    record.decision_log().materialization_path(),
                )
                .with_historical_replay_mismatch(),
            );
            return Err(error);
        }

        if planned.authority_basis().digest() != record.decision_log().authority_digest() {
            let error = BridgeReplayError::new(
                BridgeReplayErrorKind::HistoricalEvaluationAuthorityMismatch,
                format!(
                    "Bridge historical evaluation replay reconstructed authority `{}` but original authority was `{}`.",
                    planned.authority_basis().digest(),
                    record.decision_log().authority_digest()
                ),
            )
            .with_context(BridgeErrorContext::default());
            self.record_historical_evaluation_failure(
                record.declaration(),
                BridgeHistoricalEvaluationFailureClass::HistoricalReplayMismatch,
                error.to_string(),
                BridgeHistoricalEvaluationCounters::from_successful_materialization(
                    record.declaration(),
                    record.decision_log().materialization_path(),
                )
                .with_historical_replay_mismatch(),
            );
            return Err(error);
        }

        let observation = self
            .materialize_truth_view_observation(planned, execution)
            .map_err(|error| {
                let replay_error = BridgeReplayError::new(
                    BridgeReplayErrorKind::HistoricalEvaluationAuthorityMismatch,
                    format!(
                        "Bridge historical evaluation replay could not materialize the planned truth view: {error}"
                    ),
                )
                .with_context(BridgeErrorContext::default());
                self.record_historical_evaluation_failure(
                    record.declaration(),
                    BridgeHistoricalEvaluationFailureClass::HistoricalReplayMismatch,
                    replay_error.to_string(),
                    BridgeHistoricalEvaluationCounters::from_successful_materialization(
                        record.declaration(),
                        record.decision_log().materialization_path(),
                    )
                    .with_historical_replay_mismatch(),
                );
                replay_error
            })?;

        if observation.snapshot_identity() != record.decision_log().snapshot_identity() {
            let error = BridgeReplayError::new(
                BridgeReplayErrorKind::HistoricalEvaluationAuthorityMismatch,
                format!(
                    "Bridge historical evaluation replay reconstructed snapshot `{}` but original snapshot was `{}`.",
                    observation.snapshot_identity().as_str(),
                    record.decision_log().snapshot_identity().as_str()
                ),
            )
            .with_context(BridgeErrorContext::default());
            self.record_historical_evaluation_failure(
                record.declaration(),
                BridgeHistoricalEvaluationFailureClass::HistoricalReplayMismatch,
                error.to_string(),
                BridgeHistoricalEvaluationCounters::from_successful_materialization(
                    record.declaration(),
                    record.decision_log().materialization_path(),
                )
                .with_historical_replay_mismatch(),
            );
            return Err(error);
        }

        Ok(BridgeHistoricalEvaluationReplaySummary::from_record(
            &record,
        ))
    }
}
