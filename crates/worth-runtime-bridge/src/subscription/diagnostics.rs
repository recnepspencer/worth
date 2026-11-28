use std::sync::Arc;

use sha2::{Digest, Sha256};

use super::{
    BridgeSubscriptionActivationReady, BridgeSubscriptionAdmissionRejection,
    BridgeSubscriptionCounters, BridgeSubscriptionDeactivated,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSubscriptionExplanation {
    declaration_identity: super::BridgeSubscriptionDeclarationIdentity,
    requested_family_kind: super::BridgeSubscriptionDeclarationFamilyKind,
    registry_identity: Option<super::BridgeSubscriptionFamilyRegistryIdentity>,
    admitted_subscription_identity: Option<super::BridgeAdmittedSubscriptionIdentity>,
    lifecycle_identity: Option<super::BridgeSubscriptionLifecycleIdentity>,
    basis_kind: Option<super::BridgeSubscriptionBasisKind>,
    admission_rejection_kind: Option<super::BridgeSubscriptionAdmissionRejectionKind>,
    basis_resolution_failure_kind: Option<super::BridgeSubscriptionBasisResolutionFailureKind>,
    signal_strategy_kind: Option<super::BridgeSignalStrategyKind>,
    lifecycle_state_kind: Option<super::BridgeSubscriptionLifecycleStateKind>,
    normalized_slice_intent_count: usize,
    counters: BridgeSubscriptionCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeSubscriptionExplanation {
    pub(crate) fn from_activation_ready(ready: &BridgeSubscriptionActivationReady) -> Self {
        let admitted = ready.admitted();
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-subscription-explanation|declaration={}|family={}|registry={}|admitted={}|lifecycle={}|basis={}|strategy={}|state={}|slice-count={}",
            admitted.declaration().declaration_identity().as_str(),
            admitted.declaration().requested_family_kind().as_str(),
            ready.retained_bundle().registry_identity().as_str(),
            admitted.admitted_subscription_identity().as_str(),
            ready.lifecycle_record().lifecycle_identity().as_str(),
            admitted.basis_binding().basis_kind().as_str(),
            admitted.signal_strategy().strategy_kind().as_str(),
            ready.lifecycle_record().state_kind().as_str(),
            admitted.declaration().normalized_slice_intent_count(),
        ));
        Self::from_success(
            admitted.declaration().declaration_identity().clone(),
            admitted.declaration().requested_family_kind(),
            Some(ready.retained_bundle().registry_identity().clone()),
            Some(admitted.admitted_subscription_identity().clone()),
            Some(ready.lifecycle_record().lifecycle_identity().clone()),
            Some(admitted.basis_binding().basis_kind()),
            None,
            None,
            Some(admitted.signal_strategy().strategy_kind()),
            Some(ready.lifecycle_record().state_kind()),
            admitted.declaration().normalized_slice_intent_count(),
            canonical_basis,
        )
    }

    pub(crate) fn from_deactivated(deactivated: &BridgeSubscriptionDeactivated) -> Self {
        let admitted = deactivated.admitted();
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-subscription-explanation|declaration={}|family={}|registry={}|admitted={}|lifecycle={}|basis={}|strategy={}|state={}|slice-count={}",
            admitted.declaration().declaration_identity().as_str(),
            admitted.declaration().requested_family_kind().as_str(),
            deactivated.retained_bundle().registry_identity().as_str(),
            admitted.admitted_subscription_identity().as_str(),
            deactivated.lifecycle_record().lifecycle_identity().as_str(),
            admitted.basis_binding().basis_kind().as_str(),
            admitted.signal_strategy().strategy_kind().as_str(),
            deactivated.lifecycle_record().state_kind().as_str(),
            admitted.declaration().normalized_slice_intent_count(),
        ));
        Self::from_success(
            admitted.declaration().declaration_identity().clone(),
            admitted.declaration().requested_family_kind(),
            Some(deactivated.retained_bundle().registry_identity().clone()),
            Some(admitted.admitted_subscription_identity().clone()),
            Some(deactivated.lifecycle_record().lifecycle_identity().clone()),
            Some(admitted.basis_binding().basis_kind()),
            None,
            None,
            Some(admitted.signal_strategy().strategy_kind()),
            Some(deactivated.lifecycle_record().state_kind()),
            admitted.declaration().normalized_slice_intent_count(),
            canonical_basis,
        )
    }

    pub(crate) fn from_rejection(rejection: &BridgeSubscriptionAdmissionRejection) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-subscription-explanation|declaration={}|family={}|basis={}|rejection={}|basis-resolution-kind={}",
            rejection.declaration_identity().as_str(),
            rejection.requested_family_kind().as_str(),
            rejection.requested_basis_kind().as_str(),
            rejection.rejection_kind().as_str(),
            rejection
                .basis_resolution_failure_kind()
                .map(super::BridgeSubscriptionBasisResolutionFailureKind::as_str)
                .unwrap_or("-"),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            declaration_identity: rejection.declaration_identity().clone(),
            requested_family_kind: rejection.requested_family_kind(),
            registry_identity: None,
            admitted_subscription_identity: None,
            lifecycle_identity: None,
            basis_kind: Some(rejection.requested_basis_kind()),
            admission_rejection_kind: Some(rejection.rejection_kind()),
            basis_resolution_failure_kind: rejection.basis_resolution_failure_kind(),
            signal_strategy_kind: None,
            lifecycle_state_kind: None,
            normalized_slice_intent_count: 0,
            counters: BridgeSubscriptionCounters::from_diagnostics_bundle(),
            canonical_basis,
            digest: Arc::from(format!("bridge-subscription-explanation:sha256:{digest:x}")),
        }
    }

    fn from_success(
        declaration_identity: super::BridgeSubscriptionDeclarationIdentity,
        requested_family_kind: super::BridgeSubscriptionDeclarationFamilyKind,
        registry_identity: Option<super::BridgeSubscriptionFamilyRegistryIdentity>,
        admitted_subscription_identity: Option<super::BridgeAdmittedSubscriptionIdentity>,
        lifecycle_identity: Option<super::BridgeSubscriptionLifecycleIdentity>,
        basis_kind: Option<super::BridgeSubscriptionBasisKind>,
        admission_rejection_kind: Option<super::BridgeSubscriptionAdmissionRejectionKind>,
        basis_resolution_failure_kind: Option<super::BridgeSubscriptionBasisResolutionFailureKind>,
        signal_strategy_kind: Option<super::BridgeSignalStrategyKind>,
        lifecycle_state_kind: Option<super::BridgeSubscriptionLifecycleStateKind>,
        normalized_slice_intent_count: usize,
        canonical_basis: Arc<str>,
    ) -> Self {
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            declaration_identity,
            requested_family_kind,
            registry_identity,
            admitted_subscription_identity,
            lifecycle_identity,
            basis_kind,
            admission_rejection_kind,
            basis_resolution_failure_kind,
            signal_strategy_kind,
            lifecycle_state_kind,
            normalized_slice_intent_count,
            counters: BridgeSubscriptionCounters::from_diagnostics_bundle(),
            canonical_basis,
            digest: Arc::from(format!("bridge-subscription-explanation:sha256:{digest:x}")),
        }
    }

    pub fn declaration_identity(&self) -> &super::BridgeSubscriptionDeclarationIdentity {
        &self.declaration_identity
    }

    pub fn requested_family_kind(&self) -> super::BridgeSubscriptionDeclarationFamilyKind {
        self.requested_family_kind
    }

    pub fn registry_identity(&self) -> Option<&super::BridgeSubscriptionFamilyRegistryIdentity> {
        self.registry_identity.as_ref()
    }

    pub fn admitted_subscription_identity(
        &self,
    ) -> Option<&super::BridgeAdmittedSubscriptionIdentity> {
        self.admitted_subscription_identity.as_ref()
    }

    pub fn lifecycle_identity(&self) -> Option<&super::BridgeSubscriptionLifecycleIdentity> {
        self.lifecycle_identity.as_ref()
    }

    pub fn basis_kind(&self) -> Option<super::BridgeSubscriptionBasisKind> {
        self.basis_kind
    }

    pub fn admission_rejection_kind(
        &self,
    ) -> Option<super::BridgeSubscriptionAdmissionRejectionKind> {
        self.admission_rejection_kind
    }

    pub fn basis_resolution_failure_kind(
        &self,
    ) -> Option<super::BridgeSubscriptionBasisResolutionFailureKind> {
        self.basis_resolution_failure_kind
    }

    pub fn signal_strategy_kind(&self) -> Option<super::BridgeSignalStrategyKind> {
        self.signal_strategy_kind
    }

    pub fn lifecycle_state_kind(&self) -> Option<super::BridgeSubscriptionLifecycleStateKind> {
        self.lifecycle_state_kind
    }

    pub fn normalized_slice_intent_count(&self) -> usize {
        self.normalized_slice_intent_count
    }

    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
