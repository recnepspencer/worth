use crate::commit_strategies::data::StrategyCommitArtifactBundle;
use crate::history::data::{OrderedParentList, RelationalCommitReceipt};
use crate::replay::data::{
    digest_branch_head_summary, digest_branch_head_surface, digest_canonical_patch_summary,
    digest_canonical_patch_surface, digest_diagnostics_summary, digest_diagnostics_surface,
    digest_history_summary, digest_history_surface, digest_snapshot_summary,
    digest_snapshot_surface, digest_strategy_replay_descriptor, digest_strategy_replay_summary,
    CanonicalCommitEnvelope, DescriptorComparisonBasis, DescriptorParityCheck, ReplayMismatch,
    ReplayMismatchClass, ReplayObservableSurface, ReplaySnapshotSurface,
    ReplaySurfaceAuthorityKind, ReplaySurfaceComparisonBasis, ReplaySurfaceParityCheck,
    ReplayVerificationLayer, ReplayVerificationPlan, VerifiedReplaySurfaceDigest,
};
use crate::runtime::RelationalRuntime;

use super::continuity::replay_history_drift_class;
pub(super) fn compare_replay_surface(
    runtime: &RelationalRuntime,
    verification_plan: &ReplayVerificationPlan,
    mismatches: &mut Vec<ReplayMismatch>,
    surface: ReplayObservableSurface,
    mismatch_class: ReplayMismatchClass,
    expected: ReplaySurfaceComparisonBasis,
    observed: ReplaySurfaceComparisonBasis,
    detail: &str,
    deep_matches: impl FnOnce() -> bool,
    render_expected: impl FnOnce() -> String,
    render_observed: impl FnOnce() -> String,
) {
    let parity = expected.compare(&observed, mismatch_class, detail);
    match parity {
        ReplaySurfaceParityCheck::ExactDigestMatch { .. } => {
            runtime
                .performance_access()
                .count_replay_verification_layer(ReplayVerificationLayer::DigestParity);
        }
        ReplaySurfaceParityCheck::SummaryMatchDigestUnavailable { .. } => {
            runtime
                .performance_access()
                .count_replay_verification_layer(ReplayVerificationLayer::SummaryParity);
        }
        ReplaySurfaceParityCheck::Drift {
            mismatch_class,
            layer,
            detail,
            ..
        } => {
            runtime
                .performance_access()
                .count_replay_verification_layer(layer);
            if layer != ReplayVerificationLayer::DeepArtifactParity
                && verification_plan.allows_deep_artifact_parity()
            {
                if deep_matches() {
                    runtime
                        .performance_access()
                        .count_replay_verification_layer(
                            ReplayVerificationLayer::DeepArtifactParity,
                        );
                    return;
                }
                runtime
                    .performance_access()
                    .count_replay_verification_layer(ReplayVerificationLayer::DeepArtifactParity);
                mismatches.push(ReplayMismatch {
                    class: mismatch_class,
                    history_drift_class: replay_history_drift_class(mismatch_class),
                    surface,
                    verification_layer: ReplayVerificationLayer::DeepArtifactParity,
                    detail: format!("{} (confirmed by audit/deep artifact parity)", detail),
                    expected: Some(render_expected()),
                    observed: Some(render_observed()),
                });
                return;
            }
            mismatches.push(ReplayMismatch {
                class: mismatch_class,
                history_drift_class: replay_history_drift_class(mismatch_class),
                surface,
                verification_layer: layer,
                detail,
                expected: Some(render_expected()),
                observed: Some(render_observed()),
            });
        }
    }
}

pub(super) fn compare_descriptor_surface(
    runtime: &RelationalRuntime,
    verification_plan: &ReplayVerificationPlan,
    mismatches: &mut Vec<ReplayMismatch>,
    expected: Option<DescriptorComparisonBasis>,
    observed: Option<DescriptorComparisonBasis>,
    mismatch_class: ReplayMismatchClass,
    detail: &str,
    deep_matches: impl FnOnce() -> bool,
    render_expected: impl FnOnce() -> String,
    render_observed: impl FnOnce() -> String,
) {
    match (expected, observed) {
        (None, None) => {}
        (Some(expected), Some(observed)) => {
            let parity = expected.compare(&observed, mismatch_class, detail);
            match parity {
                DescriptorParityCheck::ExactDigestMatch { .. } => {
                    runtime
                        .performance_access()
                        .count_replay_verification_layer(ReplayVerificationLayer::DigestParity);
                }
                DescriptorParityCheck::SummaryMatchDigestUnavailable { .. } => {
                    runtime
                        .performance_access()
                        .count_replay_verification_layer(ReplayVerificationLayer::SummaryParity);
                }
                DescriptorParityCheck::Drift {
                    mismatch_class,
                    layer,
                    detail,
                    ..
                } => {
                    runtime
                        .performance_access()
                        .count_replay_verification_layer(layer);
                    if layer == ReplayVerificationLayer::DigestParity
                        && verification_plan.allows_deep_artifact_parity()
                    {
                        if deep_matches() {
                            runtime
                                .performance_access()
                                .count_replay_verification_layer(
                                    ReplayVerificationLayer::DeepArtifactParity,
                                );
                            return;
                        }
                        runtime
                            .performance_access()
                            .count_replay_verification_layer(
                                ReplayVerificationLayer::DeepArtifactParity,
                            );
                        mismatches.push(ReplayMismatch {
                            class: mismatch_class,
                            history_drift_class: replay_history_drift_class(mismatch_class),
                            surface: ReplayObservableSurface::History,
                            verification_layer: ReplayVerificationLayer::DeepArtifactParity,
                            detail: format!("{} (confirmed by audit/deep artifact parity)", detail),
                            expected: Some(render_expected()),
                            observed: Some(render_observed()),
                        });
                        return;
                    }
                    mismatches.push(ReplayMismatch {
                        class: mismatch_class,
                        history_drift_class: replay_history_drift_class(mismatch_class),
                        surface: ReplayObservableSurface::History,
                        verification_layer: layer,
                        detail: detail.to_string(),
                        expected: Some(render_expected()),
                        observed: Some(render_observed()),
                    });
                }
            }
        }
        _ => {
            let layer = if verification_plan.allows_deep_artifact_parity() {
                ReplayVerificationLayer::DeepArtifactParity
            } else {
                ReplayVerificationLayer::DigestParity
            };
            runtime
                .performance_access()
                .count_replay_verification_layer(layer);
            mismatches.push(ReplayMismatch {
                class: mismatch_class,
                history_drift_class: replay_history_drift_class(mismatch_class),
                surface: ReplayObservableSurface::History,
                verification_layer: layer,
                detail: detail.to_string(),
                expected: Some(render_expected()),
                observed: Some(render_observed()),
            })
        }
    }
}

