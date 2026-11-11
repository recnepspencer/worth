//! Runtime installation of one verifier into its declared operation slot.

use std::collections::btree_map::Entry;
use std::sync::Arc;
use worth_foundational::facade::InboundSourceControl;

use worth_query_installation::facade::{
    ApplicationSchema, InstalledInboundOccurrenceContract, WorthQueryInstalledApplicationOperation,
};

use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::application_aftermath::WorthQueryInboundOccurrenceVerifier;
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;

pub(crate) mod receive;
pub(crate) mod transport_binding;
mod transport_publication_permit;
pub(in crate::domain_computation) use transport_binding::WorthQueryInstalledTransportCompletionBinding;
mod cleanup;
mod cost;
mod maintenance;
mod observe;
mod progress;
mod rebuild;
mod reconstruct;
mod recover;
pub use cost::WorthQueryInboundCostObservation;
pub use maintenance::WorthQueryInboundMaintenanceReport;
pub use observe::WorthQueryInboundTerminalObservation;
pub use rebuild::WorthQueryInboundIndexRepairDenial;

pub use receive::{
    WorthQueryAdmittedInboundOccurrence, WorthQueryAuthenticatedInboundOccurrence,
    WorthQueryCorrelatedInboundOccurrence, WorthQueryInboundAdmissionDenial,
    WorthQueryInboundAuthenticatedPermanentDenial, WorthQueryInboundPendingReason,
    WorthQueryInboundPermanentDenialKind, WorthQueryInboundReceipt,
    WorthQueryInboundReceiptPosture,
};

pub(in crate::domain_computation) struct WorthQueryInstalledInboundVerifier {
    pub(in crate::domain_computation) contract: InstalledInboundOccurrenceContract,
    pub(in crate::domain_computation) verifier: Arc<dyn WorthQueryInboundOccurrenceVerifier>,
    pub(in crate::domain_computation::primary_graph) cost: cost::WorthQueryInboundCostLedger,
    source_owner: std::sync::Weak<crate::domain_computation::application_aftermath::dispatch_journal::WorthQueryDispatchJournal>,
}

/// Admission state of one installed inbound source within this runtime.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInboundSourcePosture {
    Active,
    Retired,
    Revoked,
    /// A requested control has no proven physical fate yet; consumption is fenced.
    ControlPending,
}

impl WorthQueryInstalledInboundVerifier {
    pub(in crate::domain_computation) fn source_posture(&self) -> WorthQueryInboundSourcePosture {
        use crate::domain_computation::application_aftermath::dispatch_journal::QueryInboundSourceControlObservation;
        use WorthQueryInboundSourcePosture as Posture;
        match self
            .source_owner
            .upgrade()
            .map(|owner| owner.source_control(self.contract.source_identity()))
        {
            Some(QueryInboundSourceControlObservation::Recorded(None)) => Posture::Active,
            Some(QueryInboundSourceControlObservation::Recorded(Some(
                InboundSourceControl::Retired,
            ))) => Posture::Retired,
            Some(QueryInboundSourceControlObservation::Recorded(Some(
                InboundSourceControl::Revoked,
            ))) => Posture::Revoked,
            Some(QueryInboundSourceControlObservation::Pending) | None => Posture::ControlPending,
        }
    }
}

/// Runtime-issued route binding for one exact installed operation.
#[derive(Clone)]
pub struct WorthQueryInboundVerifierHandle {
    runtime: WorthQueryRuntimeAuthorityIdentity,
    operation: String,
    installed: Arc<WorthQueryInstalledInboundVerifier>,
}

impl WorthQueryInboundVerifierHandle {
    pub(in crate::domain_computation) fn operation(&self) -> &str {
        &self.operation
    }
}

/// Why a verifier cannot be bound to a declared operation in this runtime.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInboundVerifierInstallationDenial {
    ForeignSchemaBinding,
    NoDeclaredInbound,
    IncompatibleVerifier,
    AlreadyInstalled,
}

mod source_control;
pub use source_control::WorthQueryInboundSourceControlDenial;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    /// Install a product verifier for this operation's immutable inbound
    /// source/protocol contract. The returned handle selects the mechanism;
    /// incoming bytes cannot choose one.
    pub fn install_inbound_occurrence_verifier<Operation, Input>(
        &self,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        verifier: Arc<dyn WorthQueryInboundOccurrenceVerifier>,
    ) -> Result<WorthQueryInboundVerifierHandle, WorthQueryInboundVerifierInstallationDenial> {
        if operation.binding_identity() != &self.installed_schema.binding_identity() {
            return Err(WorthQueryInboundVerifierInstallationDenial::ForeignSchemaBinding);
        }
        let contract = operation
            .contracts()
            .external_effect()
            .inbound()
            .ok_or(WorthQueryInboundVerifierInstallationDenial::NoDeclaredInbound)?;
        if verifier.source_identity() != contract.source_identity()
            || verifier.protocol_identity() != contract.protocol().identity()
            || verifier.protocol_version() != contract.protocol().version()
        {
            return Err(WorthQueryInboundVerifierInstallationDenial::IncompatibleVerifier);
        }
        let mut installed = self
            .runtime
            .inbound_verifiers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let entry = match installed.entry(operation.operation().to_owned()) {
            Entry::Vacant(entry) => entry,
            Entry::Occupied(_) => {
                return Err(WorthQueryInboundVerifierInstallationDenial::AlreadyInstalled)
            }
        };
        let installed = Arc::new(WorthQueryInstalledInboundVerifier {
            contract: contract.clone(),
            verifier,
            cost: Default::default(),
            source_owner: Arc::downgrade(&self.runtime.dispatch_journal),
        });
        entry.insert(Arc::clone(&installed));
        Ok(WorthQueryInboundVerifierHandle {
            runtime: self.runtime.authority_identity(),
            operation: operation.operation().to_owned(),
            installed,
        })
    }

    pub(in crate::domain_computation) fn installed_inbound_verifier(
        &self,
        handle: &WorthQueryInboundVerifierHandle,
    ) -> Option<Arc<WorthQueryInstalledInboundVerifier>> {
        if handle.runtime != self.runtime.authority_identity() {
            return None;
        }
        let installed = self
            .runtime
            .inbound_verifiers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let current = installed.get(&handle.operation)?;
        Arc::ptr_eq(current, &handle.installed).then(|| Arc::clone(current))
    }

    pub(in crate::domain_computation) fn inbound_source_posture_for_operation(
        &self,
        operation: &str,
    ) -> Option<WorthQueryInboundSourcePosture> {
        self.runtime
            .inbound_verifiers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(operation)
            .map(|installed| installed.source_posture())
    }
}

mod publication_retry;
