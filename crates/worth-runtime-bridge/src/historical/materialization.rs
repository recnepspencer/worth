use std::sync::Arc;

use crate::facade::{
    BridgeCanonicalHistoricalEvaluationRecord, BridgeDeliveryError, BridgeDeliveryErrorKind,
    BridgeHistoricalEvaluationCounters, BridgeHistoricalEvaluationDecisionLog,
    BridgeHistoricalEvaluationFailureClass, BridgeHistoricalEvaluationRecord, RuntimeBridge,
    TruthSnapshotIdentity,
};
use crate::historical::failures::{
    historical_failure_class_for_delivery_error, historical_failure_counters_for_delivery_error,
    historical_materialization_path_for,
};
use crate::snapshot::{
    BridgeTruthViewPolicyRejection, LoweredHistoricalEvaluationArtifact,
    MaterializedTruthViewObservation, PlannedTruthViewPacket, TruthViewPolicyRejectionKind,
};

impl RuntimeBridge {
    pub fn materialize_truth_view_observation(
        &self,
        planned: PlannedTruthViewPacket,
    ) -> Result<MaterializedTruthViewObservation, BridgeDeliveryError> {
        let snapshot_identity: TruthSnapshotIdentity = match planned
            .authority_basis()
            .snapshot_identity()
        {
            Some(snapshot_identity) => snapshot_identity.clone(),
            None => {
                let rejection = BridgeTruthViewPolicyRejection::new(
                planned.declaration(),
                TruthViewPolicyRejectionKind::UnsupportedTruthViewSelector,
                "planned truth-view packet did not carry a snapshot identity; historical lookup materialization is not wired yet",
            );
                self.record_historical_evaluation_failure(
                    planned.declaration(),
                    BridgeHistoricalEvaluationFailureClass::UnsupportedTruthViewSelector,
                    rejection.detail(),
                    BridgeHistoricalEvaluationCounters::from_failed_materialization(
                        planned.declaration(),
                        historical_materialization_path_for(&planned),
                    ),
                );
                return Err(BridgeDeliveryError::new(
                    BridgeDeliveryErrorKind::HistoricalPolicyRejected,
                    format!(
                        "Truth-view materialization rejected declaration `{}`: {}",
                        rejection.declaration_identity().as_str(),
                        rejection.detail()
                    ),
                ));
            }
        };

        let admitted = match crate::delivery::open_planned_snapshot(self, &snapshot_identity) {
            Ok(admitted) => admitted,
            Err(error) => {
                self.record_historical_evaluation_failure(
                    planned.declaration(),
                    historical_failure_class_for_delivery_error(&error),
                    error.to_string(),
                    historical_failure_counters_for_delivery_error(planned.declaration(), &error),
                );
                return Err(error);
            }
        };
        let snapshot_token = crate::snapshot::BridgeSnapshotToken::issued(
            snapshot_identity.clone(),
            format!(
                "truth-view-observation|declaration={}|policy={}|snapshot={}",
                planned.declaration().declaration_identity().as_str(),
                planned.resolved_policy().digest(),
                snapshot_identity.as_str()
            ),
        );
        let materialization_path = historical_materialization_path_for(&planned);

        Ok(MaterializedTruthViewObservation::new(
            planned,
            snapshot_token,
            materialization_path,
            admitted,
            self.policy().execution(),
        ))
    }

    pub fn canonicalize_historical_evaluation_record(
        &self,
        observation: &MaterializedTruthViewObservation,
    ) -> BridgeCanonicalHistoricalEvaluationRecord {
        let authority_basis = observation.authority_basis();
        let materialization_path = observation.materialization_path();
        let decision_log = BridgeHistoricalEvaluationDecisionLog::new(
            observation
                .planned()
                .declaration()
                .declaration_identity()
                .clone(),
            observation
                .planned()
                .declaration()
                .selector()
                .selector_identity()
                .clone(),
            Arc::from(observation.planned().resolved_policy().digest()),
            Arc::from(observation.planned().digest()),
            Arc::from(authority_basis.digest()),
            materialization_path,
            authority_basis.branch_identity().clone(),
            authority_basis.commit_identity().cloned(),
            authority_basis
                .snapshot_identity()
                .cloned()
                .unwrap_or_else(|| observation.snapshot_identity().clone()),
        );
        let counters = BridgeHistoricalEvaluationCounters::from_successful_materialization(
            observation.planned().declaration(),
            materialization_path,
        );
        let record =
            BridgeCanonicalHistoricalEvaluationRecord::new(BridgeHistoricalEvaluationRecord::new(
                observation.planned().declaration().clone(),
                observation.read_packet().clone(),
                decision_log,
                counters,
            ));
        self.diagnostic_sink
            .record_historical_evaluation(record.clone());
        record
    }

    pub fn lower_historical_evaluation_artifact(
        &self,
        observation: &MaterializedTruthViewObservation,
    ) -> LoweredHistoricalEvaluationArtifact {
        LoweredHistoricalEvaluationArtifact::lower(observation, observation.materialization_path())
    }
}
