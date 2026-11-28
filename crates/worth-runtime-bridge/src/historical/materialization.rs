use std::sync::Arc;

use crate::facade::{
    BridgeCanonicalHistoricalEvaluationRecord, BridgeDeliveryError, BridgeDeliveryErrorKind,
    BridgeHistoricalEvaluationCounters, BridgeHistoricalEvaluationDecisionLog,
    BridgeHistoricalEvaluationRecord, RuntimeBridge, TruthSnapshotIdentity,
};
use crate::historical::failures::{
    historical_failure_class_for_delivery_error, historical_failure_counters_for_delivery_error,
    historical_materialization_path_for,
};
use crate::snapshot::{
    AdmittedSnapshotContext, BridgeSnapshotContext, LoweredHistoricalEvaluationArtifact,
    MaterializedTruthViewObservation, PlannedTruthViewPacket, TruthSnapshotReader,
};

impl RuntimeBridge {
    pub fn materialize_truth_view_observation(
        &self,
        planned: PlannedTruthViewPacket,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<MaterializedTruthViewObservation, BridgeDeliveryError> {
        self.materialize_planned_truth_view(planned.clone(), execution)
            .inspect_err(|error| {
                self.record_historical_evaluation_failure(
                    planned.declaration(),
                    historical_failure_class_for_delivery_error(error),
                    error.to_string(),
                    historical_failure_counters_for_delivery_error(planned.declaration(), error),
                );
            })
    }

    pub(crate) fn materialize_planned_truth_view(
        &self,
        planned: PlannedTruthViewPacket,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<MaterializedTruthViewObservation, BridgeDeliveryError> {
        let snapshot_identity = planned
            .authority_basis()
            .snapshot_identity()
            .ok_or_else(|| {
                BridgeDeliveryError::new(
                    BridgeDeliveryErrorKind::HistoricalPolicyRejected,
                    "planned truth-view packet did not carry a snapshot identity",
                )
            })?;
        let admitted = self.open_truth_view_snapshot(&planned, snapshot_identity, execution)?;
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
        ))
    }

    fn open_truth_view_snapshot(
        &self,
        planned: &PlannedTruthViewPacket,
        identity: &TruthSnapshotIdentity,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<AdmittedSnapshotContext<Box<dyn TruthSnapshotReader>>, BridgeDeliveryError> {
        let Some(pinned) = planned.pinned_commit() else {
            return crate::delivery::open_planned_snapshot(self, identity, execution);
        };
        crate::execution_contact::admit(execution).map_err(|cause| {
            BridgeDeliveryError::new(
                BridgeDeliveryErrorKind::ExecutionDenied(cause),
                cause.label(),
            )
        })?;
        if !Arc::ptr_eq(&pinned.source, &self.snapshot_read_source) {
            return Err(BridgeDeliveryError::new(
                BridgeDeliveryErrorKind::SourceContractMismatch,
                "historical commit plan belongs to another snapshot source",
            ));
        }
        let reader: Box<dyn TruthSnapshotReader> = Box::new(pinned.reader.clone());
        AdmittedSnapshotContext::admit_for(BridgeSnapshotContext::bind(reader), identity).map_err(
            |_| {
                BridgeDeliveryError::new(
                    BridgeDeliveryErrorKind::SnapshotIdentityMismatch,
                    "pinned historical reader did not match its planned snapshot",
                )
            },
        )
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
        self.diagnostics
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
