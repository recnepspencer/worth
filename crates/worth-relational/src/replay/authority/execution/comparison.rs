use crate::replay::data::{
    CanonicalCommitEnvelope, RelationalReplayOutcome, RelationalReplayRequest,
    ReplayAuthorityBasisKind, ReplayLineageAuthorityBasis, ReplayLineageDigestMode, ReplayMismatch,
    ReplayObservableSurface, ReplayVerificationMode,
};
use crate::runtime::RelationalRuntime;
use worth_relational_durability::internal_port::replay::data::lineage_authority::ReplayLineageAuthorityBasisOwnerPort as _;

use super::super::lineage_authority::{
    lineage_decision_log_comparison_basis, lineage_event_batch_comparison_basis,
    select_published_lineage_authority,
};
use super::ReplayAdmission;

pub(super) fn select_replay_lineage_authority<'a>(
    runtime: &'a RelationalRuntime,
    envelope: &'a CanonicalCommitEnvelope,
    verification_mode: ReplayVerificationMode,
) -> Result<super::super::SelectedPublishedLineageAuthority, ()> {
    let selected = select_published_lineage_authority(runtime, envelope);
    runtime
        .performance_access()
        .count_replay_lineage_authority_basis(
            selected.indexed_source,
            selected.kind,
            selected.artifact.digest_basis().lineage_event_count(),
            selected.artifact.digest_basis().lineage_decision_count(),
        );
    if selected.kind == ReplayAuthorityBasisKind::RetainedEnvelopeCanonical
        && verification_mode != ReplayVerificationMode::NormalRecoveryVerification
    {
        runtime
            .performance_access()
            .count_replay_lineage_authoritative_basis_rejection();
        Err(())
    } else {
        Ok(selected)
    }
}

pub(super) fn replay_comparison_outcome(
    request: RelationalReplayRequest,
    admission: &ReplayAdmission,
    compared_surfaces: &[ReplayObservableSurface],
    mismatches: Vec<ReplayMismatch>,
    selected_lineage: Option<&super::super::SelectedPublishedLineageAuthority>,
) -> RelationalReplayOutcome {
    RelationalReplayOutcome {
        requested: request,
        commit: Some(admission.envelope.commit.clone()),
        reconstructed_commit_closure: admission.commit_closure.clone(),
        snapshot_version: Some(admission.envelope.commit.version_id),
        lineage_authority_basis: selected_lineage.map(|selected| {
            ReplayLineageAuthorityBasis::new(
                selected.kind,
                admission.envelope.commit.commit_id,
                ReplayLineageDigestMode::ExactCanonicalArtifactDigest,
                selected.artifact.digest_basis().lineage_event_count(),
                selected.artifact.digest_basis().lineage_decision_count(),
                lineage_event_batch_comparison_basis(&selected.artifact),
                lineage_decision_log_comparison_basis(&selected.artifact),
            )
        }),
        compared_surfaces: compared_surfaces.to_vec(),
        failure: (!mismatches.is_empty())
            .then_some(crate::replay::data::ReplayFailureClass::ObservableMismatch),
        mismatches,
    }
}
