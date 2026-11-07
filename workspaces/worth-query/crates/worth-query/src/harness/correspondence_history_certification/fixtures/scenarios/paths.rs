use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::facade::foundation::{
    admit_historical_evaluation_path, resolve_historical_materialization_path,
    HistoricalCapabilityDescriptor, HistoricalEvaluationRequest,
    HistoricalMaterializationDescriptor, HistoricalPathResolved, HistoricalPathReuseDescriptor,
    ResolvedHistoricalPathClass,
};

pub(crate) fn retained_resolved(
    basis: &str,
) -> Result<HistoricalPathResolved, IdentityDerivationDenial> {
    let request = HistoricalEvaluationRequest::retained_snapshot_for_test(
        basis.to_string(),
        1,
        1,
        HistoricalPathReuseDescriptor::retained_reuse(),
    )?;
    let capability = HistoricalCapabilityDescriptor::new_for_test(
        basis.to_string(),
        Some(crate::historical::AdmittedHistoricalPathClass::AdmittedRetainedSnapshotPath),
        false,
        false,
        true,
        false,
        HistoricalPathReuseDescriptor::retained_reuse(),
    )?;
    let admission =
        admit_historical_evaluation_path(request, capability).expect("admission should succeed");
    Ok(resolve_historical_materialization_path(
        admission,
        HistoricalMaterializationDescriptor::new_for_test(
            basis.to_string(),
            ResolvedHistoricalPathClass::ResolvedRetainedSnapshotPath,
        )?,
    )
    .expect("resolution should succeed"))
}

pub(crate) fn replay_resolved(
    basis: &str,
) -> Result<HistoricalPathResolved, IdentityDerivationDenial> {
    let request = HistoricalEvaluationRequest::delta_replay_for_test(
        basis.to_string(),
        4,
        8,
        HistoricalPathReuseDescriptor::with_replay_tail_reuse(),
    )?;
    let capability = HistoricalCapabilityDescriptor::new_for_test(
        basis.to_string(),
        Some(crate::historical::AdmittedHistoricalPathClass::AdmittedDeltaReplayPath),
        true,
        false,
        false,
        true,
        HistoricalPathReuseDescriptor::with_replay_tail_reuse(),
    )?;
    let admission =
        admit_historical_evaluation_path(request, capability).expect("admission should succeed");
    Ok(resolve_historical_materialization_path(
        admission,
        HistoricalMaterializationDescriptor::new_for_test(
            basis.to_string(),
            ResolvedHistoricalPathClass::ResolvedDeltaReplayPath,
        )?,
    )
    .expect("resolution should succeed"))
}

pub(crate) fn reconstruction_resolved(
    basis: &str,
) -> Result<HistoricalPathResolved, IdentityDerivationDenial> {
    let request = HistoricalEvaluationRequest::full_reconstruction_for_test(
        basis.to_string(),
        4,
        8,
        HistoricalPathReuseDescriptor::no_reuse(),
    )?;
    let capability = HistoricalCapabilityDescriptor::new_for_test(
        basis.to_string(),
        Some(crate::historical::AdmittedHistoricalPathClass::AdmittedFullReconstructionPath),
        true,
        false,
        false,
        true,
        HistoricalPathReuseDescriptor::no_reuse(),
    )?;
    let admission =
        admit_historical_evaluation_path(request, capability).expect("admission should succeed");
    Ok(resolve_historical_materialization_path(
        admission,
        HistoricalMaterializationDescriptor::new_for_test(
            basis.to_string(),
            ResolvedHistoricalPathClass::ResolvedFullReconstructionPath,
        )?,
    )
    .expect("resolution should succeed"))
}
