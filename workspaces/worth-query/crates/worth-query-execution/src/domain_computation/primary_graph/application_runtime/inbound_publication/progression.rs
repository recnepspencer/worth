use std::sync::atomic::Ordering;
use std::sync::Arc;

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::RuntimeWorldPublicationOutcome;

use super::outcome::{
    WorthQueryInboundPublicationDenial as Denial, WorthQueryInboundPublicationOutcome as Outcome,
    WorthQueryPerformedInboundCompletion, WorthQueryUnpublishedInboundCompletion,
};
use crate::domain_computation::application_aftermath::WorthQueryAcceptedInboundOccurrence;
use crate::domain_computation::execution_runtime::product_world::WorthQueryReservedProductPublicationReceipt;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::WorthQueryProductUnpublishedApplication;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Progress one accepted occurrence at the current head of the exact
    /// incarnation that issued its dispatch. Only World Performed may become
    /// terminal completion; an unpublished Relational effect stays in custody.
    pub(in crate::domain_computation) fn publish_inbound_completion(
        &self,
        accepted: Arc<WorthQueryAcceptedInboundOccurrence>,
        request: &WorthQueryRequestScope,
    ) -> Outcome {
        let owner = accepted.owner();
        let cost = self
            .inbound_cost_for_operation(accepted.operation())
            .expect("accepted occurrence retains its installed operation");
        let original = owner.committed_product_publication();
        let incarnation = original.product_incarnation();
        let runtime_instance = self
            .product_runtime
            .source
            .authoritative_source_profile()
            .runtime_instance_id();
        if owner.relational_runtime_instance_id() != runtime_instance {
            return Outcome::Denied(Denial::ForeignRelationalRuntime);
        }
        if original.product_branch().owner_identity() != self.product_runtime.owner.owner_identity()
        {
            return Outcome::Denied(Denial::ForeignProductWorld);
        }
        if owner.commit_reference() != original.relational_commit() {
            return Outcome::Denied(Denial::OriginalPublicationCommitMismatch);
        }

        // Ordinary application commits and branch close share this exact
        // incarnation lane, so none can pass between basis admission and the
        // completion's World publication attempt.
        let lane = self
            .primary_provider
            .application_branch_commit_lane_for_occurrence(incarnation);
        let lane = match lane {
            Ok(lane) => lane,
            Err(_) => return Outcome::Denied(Denial::BranchCoordinationCapacityExhausted),
        };
        let _coordination = lane.enter();
        // A transport completion may have won after this signed occurrence
        // entered custody but before it acquired the shared incarnation lane.
        // Recheck canonical terminal ownership here before creating a second
        // completion mutation.
        cost.cost
            .terminal_key_probes
            .fetch_add(1, Ordering::Relaxed);
        match self
            .primary_provider
            .lookup_completed_inbound(accepted.owner().record().correlation())
        {
            Ok(Some(terminal))
                if terminal.matches_effect(accepted.operation(), accepted.claims()) =>
            {
                return Outcome::AlreadyCompleted;
            }
            Ok(Some(_)) => return Outcome::Denied(Denial::CorrelationAlreadyOwned),
            Err(_) => return Outcome::Denied(Denial::TerminalIndexUnavailable),
            Ok(None) => {}
        }
        let lease = match self.product_runtime.admit_product_occurrence(incarnation) {
            Ok(lease) => lease,
            Err(_) => return Outcome::Denied(Denial::ProductAdmission),
        };
        cost.cost
            .completion_candidate_prepares
            .fetch_add(1, Ordering::Relaxed);
        let candidate = match self
            .primary_provider
            .prepare_inbound_completion_candidate(lease.relational_basis(), &accepted)
        {
            Ok(candidate) => candidate,
            Err(_) => return Outcome::Denied(Denial::CompletionPreparation),
        };
        let binding = lease.publication_binding();
        let recovery = binding.recovery();
        let prepared = match binding.prepare_relational_candidate(candidate, request, false) {
            Ok(prepared) => prepared,
            Err(_) => return Outcome::NoEffect,
        };
        let terminal = WorthQueryReservedProductPublicationReceipt::new(
            binding.root_identity(),
            prepared.unpublished_recovery_handle(),
            false,
            false,
            false,
        );
        let correlation = *accepted.owner().record().correlation();
        self.primary_provider
            .mark_inbound_completion_publication_pending(&correlation);
        cost.cost
            .world_publication_attempts
            .fetch_add(1, Ordering::Relaxed);
        match prepared.execute() {
            RuntimeWorldPublicationOutcome::Performed(publication) => {
                cost.cost
                    .world_performed_publications
                    .fetch_add(1, Ordering::Relaxed);
                Outcome::Performed(WorthQueryPerformedInboundCompletion::new(
                    accepted,
                    incarnation,
                    terminal.fill(publication.consume(), None),
                ))
            }
            RuntimeWorldPublicationOutcome::NoEffect(_) => {
                self.primary_provider
                    .clear_inbound_completion_publication_pending(&correlation);
                Outcome::NoEffect
            }
            RuntimeWorldPublicationOutcome::ProductUnpublished(effects) => {
                Outcome::ProductUnpublished(WorthQueryUnpublishedInboundCompletion::new(
                    accepted,
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
