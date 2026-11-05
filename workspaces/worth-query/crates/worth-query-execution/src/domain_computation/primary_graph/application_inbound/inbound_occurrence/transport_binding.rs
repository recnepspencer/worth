//! Exact installed route for completion observed on the outbound transport.

use worth_query_installation::facade::{ApplicationSchema, InstalledInboundOccurrenceContract};

use crate::domain_computation::vocabulary::execution_runtime::runtime_identity::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation;



#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryTransportCompletionBindingDenial {
    OutboxWithoutInboundContract,
    OutboxContractMismatch,
    OutboxOperationUnbound,
    VerifierMissing,
}

pub(in crate::domain_computation) trait TransportBindingScopedOps<Schema>:
    crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema>
where
    Schema: ApplicationSchema,
{
    fn resolve_installed_transport_completion_binding(
        &self,
        owner: &WorthQueryCommittedDispatchOutboxObservation,
    ) -> Result<
        WorthQueryInstalledTransportCompletionBinding,
        WorthQueryTransportCompletionBindingDenial,
    > {
        use WorthQueryTransportCompletionBindingDenial as Denial;
        let record = owner.record();
        let contract = record
            .inbound()
            .ok_or(Denial::OutboxWithoutInboundContract)?;
        if record.effect() != contract.effect()
            || record.protocol_identity() != contract.protocol().identity()
            || record.protocol_version() != contract.protocol().version()
        {
            return Err(Denial::OutboxContractMismatch);
        }
        let operation = record
            .operation_slot()
            .ok_or(Denial::OutboxOperationUnbound)?;
        let installed = self
            .inbound()
            .verifiers()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let verifier = installed.get(operation).ok_or(Denial::VerifierMissing)?;
        if verifier.contract != *contract {
            return Err(Denial::OutboxContractMismatch);
        }
        Ok(WorthQueryInstalledTransportCompletionBinding {
            operation: operation.to_owned(),
            audience: verifier.verifier.audience().to_owned(),
            contract: verifier.contract.clone(),
            runtime: self.authority_identity(),
        })
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema> + ?Sized> TransportBindingScopedOps<Schema> for HostRuntime where
    Schema: ApplicationSchema, {}

pub(crate) mod court_doors_for_test;
pub use court_doors_for_test::TransportBindingDoorsOps;
use crate::domain_computation::primary_graph::vocabulary::application_inbound::transport_binding::WorthQueryInstalledTransportCompletionBinding;
