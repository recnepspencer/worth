//! World publication from one installed transport's observed completion.
use crate::domain_computation::primary_graph::provider::WorthQueryInboundCompletionPreparationDenial as Preparation;
use worth_relational::facade::mvcc::PreparedRelationalCommitCandidate;

use std::sync::Arc;

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_runtime_world::facade::RuntimeWorldPublicationOutcome;

use super::installed_transport::{
    InstalledTransportCompletion, InstalledTransportPublicationDenial as Denial,
    InstalledTransportPublicationOutcome as Outcome, PerformedInstalledTransportCompletion,
    UnpublishedInstalledTransportCompletion,
};
use crate::domain_computation::execution_runtime::product_world::WorthQueryReservedProductPublicationReceipt;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::WorthQueryProductUnpublishedApplication;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    /// Continue one real completed dispatch into the same Relational + World
    /// terminal owner used by authenticated inbound completion. This consumes
    /// no signature claim and keeps the actual dispatch proof in every result.
    pub(in crate::domain_computation::primary_graph) fn publish_installed_transport_completion(
        &self,
        evidence: Arc<InstalledTransportCompletion>,
        request: &WorthQueryRequestScope,
    ) -> Outcome {
        let owner = evidence.committed();
        let original = owner.committed_product_publication();
        let incarnation = original.product_incarnation();
        let runtime_instance = self
            .product_runtime
            .source
            .authoritative_source_profile()
            .runtime_instance_id();
        if owner.relational_runtime_instance_id() != runtime_instance {
            return Outcome::Denied(evidence, Denial::ForeignRelationalRuntime);
        }
        if original.product_branch().owner_identity() != self.product_runtime.owner.owner_identity()
        {
            return Outcome::Denied(evidence, Denial::ForeignProductWorld);
        }
        if owner.commit_reference() != original.relational_commit() {
            return Outcome::Denied(evidence, Denial::OriginalPublicationCommitMismatch);
        }
        let binding = match self.resolve_installed_transport_completion_binding(owner) {
            Ok(binding) => binding,
            Err(_) => return Outcome::Denied(evidence, Denial::BindingUnavailable),
        };
        let lane = self
            .primary_provider
            .application_branch_commit_lane_for_occurrence(incarnation);
        let lane = match lane {
            Ok(lane) => lane,
            Err(_) => {
                return Outcome::Denied(evidence, Denial::BranchCoordinationCapacityExhausted)
            }
        };
        let _coordination = lane.enter();
        match self
            .primary_provider
            .lookup_completed_inbound(owner.record().correlation())
        {
            Ok(Some(terminal)) if terminal.matches_transport_observation(&evidence, &binding) => {
                return Outcome::AlreadyCompleted;
            }
            Ok(Some(_)) => return Outcome::Denied(evidence, Denial::CorrelationAlreadyOwned),
            Err(_) => return Outcome::Denied(evidence, Denial::TerminalIndexUnavailable),
            Ok(None) => {}
        }
        let _publication_permit = match self.reserve_installed_transport_publication(&binding) {
            Ok(permit) => permit,
            Err(_) => return Outcome::Denied(evidence, Denial::PublicationPermit),
        };
        let lease = match self.product_runtime.admit_product_occurrence(incarnation) {
            Ok(lease) => lease,
            Err(_) => return Outcome::Denied(evidence, Denial::ProductAdmission),
        };
        let candidate = match completion_candidate(
            Arc::clone(&evidence),
            self.primary_provider
                .prepare_installed_transport_completion_candidate(
                    lease.relational_basis(),
                    owner,
                    evidence.dispatch(),
                    &binding,
                ),
        ) {
            Ok(candidate) => candidate,
            Err(outcome) => return outcome,
        };
        let publication = lease.publication_binding();
        let recovery = publication.recovery();
        let prepared = match publication.prepare_relational_candidate(candidate, request, false) {
            Ok(prepared) => prepared,
            Err(_) => return Outcome::NoEffect(evidence),
        };
        let terminal = WorthQueryReservedProductPublicationReceipt::new(
            publication.root_identity(),
            prepared.unpublished_recovery_handle(),
            false,
            false,
            false,
        );
        let correlation = *owner.record().correlation();
        self.primary_provider
            .mark_inbound_completion_publication_pending(&correlation);
        match prepared.execute() {
            RuntimeWorldPublicationOutcome::Performed(performed) => {
                Outcome::Performed(PerformedInstalledTransportCompletion::new(
                    evidence,
                    binding,
                    incarnation,
                    terminal.fill(performed.consume(), None),
                ))
            }
            RuntimeWorldPublicationOutcome::NoEffect(_) => {
                self.primary_provider
                    .clear_inbound_completion_publication_pending(&correlation);
                Outcome::NoEffect(evidence)
            }
            RuntimeWorldPublicationOutcome::ProductUnpublished(effects) => {
                Outcome::ProductUnpublished(UnpublishedInstalledTransportCompletion::new(
                    evidence,
                    WorthQueryProductUnpublishedApplication::new(
                        effects,
                        recovery,
                        self.primary_provider.unpublished_idempotency_disposition(),
                    ),
                ))
            }
        }
    }
}

// Both entry routes keep HEAD's preparation-refusal posture. The complete
// refusal travels with the publication outcome rather than becoming a string.
pub(in crate::domain_computation::primary_graph) fn completion_candidate(
    evidence: Arc<InstalledTransportCompletion>,
    result: Result<PreparedRelationalCommitCandidate, Preparation>,
) -> Result<PreparedRelationalCommitCandidate, Outcome> {
    match result {
        Ok(candidate) => Ok(candidate),
        Err(
            denial @ (Preparation::ExecutionDenied { .. }
            | Preparation::ExecutionControlStopped { .. }
            | Preparation::OriginalOutboxNotAnEntity
            | Preparation::ForeignOrStaleBasis
            | Preparation::AllocationDenied { .. }
            | Preparation::StagingAllocationDenied { .. }
            | Preparation::StagingCardinalityOverflow
            | Preparation::StagingInputDirectoryAllocationDenied { .. }
            | Preparation::StagingUnavailable
            | Preparation::ValidationUnavailable
            | Preparation::PreparationUnavailable
            | Preparation::SnapshotUnavailable
            | Preparation::IndexPreparationUnavailable
            | Preparation::InstalledBindingMismatch
            | Preparation::DispatchOwnerMismatch),
        ) => Err(Outcome::Denied(
            evidence,
            Denial::CompletionPreparation(denial),
        )),
    }
}
