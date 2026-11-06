//! Runtime installation of one verifier into its declared operation slot.

use std::collections::btree_map::Entry;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

use worth_query_installation::facade::{
    ApplicationSchema, InstalledInboundOccurrenceContract, WorthQueryInstalledApplicationOperation,
};

use crate::domain_computation::application_aftermath::WorthQueryInboundOccurrenceVerifier;
use crate::domain_computation::vocabulary::execution_runtime::runtime_identity::WorthQueryRuntimeAuthorityIdentity;

pub(crate) mod receive;
pub(crate) mod transport_binding;
pub(crate) mod transport_publication_permit;

pub(crate) mod cleanup;
mod cost;
mod maintenance;
mod observe;
pub(crate) mod progress;
mod rebuild;
pub(crate) mod reconstruct;
pub(crate) mod recover;
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

pub trait InboundOccurrenceOps<Schema>:
    crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema>
where
    Schema: ApplicationSchema,
{
    fn install_inbound_occurrence_verifier<Operation, Input>(
        &self,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        verifier: Arc<dyn WorthQueryInboundOccurrenceVerifier>,
    ) -> Result<WorthQueryInboundVerifierHandle, WorthQueryInboundVerifierInstallationDenial> {
        if operation.binding_identity() != &self.installed_schema().binding_identity() {
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
            .inbound()
            .verifiers()
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
            runtime: self.authority_identity(),
            operation: operation.operation().to_owned(),
            installed,
        })
    }

    fn retire_inbound_occurrence_source(
        &self,
        handle: &WorthQueryInboundVerifierHandle,
    ) -> Result<(), WorthQueryInboundSourceControlDenial> {
        let installed = self
            .installed_inbound_verifier(handle)
            .ok_or(WorthQueryInboundSourceControlDenial::ForeignVerifier)?;
        installed.retire();
        Ok(())
    }

    fn revoke_inbound_occurrence_source(
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

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema> + ?Sized> InboundOccurrenceOps<Schema> for HostRuntime where
    Schema: ApplicationSchema, {}

pub(in crate::domain_computation) trait InboundOccurrenceScopedOps<Schema>:
    crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema>
where
    Schema: ApplicationSchema,
{
    fn installed_inbound_verifier(
        &self,
        handle: &WorthQueryInboundVerifierHandle,
    ) -> Option<Arc<WorthQueryInstalledInboundVerifier>> {
        if handle.runtime != self.authority_identity() {
            return None;
        }
        let installed = self
            .inbound()
            .verifiers()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let current = installed.get(&handle.operation)?;
        Arc::ptr_eq(current, &handle.installed).then(|| Arc::clone(current))
    }

    fn inbound_source_posture_for_operation(
        &self,
        operation: &str,
    ) -> Option<WorthQueryInboundSourcePosture> {
        self.inbound()
            .verifiers()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(operation)
            .map(|installed| installed.source_posture())
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema> + ?Sized> InboundOccurrenceScopedOps<Schema> for HostRuntime where
    Schema: ApplicationSchema, {}

mod publication_retry;
pub use cleanup::CleanupCleanupDoorsOps;
pub use cleanup::CleanupDoorsOps;
pub use cleanup::CleanupOps;
pub(in crate::domain_computation::primary_graph) use cleanup::CleanupScopedOps;
pub use cost::CostOps;
pub(in crate::domain_computation::primary_graph) use cost::CostScopedOps;
pub use maintenance::MaintenanceOps;
pub use observe::ObserveOps;
pub use progress::ProgressDoorsOps;
pub(in crate::domain_computation) use progress::ProgressScopedOps;
pub use rebuild::RebuildOps;
pub(in crate::domain_computation) use rebuild::RebuildScopedOps;
pub use receive::AdmitDoorsOps;
pub(in crate::domain_computation) use receive::AdmitScopedOps;
pub use receive::PhasesOps;
pub use receive::ReceiveOps;
pub use reconstruct::ReconstructDoorsOps;
pub(in crate::domain_computation) use reconstruct::ReconstructScopedOps;
pub use recover::RecoverDoorsOps;
pub(in crate::domain_computation) use recover::RecoverScopedOps;
pub use transport_binding::TransportBindingDoorsOps;
pub(in crate::domain_computation) use transport_binding::TransportBindingScopedOps;
pub use transport_publication_permit::TransportPublicationPermitDoorsOps;
pub(in crate::domain_computation) use transport_publication_permit::TransportPublicationPermitScopedOps;
