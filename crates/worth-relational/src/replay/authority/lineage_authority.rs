use std::collections::BTreeSet;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyOwnerPort as _;

use crate::lineage::data::PublishedLineageArtifact;
use crate::performance::ReplayLineageAuthorityIndexedSource;
use crate::replay::data::{
    digest_lineage_decision_log_surface, digest_lineage_decision_summary,
    digest_lineage_event_batch_surface, digest_lineage_event_summary, CanonicalCommitEnvelope,
    CertifiedLineageSurfaceComparisonBasis, CertifiedLineageSurfaceDigest,
    LineageCertifiedSurfaceKind, ReplayAuthorityBasisKind,
};
use crate::runtime::RelationalRuntime;

use super::SelectedPublishedLineageAuthority;

pub(super) fn published_lineage_artifacts_match(
    expected: &PublishedLineageArtifact,
    observed: &PublishedLineageArtifact,
) -> bool {
    if expected.branch_id() != observed.branch_id()
        || expected.lineage_event_ids() != observed.lineage_event_ids()
        || expected.lineage_events() != observed.lineage_events()
        || expected.digest_basis() != observed.digest_basis()
        || expected.observed_event_batch_digest_basis()
            != observed.observed_event_batch_digest_basis()
        || expected.observed_decision_log_digest_basis()
            != observed.observed_decision_log_digest_basis()
        || expected.counters() != observed.counters()
        || expected.lineage_decision_log() != observed.lineage_decision_log()
    {
        return false;
    }

    let event_ids = expected
        .lineage_event_ids()
        .iter()
        .copied()
        .chain(
            expected
                .lineage_decision_log()
                .iter()
                .chain(observed.lineage_decision_log().iter())
                .filter_map(|decision| decision.event_id),
        )
        .collect::<BTreeSet<u64>>();
    for event_id in event_ids {
        if expected
            .decisions_for_event_id(event_id)
            .collect::<Vec<_>>()
            != observed
                .decisions_for_event_id(event_id)
                .collect::<Vec<_>>()
        {
            return false;
        }
    }

    true
}

pub(super) fn lineage_event_batch_comparison_basis(
    published_lineage: &PublishedLineageArtifact,
) -> CertifiedLineageSurfaceComparisonBasis {
    CertifiedLineageSurfaceComparisonBasis::new(
        LineageCertifiedSurfaceKind::EventBatch,
        Some(CertifiedLineageSurfaceDigest::from_digest(
            LineageCertifiedSurfaceKind::EventBatch,
            digest_lineage_event_batch_surface(published_lineage),
        )),
        Some(digest_lineage_event_summary(
            published_lineage.event_batch_digest_basis(),
        )),
    )
}

pub(super) fn lineage_decision_log_comparison_basis(
    published_lineage: &PublishedLineageArtifact,
) -> CertifiedLineageSurfaceComparisonBasis {
    CertifiedLineageSurfaceComparisonBasis::new(
        LineageCertifiedSurfaceKind::DecisionLog,
        Some(CertifiedLineageSurfaceDigest::from_digest(
            LineageCertifiedSurfaceKind::DecisionLog,
            digest_lineage_decision_log_surface(published_lineage),
        )),
        Some(digest_lineage_decision_summary(
            published_lineage.decision_log_digest_basis(),
        )),
    )
}

pub(super) fn select_published_lineage_authority(
    runtime: &RelationalRuntime,
    envelope: &CanonicalCommitEnvelope,
) -> SelectedPublishedLineageAuthority {
    if let Some(artifact) = runtime
        .durability
        .durable_log_envelope(envelope.commit.commit_id)
        .map(|candidate| candidate.published_lineage().clone())
    {
        SelectedPublishedLineageAuthority {
            kind: ReplayAuthorityBasisKind::DurableLogCanonical,
            indexed_source: Some(ReplayLineageAuthorityIndexedSource::DurableLog),
            artifact,
        }
    } else if let Some(artifact) = runtime
        .durability
        .checkpoint_envelope(envelope.commit.commit_id)
        .map(|candidate| candidate.published_lineage().clone())
    {
        SelectedPublishedLineageAuthority {
            kind: ReplayAuthorityBasisKind::DurableLogCanonical,
            indexed_source: Some(ReplayLineageAuthorityIndexedSource::Checkpoint),
            artifact,
        }
    } else {
        SelectedPublishedLineageAuthority {
            kind: ReplayAuthorityBasisKind::RetainedEnvelopeCanonical,
            indexed_source: None,
            artifact: envelope.published_lineage().clone(),
        }
    }
}