pub(super) fn surface_basis_for_patch(
    envelope: &CanonicalCommitEnvelope,
) -> ReplaySurfaceComparisonBasis {
    let patch = envelope.patch.canonicalized();
    ReplaySurfaceComparisonBasis::new(
        ReplaySurfaceAuthorityKind::Patch,
        Some(VerifiedReplaySurfaceDigest::from_digest(
            ReplaySurfaceAuthorityKind::Patch,
            digest_canonical_patch_surface(&patch),
        )),
        Some(digest_canonical_patch_summary(&patch)),
    )
}

pub(super) fn surface_basis_for_diagnostics(
    summary: &crate::diagnostics::data::RelationalDiagnosticArtifact,
) -> ReplaySurfaceComparisonBasis {
    ReplaySurfaceComparisonBasis::new(
        ReplaySurfaceAuthorityKind::Diagnostics,
        Some(VerifiedReplaySurfaceDigest::from_digest(
            ReplaySurfaceAuthorityKind::Diagnostics,
            digest_diagnostics_surface(summary),
        )),
        Some(digest_diagnostics_summary(summary)),
    )
}

pub(super) fn surface_basis_for_history(
    envelope: &CanonicalCommitEnvelope,
) -> ReplaySurfaceComparisonBasis {
    let history = (
        OrderedParentList::from_authoritative(envelope.commit.parents.clone()),
        envelope.merge_parent_branches.clone(),
        envelope.merge_base_commits.clone(),
    );
    ReplaySurfaceComparisonBasis::new(
        ReplaySurfaceAuthorityKind::History,
        Some(VerifiedReplaySurfaceDigest::from_digest(
            ReplaySurfaceAuthorityKind::History,
            digest_history_surface(&history.0, &history.1, &history.2),
        )),
        Some(digest_history_summary(&history.0)),
    )
}

pub(super) fn surface_basis_for_snapshot(
    surface: &ReplaySnapshotSurface,
) -> ReplaySurfaceComparisonBasis {
    ReplaySurfaceComparisonBasis::new(
        ReplaySurfaceAuthorityKind::Snapshot,
        Some(VerifiedReplaySurfaceDigest::from_digest(
            ReplaySurfaceAuthorityKind::Snapshot,
            digest_snapshot_surface(surface),
        )),
        Some(digest_snapshot_summary(surface)),
    )
}

pub(super) fn surface_basis_for_branch_head(
    commit: Option<&RelationalCommitReceipt>,
) -> ReplaySurfaceComparisonBasis {
    let basis = commit.cloned();
    ReplaySurfaceComparisonBasis::new(
        ReplaySurfaceAuthorityKind::BranchHead,
        Some(VerifiedReplaySurfaceDigest::from_digest(
            ReplaySurfaceAuthorityKind::BranchHead,
            digest_branch_head_surface(basis.as_ref()),
        )),
        Some(digest_branch_head_summary(basis.as_ref())),
    )
}

pub(super) fn surface_basis_for_published_lineage(
    published_lineage: &crate::lineage::data::PublishedLineageArtifact,
) -> ReplaySurfaceComparisonBasis {
    ReplaySurfaceComparisonBasis::new(
        ReplaySurfaceAuthorityKind::Lineage,
        Some(VerifiedReplaySurfaceDigest::from_digest(
            ReplaySurfaceAuthorityKind::Lineage,
            crate::replay::data::digest_lineage_event_batch_surface(published_lineage),
        )),
        Some(crate::replay::data::digest_lineage_event_summary(
            published_lineage.digest_basis().event_batch(),
        )),
    )
}

pub(super) fn surface_basis_for_strategy(
    strategy_artifacts: Option<&StrategyCommitArtifactBundle>,
) -> ReplaySurfaceComparisonBasis {
    ReplaySurfaceComparisonBasis::new(
        ReplaySurfaceAuthorityKind::Strategy,
        strategy_artifacts.map(|artifacts| {
            VerifiedReplaySurfaceDigest::from_digest(
                ReplaySurfaceAuthorityKind::Strategy,
                digest_strategy_replay_descriptor(artifacts.replay_descriptor()),
            )
        }),
        strategy_artifacts
            .map(|artifacts| digest_strategy_replay_summary(artifacts.replay_descriptor())),
    )
}
