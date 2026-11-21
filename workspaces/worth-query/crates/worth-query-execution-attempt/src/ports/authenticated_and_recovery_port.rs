//! Crate-boundary ports for authenticated principals, inbound receipts, and
//! recovery-authority minting. Never exported on `worth-query-execution::facade`
//! or Host.

use std::sync::Arc;

use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_installation::facade::WorthQueryCanonicalWorkPhases;
use worth_runtime_world::facade::RuntimeWorldOwnerIdentity;

use crate::domain_computation::application_aftermath::external_effect::inbound::{
    WorthQueryAcceptedInboundOccurrence, WorthQueryInboundOccurrenceClaims,
};
use crate::domain_computation::application_aftermath::recovery_handle::identity::WorthQueryRecoveryHandleIdentity;
use crate::domain_computation::application_aftermath::recovery_handle::{
    WorthQueryRecoveryHandle, WorthQueryRecoveryHandleAuthorityIdentity,
    WorthQueryRecoveryHandleBinding, WorthQueryRecoveryHandleDenial,
};
use crate::domain_computation::application_aftermath::recovery_progression::{
    WorthQueryAdmittedIdempotencyRead, WorthQueryRecoveryEffectAuthority,
    WorthQueryRecoveryInspectAuthority, WorthQueryRecoverySafeRetryAdmission,
};
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::managed_run::{
    WorthQueryRecoveryHandleRegistry, WorthQueryRecoveryRegistrySlot,
};
use crate::domain_computation::primary_graph::application_query::observed_source::{
    WorthQueryObservedResultSet, WorthQueryObservedSource,
};
use crate::domain_computation::primary_graph::application_runtime::inbound_occurrence::{
    WorthQueryInboundPendingReason, WorthQueryInboundReceipt, WorthQueryInboundReceiptPosture,
};
use crate::domain_computation::primary_graph::authenticated_principal::WorthQueryResolvedPrincipalEvidence;
use crate::domain_computation::primary_graph::workflow::WorthQueryWorkflowDispositions;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationIdempotencyResolution, WorthQueryAuthenticatedPrincipal,
};

pub trait AuthenticatedPrincipalInternalPort<Schema, Principal, PrincipalIdentity>: Sized {
    fn mint(
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        evidence: WorthQueryResolvedPrincipalEvidence<PrincipalIdentity>,
    ) -> Self;
}

impl<Schema, Principal, PrincipalIdentity>
    AuthenticatedPrincipalInternalPort<Schema, Principal, PrincipalIdentity>
    for WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>
{
    fn mint(
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        evidence: WorthQueryResolvedPrincipalEvidence<PrincipalIdentity>,
    ) -> Self {
        Self::mint(external, evidence)
    }
}

pub trait InboundReceiptInternalPort: Sized {
    fn from_accepted(
        accepted: &WorthQueryAcceptedInboundOccurrence,
        envelope: &[u8],
        posture: WorthQueryInboundReceiptPosture,
    ) -> Self;
    fn from_claims(
        claims: &WorthQueryInboundOccurrenceClaims,
        envelope: &[u8],
        posture: WorthQueryInboundReceiptPosture,
    ) -> Self;
    fn pending(
        accepted: &WorthQueryAcceptedInboundOccurrence,
        envelope: &[u8],
        posture: WorthQueryInboundReceiptPosture,
        reason: WorthQueryInboundPendingReason,
    ) -> Self;
}

impl InboundReceiptInternalPort for WorthQueryInboundReceipt {
    fn from_accepted(
        accepted: &WorthQueryAcceptedInboundOccurrence,
        envelope: &[u8],
        posture: WorthQueryInboundReceiptPosture,
    ) -> Self {
        Self::from_accepted(accepted, envelope, posture)
    }

    fn from_claims(
        claims: &WorthQueryInboundOccurrenceClaims,
        envelope: &[u8],
        posture: WorthQueryInboundReceiptPosture,
    ) -> Self {
        Self::from_claims(claims, envelope, posture)
    }

    fn pending(
        accepted: &WorthQueryAcceptedInboundOccurrence,
        envelope: &[u8],
        posture: WorthQueryInboundReceiptPosture,
        reason: WorthQueryInboundPendingReason,
    ) -> Self {
        Self::pending(accepted, envelope, posture, reason)
    }
}

