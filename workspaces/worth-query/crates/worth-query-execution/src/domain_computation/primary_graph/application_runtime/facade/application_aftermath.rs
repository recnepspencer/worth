//! The runtime's consumer methods over `domain_computation::application_aftermath`; each only calls its subsystem.
use crate::domain_computation::application_aftermath::recovery_handle::identity::WorthQueryRecoveryHandleIdentity;
use crate::domain_computation::application_aftermath::recovery_handle::WorthQueryRecoveryHandleAuthorityIdentity;
use crate::domain_computation::application_aftermath::WorthQueryRecoveryClaimStatus;
use crate::domain_computation::application_aftermath::WorthQueryRecoveryDisclosureAdmission;
use crate::domain_computation::application_aftermath::WorthQueryRecoveryEffectAuthority;
use crate::domain_computation::application_aftermath::WorthQueryRecoveryExpiryEvaluation;
use crate::domain_computation::application_aftermath::WorthQueryRecoveryHandle;
use crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleDenial;
use crate::domain_computation::application_aftermath::WorthQueryRecoveryInspectAuthority;
use crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;
use crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::vocabulary::execution_runtime::runtime_identity::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::managed_run::WorthQueryRecoveryHandleRegistry;
use crate::domain_computation::managed_run::WorthQueryRecoveryMintClaim;
use crate::domain_computation::managed_run::WorthQueryRecoveryResourceTerminal;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::runtime_time::WorthQueryRuntimeClock;
use crate::domain_computation::runtime_time::WorthQueryRuntimeTimeSample;
use std::sync::Arc;
use worth_foundational::facade::AspectValue;
use worth_proof::evaluate_freshness;
use worth_proof::Artifact;
use worth_proof::AssumptionBasis;
use worth_proof::AuthorityRevalidationRequired;
use worth_proof::CurrentValidity;
use worth_proof::EvaluatedFreshness;
use worth_proof::FreshnessEvaluation;
use worth_proof::FreshnessPolicy;
use worth_proof::FreshnessScopedBasis;
use worth_proof::FreshnessVerdict;
use worth_proof::NoProofs;
use worth_proof::PhaseMarker;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline;
use worth_query_installation::facade::ApplicationSchema;
impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn mint_recovery_handle(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<WorthQueryRecoveryHandle, WorthQueryRecoveryHandleDenial> {
        <Self as crate::domain_computation::application_aftermath::MintOps<Schema>>::mint_recovery_handle(self, receipt)
    }

    pub fn recovery_claim_status(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<WorthQueryRecoveryClaimStatus, WorthQueryRecoveryHandleDenial> {
        <Self as crate::domain_computation::application_aftermath::MintOps<Schema>>::recovery_claim_status(self, receipt)
    }

    pub fn admit_recovery_effect_authority<Operation, Input, Scope>(
        &self,
        handle: &WorthQueryRecoveryHandle,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<WorthQueryRecoveryEffectAuthority, WorthQueryRecoveryHandleDenial> {
        <Self as crate::domain_computation::application_aftermath::AuthorityOps<Schema>>::admit_recovery_effect_authority(self, handle, admission)
    }

    pub fn admit_recovery_inspect_authority<Operation, Input, Scope>(
        &self,
        handle: &WorthQueryRecoveryHandle,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        disclosure: &WorthQueryRecoveryDisclosureAdmission,
    ) -> Result<WorthQueryRecoveryInspectAuthority, WorthQueryRecoveryHandleDenial> {
        <Self as crate::domain_computation::application_aftermath::AuthorityOps<Schema>>::admit_recovery_inspect_authority(self, handle, admission, disclosure)
    }

    pub fn admit_recovery_inspection_disclosure<Capability, Operation, Input>(
        &self,
        access: &WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
    ) -> Result<WorthQueryRecoveryDisclosureAdmission, WorthQueryRecoveryHandleDenial>
    where
        Input: ApplicationCapabilityRequest<Schema, Capability>,
    {
        <Self as crate::domain_computation::application_aftermath::DiscloseOps<Schema>>::admit_recovery_inspection_disclosure(self, access)
    }

    pub fn evaluate_recovery_expiry(
        &self,
        handle: &WorthQueryRecoveryHandle,
    ) -> Result<WorthQueryRecoveryExpiryEvaluation, WorthQueryRecoveryHandleDenial> {
        <Self as crate::domain_computation::application_aftermath::ExpiryOps<Schema>>::evaluate_recovery_expiry(self, handle)
    }
}
