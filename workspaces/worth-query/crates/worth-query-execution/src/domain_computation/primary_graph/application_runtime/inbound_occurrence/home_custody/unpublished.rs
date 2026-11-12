//! Actual recovered completion effects join their original journal acceptance or transport binding.
use super::*;
use crate::domain_computation::primary_graph::application_runtime::inbound_publication::recovered::RecordedCompletionContinuation;
use crate::domain_computation::primary_graph::provider::inbound_completion::WorthQueryCompletionProvenance;
use crate::domain_computation::primary_graph::provider::WorthQueryCanonicalCompletionRow;
use worth_runtime_world::facade::WorldRecoveredUnpublishedEffects;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn admit_recovered_inbound_completion(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        effects: &WorldRecoveredUnpublishedEffects,
        entity: worth_relational::facade::identity::EntityId,
        row: WorthQueryCanonicalCompletionRow,
    ) -> Result<(), Denial> {
        let owner = self
            .observe_committed_dispatch_outbox_for_correlation(execution, &row.correlation)
            .map_err(Denial::OwnerReadDenied)?;
        let publication = owner.committed_product_publication();
        if !row.matches_original(&owner)
            || effects.expected_head().branch_identity() != publication.product_branch()
            || effects.expected_head().lifecycle_incarnation() != publication.product_incarnation()
        {
            return Err(Denial::ForeignOwner);
        }
        let continuation = RecordedCompletionContinuation::from_original(effects, entity)
            .ok_or(Denial::RecoveryUnavailable)?;
        match &row.provenance {
            WorthQueryCompletionProvenance::AuthenticatedInbound => {
                let mut custody = self
                    .runtime
                    .inbound_custody
                    .lock()
                    .unwrap_or_else(|p| p.into_inner());
                let accepted = custody
                    .accepted_by_correlation(&row.correlation)
                    .ok_or(Denial::UnknownCorrelation)?;
                let claims = accepted.claims();
                if row.operation != accepted.operation()
                    || row.audience != claims.audience
                    || row.expires_at != Some(claims.expires_at_unix_seconds)
                    || row.key_epoch != Some(claims.key_epoch)
                    || row.message_identity != Some(claims.message_identity)
                    || row.signed_meaning_digest != Some(claims.signed_meaning_digest)
                {
                    return Err(Denial::IncompatibleMeaning);
                }
                if !custody.readmit_unpublished_completion(&accepted, continuation) {
                    return Err(Denial::RecoveryUnavailable);
                }
            }
            WorthQueryCompletionProvenance::InstalledTransportCompletion {
                attempt_identity,
                observation_identity,
            } => {
                let binding = self
                    .runtime
                    .dispatch_journal
                    .recorded_transport_binding(&owner)
                    .map_err(|_| Denial::UnsupportedOutbox)?
                    .ok_or(Denial::UnsupportedOutbox)?;
                if binding.audience.as_deref() != Some(row.audience.as_str()) {
                    return Err(Denial::IncompatibleMeaning);
                }
                self.readmit_unpublished_transport_completion(
                    &row.correlation,
                    attempt_identity,
                    observation_identity,
                    continuation,
                )?;
            }
        }
        self.runtime
            .primary_provider
            .mark_inbound_completion_publication_pending(&row.correlation);
        Ok(())
    }
}
