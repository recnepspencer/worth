use crate::application::WorthQueryContinuationExecutionReadmissionObservation;
use crate::basis_lifecycle::LowerRuntimeEvidenceAuthority;
use crate::continuation_pipeline::readmission::{
    continuation_readmission_basis_identity,
    continuation_readmission_lower_runtime_binding_identity,
    WorthQueryPreparedContinuationDriftKind, WorthQueryPreparedContinuationExecutionReadmission,
    WorthQueryPreparedContinuationFreshnessPosture,
};

use super::readmission::lower_runtime_authority_from_witness;

pub(crate) fn drifted_observation_from_retained(
    retained: &WorthQueryPreparedContinuationExecutionReadmission,
    freshness_posture: WorthQueryPreparedContinuationFreshnessPosture,
    basis_identity_digest: Option<String>,
    lower_runtime_binding_identity_digest: Option<String>,
    authority: Option<LowerRuntimeEvidenceAuthority>,
    drift_kind: Option<WorthQueryPreparedContinuationDriftKind>,
) -> WorthQueryContinuationExecutionReadmissionObservation {
    let witness = retained.basis_witness();
    WorthQueryContinuationExecutionReadmissionObservation::new(
        authority
            .unwrap_or_else(|| lower_runtime_authority_from_witness(retained.authority_witness())),
        basis_identity_digest
            .map(|identity| {
                continuation_readmission_basis_identity(witness.kind(), identity)
                    .expect("fixture digest must fit")
            })
            .unwrap_or_else(|| witness.basis_identity().clone()),
        lower_runtime_binding_identity_digest
            .map(|digest| {
                continuation_readmission_lower_runtime_binding_identity(digest)
                    .expect("fixture digest must fit")
            })
            .or_else(|| witness.expected_lower_runtime_binding_identity().cloned()),
        freshness_posture,
        drift_kind,
    )
}
