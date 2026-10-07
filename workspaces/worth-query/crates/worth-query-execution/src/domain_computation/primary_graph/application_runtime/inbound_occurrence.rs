//! Runtime installation of one verifier into its declared operation slot.

use std::collections::btree_map::Entry;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

use worth_query_installation::facade::{
    ApplicationSchema, InstalledInboundOccurrenceContract, WorthQueryInstalledApplicationOperation,
};

use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::application_aftermath::WorthQueryInboundOccurrenceVerifier;
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;

mod receive;
mod transport_binding;
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
#[cfg(test)]
pub(in crate::domain_computation::primary_graph) use receive::WorthQueryInboundAdmission;
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
    source_posture: AtomicU8,
}

/// Admission state of one installed inbound source within this runtime.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInboundSourcePosture {
    Active,
    Retired,
    Revoked,
}

impl WorthQueryInstalledInboundVerifier {
    pub(in crate::domain_computation) fn source_posture(&self) -> WorthQueryInboundSourcePosture {
        match self.source_posture.load(Ordering::Acquire) {
            0 => WorthQueryInboundSourcePosture::Active,
            1 => WorthQueryInboundSourcePosture::Retired,
            _ => WorthQueryInboundSourcePosture::Revoked,
        }
    }

    fn retire(&self) {
        let _ = self
            .source_posture
            .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire);
    }

    fn revoke(&self) {
        self.source_posture.store(2, Ordering::Release);
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

/// Why source retirement or revocation could not select this installation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInboundSourceControlDenial {
    ForeignVerifier,
}

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
            source_posture: AtomicU8::new(0),
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
        self.inbound_verifiers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(operation)
            .map(|installed| installed.source_posture())
    }

    /// Stop admitting new source messages. Already accepted work remains
    /// owned by the aftermath custodian and may finish recovery.
    pub fn retire_inbound_occurrence_source(
        &self,
        handle: &WorthQueryInboundVerifierHandle,
    ) -> Result<(), WorthQueryInboundSourceControlDenial> {
        let installed = self
            .installed_inbound_verifier(handle)
            .ok_or(WorthQueryInboundSourceControlDenial::ForeignVerifier)?;
        installed.retire();
        Ok(())
    }

    /// Security revocation fences fresh consumption, including an accepted
    /// occurrence that has not yet reached World Performed. Custody stays live
    /// for explicit resolution rather than being silently discarded.
    pub fn revoke_inbound_occurrence_source(
        &self,
        handle: &WorthQueryInboundVerifierHandle,
    ) -> Result<(), WorthQueryInboundSourceControlDenial> {
        let installed = self
            .installed_inbound_verifier(handle)
            .ok_or(WorthQueryInboundSourceControlDenial::ForeignVerifier)?;
        installed.revoke();
        Ok(())
    }
}

mod publication_retry;
