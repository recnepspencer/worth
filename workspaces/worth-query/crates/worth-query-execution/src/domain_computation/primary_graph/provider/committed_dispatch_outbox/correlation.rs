//! Correlation selects owner evidence; the exact Relational row and World publication decide authority.

use super::{Denial, WorthQueryCommittedDispatchOutboxObservation};
use crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity;
use crate::domain_computation::primary_graph::provider::{
    outstanding_dispatch::OutstandingDispatchPosture, WorthQueryPrimaryGraphProvider,
};

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn committed_dispatch_outbox_for_correlation(
        &self,
        correlation: &ExternalEffectCorrelationIdentity,
    ) -> Result<WorthQueryCommittedDispatchOutboxObservation, Denial> {
        let outstanding = self.outstanding_dispatch.posture(correlation);
        if matches!(
            outstanding,
            Some(OutstandingDispatchPosture::PendingPublication)
        ) {
            return Err(Denial::PendingPublication);
        }
        let committed = self
            .completed_commit_evidence
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .observe_correlation(correlation)
            .map_err(|()| Denial::AmbiguousCorrelation)?;
        let Some(committed) = committed else {
            return match outstanding {
                Some(OutstandingDispatchPosture::Committed(_)) => {
                    Err(Denial::CommittedIndexUnavailable)
                }
                _ => Err(Denial::Missing),
            };
        };
        if let Some(OutstandingDispatchPosture::Committed(expected)) = outstanding {
            if committed.commit_reference().commit_id != expected {
                return Err(Denial::CommitMismatch);
            }
        }
        let binding = committed
            .commit_evidence()
            .committed_dispatch_outbox()
            .seal_for_receipt()
            .map_err(|_| Denial::NotAuthoritative)?
            .into_binding()
            .ok_or(Denial::Missing)?;
        if binding.record().correlation() != correlation {
            return Err(Denial::RecordMismatch);
        }
        let owner = self.observe_expected(
            &binding,
            committed.commit_reference(),
            committed.runtime_instance_id(),
        )?;
        let publication = committed.committed_product_publication().clone();
        if owner.commit_reference() != publication.relational_commit() {
            return Err(Denial::CommitMismatch);
        }
        Ok(WorthQueryCommittedDispatchOutboxObservation::seal(
            owner,
            publication,
        ))
    }
}
