use std::sync::Arc;

use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::{
    WorthQueryDecisionFactAdmission, WorthQueryDecisionFactComparisonAdmission,
    WorthQueryDecisionFactComparisonEvidence, WorthQueryDecisionFactEvidence,
    WorthQueryDecisionFactEvidenceView, WorthQueryDecisionFactProvider,
    WorthQueryDecisionFactRequestView, WorthQueryDecisionReadSetFailure,
    WorthQueryProviderSessionView,
};

impl WorthQueryDecisionFactProvider for Arc<WorthQueryPrimaryGraphProvider> {
    fn observe_decision_fact(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        request: WorthQueryDecisionFactRequestView<'_>,
        admission: WorthQueryDecisionFactAdmission,
    ) -> Result<WorthQueryDecisionFactEvidence, WorthQueryDecisionReadSetFailure> {
        let observed = self
            .attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .contains_observed_fact(session, request.locator().identity());
        if !observed {
            return Err(provider_rejected());
        }
        admission.observe(format!(
            "application-observed:{}",
            request.locator().identity()
        ))
    }

    fn compare_decision_fact(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        evidence: WorthQueryDecisionFactEvidenceView<'_>,
        admission: WorthQueryDecisionFactComparisonAdmission,
    ) -> Result<WorthQueryDecisionFactComparisonEvidence, WorthQueryDecisionReadSetFailure> {
        let fact_basis = {
            self.attempts
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .observed_fact_and_product(session, evidence.locator().identity())
                .ok_or_else(provider_rejected)?
        };
        let fresh = self
            .graph
            .with_runtime_mut(|runtime| fact_basis.remains_equal_in(runtime))
            .map_err(snapshot_read_set_failure)?;
        admission.observe_current_version(if fresh {
            evidence.physical_version_evidence().to_owned()
        } else {
            format!("application-stale:{}", evidence.locator().identity())
        })
    }
}

fn snapshot_read_set_failure(
    denial: crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial,
) -> WorthQueryDecisionReadSetFailure {
    let kind = match denial {
        crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => crate::domain_computation::WorthQueryDecisionReadSetDenialKind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        },
        crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::RetentionCapacityExhausted => {
            crate::domain_computation::WorthQueryDecisionReadSetDenialKind::RetentionCapacityExhausted
        }
        crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::RetentionIdentityExhausted => {
            crate::domain_computation::WorthQueryDecisionReadSetDenialKind::RetentionIdentityExhausted
        }
        crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::SnapshotIdentityExhausted => {
            crate::domain_computation::WorthQueryDecisionReadSetDenialKind::SnapshotIdentityExhausted
        }
        _ => crate::domain_computation::WorthQueryDecisionReadSetDenialKind::ProviderRejected,
    };
    WorthQueryDecisionReadSetFailure::new(
        kind,
        "primary provider could not open the exact decision-fact basis",
    )
}

fn provider_rejected() -> WorthQueryDecisionReadSetFailure {
    WorthQueryDecisionReadSetFailure::new(
        crate::domain_computation::WorthQueryDecisionReadSetDenialKind::ProviderRejected,
        "primary provider has no exact session-owned application fact",
    )
}
