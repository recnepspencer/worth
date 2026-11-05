//! The runtime's consumer methods over `domain_computation::primary_graph::application_inbound`; each only calls its subsystem.
use crate::domain_computation::vocabulary::application_aftermath::correlation::ExternalEffectCorrelationIdentity;
use crate::domain_computation::application_aftermath::WorthQueryAcceptedInboundOccurrence;
use crate::domain_computation::application_aftermath::WorthQueryInboundCleanupReport;
use crate::domain_computation::application_aftermath::WorthQueryInboundCustodyAdmission;
use crate::domain_computation::application_aftermath::WorthQueryInboundOccurrenceClaims;
use crate::domain_computation::application_aftermath::WorthQueryInboundOccurrenceVerifier;
use crate::domain_computation::application_aftermath::WorthQueryInboundPublicationClaim;
use crate::domain_computation::application_aftermath::WorthQueryInboundVerificationDenial;
use crate::domain_computation::vocabulary::execution_runtime::runtime_identity::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::application_inbound::inbound_occurrence::receive::WorthQueryInboundAdmission;
use crate::domain_computation::primary_graph::application_inbound::inbound_occurrence::receive::WorthQueryInboundAdmissionDenial as Denial;
use crate::domain_computation::primary_graph::application_inbound::inbound_occurrence::receive::WorthQueryInboundReceipt;
use crate::domain_computation::primary_graph::application_inbound::inbound_occurrence::receive::WorthQueryInboundReceiptPosture as Posture;
use crate::domain_computation::primary_graph::application_inbound::inbound_occurrence::WorthQueryAuthenticatedInboundOccurrence;
use crate::domain_computation::primary_graph::application_inbound::inbound_occurrence::WorthQueryInboundAdmissionDenial;
use crate::domain_computation::primary_graph::application_inbound::inbound_occurrence::WorthQueryInboundCostObservation;
use crate::domain_computation::primary_graph::application_inbound::inbound_occurrence::WorthQueryInboundIndexRepairDenial;
use crate::domain_computation::primary_graph::application_inbound::inbound_occurrence::WorthQueryInboundMaintenanceReport;
use crate::domain_computation::primary_graph::application_inbound::inbound_occurrence::WorthQueryInboundSourceControlDenial;
use crate::domain_computation::primary_graph::application_inbound::inbound_occurrence::WorthQueryInboundSourcePosture;
use crate::domain_computation::primary_graph::application_inbound::inbound_occurrence::WorthQueryInboundTerminalObservation;
use crate::domain_computation::primary_graph::application_inbound::inbound_occurrence::WorthQueryInboundVerifierHandle;
use crate::domain_computation::primary_graph::application_inbound::inbound_occurrence::WorthQueryInboundVerifierInstallationDenial;
use crate::domain_computation::primary_graph::application_inbound::inbound_occurrence::WorthQueryInstalledInboundVerifier;
use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::provider::WorthQueryCanonicalInboundCompletion;
use crate::domain_computation::primary_graph::provider::WorthQueryInboundCompletionReadDenial;
use crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation;
use crate::domain_computation::vocabulary::primary_graph::committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxReadDenial;
use std::collections::btree_map::Entry;
use std::num::NonZeroUsize;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use worth_foundational::facade::AspectValue;
use worth_foundational::facade::CanonicalDigestId;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline;
use worth_query_installation::facade::ApplicationSchema;
use worth_query_installation::facade::InstalledInboundOccurrenceContract;
use worth_query_installation::facade::WorthQueryInstalledApplicationOperation;
use worth_runtime_world::facade::CompositeCommitIdentity;
use worth_runtime_world::facade::CompositeComponentChangePosture;
use worth_runtime_world::facade::CompositePublicationAttemptIdentity;
use worth_runtime_world::facade::RuntimeWorldPublicationRow;
impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn install_inbound_occurrence_verifier<Operation, Input>(
        &self,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        verifier: Arc<dyn WorthQueryInboundOccurrenceVerifier>,
    ) -> Result<WorthQueryInboundVerifierHandle, WorthQueryInboundVerifierInstallationDenial> {
        <Self as crate::domain_computation::primary_graph::application_inbound::InboundOccurrenceOps<Schema>>::install_inbound_occurrence_verifier(self, operation, verifier)
    }

    pub fn retire_inbound_occurrence_source(
        &self,
        handle: &WorthQueryInboundVerifierHandle,
    ) -> Result<(), WorthQueryInboundSourceControlDenial> {
        <Self as crate::domain_computation::primary_graph::application_inbound::InboundOccurrenceOps<Schema>>::retire_inbound_occurrence_source(self, handle)
    }

    pub fn revoke_inbound_occurrence_source(
        &self,
        handle: &WorthQueryInboundVerifierHandle,
    ) -> Result<(), WorthQueryInboundSourceControlDenial> {
        <Self as crate::domain_computation::primary_graph::application_inbound::InboundOccurrenceOps<Schema>>::revoke_inbound_occurrence_source(self, handle)
    }

    pub fn cleanup_completed_inbound_occurrences(
        &self,
        handle: &WorthQueryInboundVerifierHandle,
        maximum_work: NonZeroUsize,
    ) -> Result<WorthQueryInboundCleanupReport, Denial> {
        <Self as crate::domain_computation::primary_graph::application_inbound::CleanupOps<
            Schema,
        >>::cleanup_completed_inbound_occurrences(self, handle, maximum_work)
    }

    pub fn observe_inbound_cost(
        &self,
        handle: &WorthQueryInboundVerifierHandle,
    ) -> Option<WorthQueryInboundCostObservation> {
        <Self as crate::domain_computation::primary_graph::application_inbound::CostOps<Schema>>::observe_inbound_cost(self, handle)
    }

    pub fn maintain_inbound_occurrences(
        &self,
        handle: &WorthQueryInboundVerifierHandle,
        maximum_work: NonZeroUsize,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryInboundMaintenanceReport, Denial> {
        <Self as crate::domain_computation::primary_graph::application_inbound::MaintenanceOps<
            Schema,
        >>::maintain_inbound_occurrences(self, handle, maximum_work, request)
    }

    pub fn observe_inbound_terminal(
        &self,
        handle: &WorthQueryInboundVerifierHandle,
        correlation_token: [u8; 32],
    ) -> Option<WorthQueryInboundTerminalObservation> {
        <Self as crate::domain_computation::primary_graph::application_inbound::ObserveOps<
            Schema,
        >>::observe_inbound_terminal(self, handle, correlation_token)
    }

    pub fn repair_completed_inbound_index(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        handle: &WorthQueryInboundVerifierHandle,
        maximum_world_slots: NonZeroUsize,
        maximum_changed_records_per_commit: NonZeroUsize,
        maximum_ancestry_commits_per_completion: NonZeroUsize,
    ) -> Result<bool, WorthQueryInboundIndexRepairDenial> {
        <Self as crate::domain_computation::primary_graph::application_inbound::RebuildOps<
            Schema,
        >>::repair_completed_inbound_index(
            self,
            phase,
            handle,
            maximum_world_slots,
            maximum_changed_records_per_commit,
            maximum_ancestry_commits_per_completion,
        )
    }

    pub fn receive_inbound_occurrence(
        &self,

        handle: &WorthQueryInboundVerifierHandle,
        envelope: &[u8],
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryInboundReceipt, WorthQueryInboundAdmissionDenial> {
        <Self as crate::domain_computation::primary_graph::application_inbound::ReceiveOps<
            Schema,
        >>::receive_inbound_occurrence(self, handle, envelope, request)
    }

    pub fn authenticate_inbound_occurrence<'a>(
        &'a self,
        handle: &WorthQueryInboundVerifierHandle,
        envelope: &'a [u8],
    ) -> Result<WorthQueryAuthenticatedInboundOccurrence<'a, Schema>, Denial> {
        <Self as crate::domain_computation::primary_graph::application_inbound::PhasesOps<Schema>>::authenticate_inbound_occurrence(self, handle, envelope)
    }
}
