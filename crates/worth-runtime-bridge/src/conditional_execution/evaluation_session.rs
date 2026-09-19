use std::sync::Arc;

use worth_signal::facade::branch::SignalConditionalEvaluationAdmission;

use super::{
    BridgeConditionalDenial, BridgeConditionalDenialKind, BridgeInstalledConditionalLowering,
    BridgeOwnedSignalRuntime,
};

type BridgeAdmittedSnapshot =
    crate::snapshot::AdmittedSnapshotContext<Box<dyn crate::snapshot::TruthSnapshotReader>>;

pub enum BridgeConditionalEvaluationSource<'a> {
    Relational(&'a crate::snapshot::TruthSnapshotIdentity),
    SourceFree,
}

pub struct BridgeConditionalEvaluationAdmissionRequest<'a> {
    pub lowering: &'a Arc<BridgeInstalledConditionalLowering>,
    pub source: BridgeConditionalEvaluationSource<'a>,
    pub(super) signal_basis: &'a super::BridgeConditionalSignalBasisBinding,
}

impl<'a> BridgeConditionalEvaluationAdmissionRequest<'a> {
    pub fn source_free_at_signal_basis(
        signal_basis: &'a super::BridgeConditionalSignalBasisBinding,
    ) -> Self {
        Self {
            lowering: &signal_basis.lowering,
            source: BridgeConditionalEvaluationSource::SourceFree,
            signal_basis,
        }
    }

    pub fn source_present_at_signal_basis(
        signal_basis: &'a super::BridgeConditionalSignalBasisBinding,
        identity: &'a crate::snapshot::TruthSnapshotIdentity,
    ) -> Self {
        Self {
            lowering: &signal_basis.lowering,
            source: BridgeConditionalEvaluationSource::Relational(identity),
            signal_basis,
        }
    }
}

/// Retained admission for one exact Bridge source and Signal execution basis.
/// Reusing the session never reopens its source and never creates another
/// Signal evaluation slot.
pub struct BridgeConditionalEvaluationSession {
    pub(super) bridge_runtime_key: u64,
    pub(super) lowering: Arc<BridgeInstalledConditionalLowering>,
    pub(super) source_snapshot: Option<Arc<BridgeAdmittedSnapshot>>,
    pub(super) signal: SignalConditionalEvaluationAdmission,
    pub(super) signal_port: super::signal_port::BridgeConditionalSignalPort,
    pub(super) snapshot_admission_attempts: usize,
    pub(super) managed_source_record:
        Option<crate::relational_identity::RelationalBridgeRecordIdentityParts>,
    pub(super) observation_baselines: Arc<super::observation_retention::BridgeObservationBaselines>,
    pub(super) readmission_counters: Option<super::BridgeConditionalEvaluationReadmissionCounters>,
}

impl std::fmt::Debug for BridgeConditionalEvaluationSession {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BridgeConditionalEvaluationSession")
            .field("posture", &"bridge-admitted")
            .finish_non_exhaustive()
    }
}

impl BridgeOwnedSignalRuntime {
    pub fn admit_conditional_evaluation(
        &self,
        request: BridgeConditionalEvaluationAdmissionRequest<'_>,
    ) -> Result<BridgeConditionalEvaluationSession, BridgeConditionalDenial> {
        self.admit_conditional_evaluation_with_record(request, None)
    }

    pub(super) fn admit_conditional_evaluation_with_record(
        &self,
        request: BridgeConditionalEvaluationAdmissionRequest<'_>,
        managed_source_record: Option<
            crate::relational_identity::RelationalBridgeRecordIdentityParts,
        >,
    ) -> Result<BridgeConditionalEvaluationSession, BridgeConditionalDenial> {
        let bridge_snapshot_identity = match request.source {
            BridgeConditionalEvaluationSource::Relational(identity) => Some(identity),
            BridgeConditionalEvaluationSource::SourceFree => None,
        };
        if !request.lowering.correspondences.is_empty() && bridge_snapshot_identity.is_none() {
            return Err(BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::MissingSourceObservation,
                "installed conditional dependencies require an admitted source observation",
            ));
        }
        if request.lowering.correspondences.is_empty() && bridge_snapshot_identity.is_some() {
            return Err(BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::SourcePostureMismatch,
                "source-free conditional lowering cannot admit a relational source",
            ));
        }
        self.require_current_signal_graph(request.lowering)?;
        self.require_live_installed_lowering(request.lowering)?;
        if !Arc::ptr_eq(&request.signal_basis.lowering, request.lowering) {
            return Err(BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::OperationAuthorityMismatch,
                "Signal-basis definition binding does not retain the requested lowering",
            ));
        }
        let observation_baselines =
            super::observation_retention::BridgeObservationBaselines::new(&self.retention)?;
        let source_snapshot = bridge_snapshot_identity
            .map(|identity| crate::delivery::open_planned_snapshot(&self.bridge, identity))
            .transpose()
            .map_err(|error| {
                BridgeConditionalDenial::new(
                    BridgeConditionalDenialKind::SnapshotAdmission,
                    format!("conditional snapshot admission failed: {error:?}"),
                )
            })?
            .map(Arc::new);
        let services = self.signal_services()?;
        let signal_port = request.signal_basis.signal_port.clone();
        let signal = signal_port
            .admit_evaluation(
                request.lowering.signal_contract(),
                services.evaluation_source(source_snapshot.as_deref()),
            )
            .map_err(super::execution::signal_service_denial)?;
        Ok(BridgeConditionalEvaluationSession {
            bridge_runtime_key: self.bridge.signal_runtime_key,
            lowering: Arc::clone(request.lowering),
            source_snapshot,
            signal,
            signal_port,
            snapshot_admission_attempts: usize::from(bridge_snapshot_identity.is_some()),
            managed_source_record,
            observation_baselines,
            readmission_counters: None,
        })
    }
}

impl BridgeConditionalEvaluationSession {
    pub const fn readmission_counters(
        &self,
    ) -> Option<super::BridgeConditionalEvaluationReadmissionCounters> {
        self.readmission_counters
    }
}
