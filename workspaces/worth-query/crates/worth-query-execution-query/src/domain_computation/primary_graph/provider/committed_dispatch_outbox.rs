//! Fresh Relational owner reads for committed dispatch-outbox rows.

use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;

pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::provider::committed_dispatch_outbox::correlation;
pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::provider::committed_dispatch_outbox::original_attempt;
pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::provider::committed_dispatch_outbox::original_row;
pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::provider::committed_dispatch_outbox::readmission;
pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::provider::committed_dispatch_outbox::recovered;
pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::provider::committed_dispatch_outbox::recovery;
pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::provider::committed_dispatch_outbox::restoration;

pub use worth_query_execution_lineage::facade::primary_graph::provider::committed_dispatch_outbox::{
    WorthQueryCommittedDispatchOutboxObservation,
    WorthQueryCommittedDispatchOutboxOwnerObservation,
    WorthQueryCommittedDispatchOutboxReadDenial,
    WorthQueryCommittedDispatchOutboxReadWork,
};

use WorthQueryCommittedDispatchOutboxReadDenial as Denial;

impl<Schema> super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    /// Uses the correlation only to select a candidate. The provider then
    /// re-reads that candidate from the exact retained Relational commit.
    pub fn observe_committed_dispatch_outbox_for_correlation(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        correlation: &crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity,
    ) -> Result<WorthQueryCommittedDispatchOutboxObservation, Denial> {
        self.runtime
            .primary_provider
            .committed_dispatch_outbox_for_correlation(
                execution,
                correlation,
                &self.runtime.product_runtime.owner_ref().observation_port(),
                &self.runtime.product_runtime.owner_ref().inspection_port(),
            )
    }

    /// Selects a candidate by correlation and verifies its exact original
    /// Native row and World publication. This is read-only owner evidence;
    /// it grants neither dispatch nor fresh publication delivery.
    pub fn observe_committed_dispatch_outbox_by_correlation(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        correlation: &crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity,
    ) -> Result<WorthQueryCommittedDispatchOutboxObservation, Denial> {
        let execution = phase
            .execution_request_for(&self.runtime.product_runtime)
            .map_err(|_| Denial::ForeignPhase)?;
        self.observe_committed_dispatch_outbox_for_correlation(&execution, correlation)
    }

    /// Reads this receipt's outbox from a fresh provider-owned Relational view.
    pub fn observe_committed_dispatch_outbox(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<Option<WorthQueryCommittedDispatchOutboxObservation>, Denial> {
        let phase_request = phase
            .execution_request_for(&self.runtime.product_runtime)
            .map_err(|_| Denial::ForeignPhase)?;
        self.observe_committed_dispatch_outbox_with_request(&phase_request, receipt)
    }

    pub fn observe_committed_dispatch_outbox_with_request(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<Option<WorthQueryCommittedDispatchOutboxObservation>, Denial> {
        self.runtime
            .primary_provider
            .committed_dispatch_outbox(execution, receipt)
    }
}

pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::provider::committed_dispatch_outbox::record_read;