pub trait AdmittedIdempotencyReadInternalPort: Sized {
    fn mint(
        binding: WorthQueryApplicationIdempotencyBinding,
        resolution: WorthQueryApplicationIdempotencyResolution,
    ) -> Self;
}

impl AdmittedIdempotencyReadInternalPort for WorthQueryAdmittedIdempotencyRead {
    fn mint(
        binding: WorthQueryApplicationIdempotencyBinding,
        resolution: WorthQueryApplicationIdempotencyResolution,
    ) -> Self {
        Self::mint(binding, resolution)
    }
}

pub trait RecoveryEffectAuthorityInternalPort: Sized {
    fn mint(
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        handle_authority: WorthQueryRecoveryHandleAuthorityIdentity,
        home_owner: RuntimeWorldOwnerIdentity,
    ) -> Self;
}

impl RecoveryEffectAuthorityInternalPort for WorthQueryRecoveryEffectAuthority {
    fn mint(
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        handle_authority: WorthQueryRecoveryHandleAuthorityIdentity,
        home_owner: RuntimeWorldOwnerIdentity,
    ) -> Self {
        Self::mint(runtime_authority, handle_authority, home_owner)
    }
}

pub trait RecoveryInspectAuthorityInternalPort: Sized {
    fn mint(
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        handle_authority: WorthQueryRecoveryHandleAuthorityIdentity,
        home_owner: RuntimeWorldOwnerIdentity,
    ) -> Self;
}

impl RecoveryInspectAuthorityInternalPort for WorthQueryRecoveryInspectAuthority {
    fn mint(
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        handle_authority: WorthQueryRecoveryHandleAuthorityIdentity,
        home_owner: RuntimeWorldOwnerIdentity,
    ) -> Self {
        Self::mint(runtime_authority, handle_authority, home_owner)
    }
}

pub trait RecoveryHandleInternalPort: Sized {
    fn from_parts(
        identity: WorthQueryRecoveryHandleIdentity,
        slot: WorthQueryRecoveryRegistrySlot,
        registry: Arc<WorthQueryRecoveryHandleRegistry>,
        binding: WorthQueryRecoveryHandleBinding,
        canonical_work: WorthQueryCanonicalWorkPhases,
    ) -> Self;

    fn admit_home_owner(
        &self,
        receiver: RuntimeWorldOwnerIdentity,
    ) -> Result<(), WorthQueryRecoveryHandleDenial>;

    fn home_owner(&self) -> RuntimeWorldOwnerIdentity;

    fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity;

    fn registry_clock(&self) -> &crate::domain_computation::runtime_time::WorthQueryRuntimeClock;

    fn authority_identity(&self) -> WorthQueryRecoveryHandleAuthorityIdentity;

    fn ensure_live(&self) -> Result<(), WorthQueryRecoveryHandleDenial>;

    fn consume(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        terminal: crate::domain_computation::managed_run::RecoveryClaimDisposition,
    ) -> Result<
        WorthQueryRecoveryHandleBinding,
        crate::domain_computation::application_aftermath::recovery_handle::WorthQueryRecoveryTransitionDenial,
    >;
}

impl RecoveryHandleInternalPort for WorthQueryRecoveryHandle {
    fn from_parts(
        identity: WorthQueryRecoveryHandleIdentity,
        slot: WorthQueryRecoveryRegistrySlot,
        registry: Arc<WorthQueryRecoveryHandleRegistry>,
        binding: WorthQueryRecoveryHandleBinding,
        canonical_work: WorthQueryCanonicalWorkPhases,
    ) -> Self {
        Self::new(identity, slot, registry, binding, canonical_work)
    }

    fn admit_home_owner(
        &self,
        receiver: RuntimeWorldOwnerIdentity,
    ) -> Result<(), WorthQueryRecoveryHandleDenial> {
        Self::admit_home_owner(self, receiver)
    }

    fn home_owner(&self) -> RuntimeWorldOwnerIdentity {
        Self::home_owner(self)
    }

    fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity {
        Self::runtime_authority(self)
    }

    fn registry_clock(&self) -> &crate::domain_computation::runtime_time::WorthQueryRuntimeClock {
        Self::registry_clock(self)
    }

    fn authority_identity(&self) -> WorthQueryRecoveryHandleAuthorityIdentity {
        Self::authority_identity(self)
    }

    fn ensure_live(&self) -> Result<(), WorthQueryRecoveryHandleDenial> {
        Self::ensure_live(self)
    }

    fn consume(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        terminal: crate::domain_computation::managed_run::RecoveryClaimDisposition,
    ) -> Result<
        WorthQueryRecoveryHandleBinding,
        crate::domain_computation::application_aftermath::recovery_handle::WorthQueryRecoveryTransitionDenial,
    >{
        Self::consume(self, phase, terminal)
    }
}

pub trait RecoverySafeRetryAdmissionInternalPort {
    fn completes_receipt(&self, receipt: &WorthQueryApplicationCommitReceipt) -> bool;
}

impl RecoverySafeRetryAdmissionInternalPort for WorthQueryRecoverySafeRetryAdmission {
    fn completes_receipt(&self, receipt: &WorthQueryApplicationCommitReceipt) -> bool {
        Self::completes_receipt(self, receipt)
    }
}

pub trait ObservedResultSetInternalPort<Query>: Sized {
    fn from_source(source: WorthQueryObservedSource<Query>) -> Self;
    fn into_source(self) -> WorthQueryObservedSource<Query>;
}

impl<Query> ObservedResultSetInternalPort<Query> for WorthQueryObservedResultSet<Query> {
    fn from_source(source: WorthQueryObservedSource<Query>) -> Self {
        Self::new(source)
    }

    fn into_source(self) -> WorthQueryObservedSource<Query> {
        self.source
    }
}

pub trait AdmittedApplicationConditionalDefinitionInternalPort: Sized {
    #[allow(clippy::too_many_arguments)]
    fn from_admitted_parts(
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        application_binding: worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        expected_product: worth_runtime_world::facade::ProductBranchObservation,
        identity: [u8; 32],
        predecessor: Arc<worth_runtime_bridge::facade::BridgeInstalledConditionalLowering>,
        request: worth_runtime_bridge::facade::BridgeOwnedConditionalInstallationRequest,
        bridge: crate::domain_computation::primary_graph::managed_bridge::conditional_owner::ConditionalOwner,
        activations: Arc<
            crate::domain_computation::execution_runtime::product_world::activation::WorthQueryProductActivationRegistry,
        >,
    ) -> Self;
}

impl AdmittedApplicationConditionalDefinitionInternalPort
    for crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition
{
    fn from_admitted_parts(
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        application_binding: worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        expected_product: worth_runtime_world::facade::ProductBranchObservation,
        identity: [u8; 32],
        predecessor: Arc<worth_runtime_bridge::facade::BridgeInstalledConditionalLowering>,
        request: worth_runtime_bridge::facade::BridgeOwnedConditionalInstallationRequest,
        bridge: crate::domain_computation::primary_graph::managed_bridge::conditional_owner::ConditionalOwner,
        activations: Arc<
            crate::domain_computation::execution_runtime::product_world::activation::WorthQueryProductActivationRegistry,
        >,
    ) -> Self {
        Self::from_admitted_parts(
            runtime_authority,
            application_binding,
            expected_product,
            identity,
            predecessor,
            request,
            bridge,
            activations,
        )
    }
}

pub trait WorkflowDispositionsInternalPort {
    fn admit_empty_population(&self) -> Result<(), ()>;

    fn resolve(
        &self,
        inventory: &crate::domain_computation::primary_graph::workflow::adoption::WorthQueryWorkflowAdoptionInventory,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<
        crate::domain_computation::primary_graph::workflow::adoption::AdmittedWorkflowDispositions,
        (),
    >;
}

impl WorkflowDispositionsInternalPort for WorthQueryWorkflowDispositions {
    fn admit_empty_population(&self) -> Result<(), ()> {
        Self::admit_empty_population(self)
    }

    fn resolve(
        &self,
        inventory: &crate::domain_computation::primary_graph::workflow::adoption::WorthQueryWorkflowAdoptionInventory,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<
        crate::domain_computation::primary_graph::workflow::adoption::AdmittedWorkflowDispositions,
        (),
    > {
        Self::resolve(self, inventory, branch)
    }
}
