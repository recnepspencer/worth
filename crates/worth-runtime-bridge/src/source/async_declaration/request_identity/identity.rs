use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::identity::{AsyncInFlightRequestIdentityTag, AsyncRequestIdentityTag, BridgeIdentity};
use worth_signal::facade::{
    AdmittedResourceRequest, InFlightResourceRequest, ResourceAttemptId, ResourceRequestHandle,
};

use super::super::LoweredBridgeAsyncSourceDeclaration;
use super::binding::ValidatedBridgeAsyncRequestBasisBinding;
use super::counters::BridgeAsyncRequestIdentityCounters;
use super::subscription_instance::BridgeAsyncRequestSubscriptionInstance;

pub type BridgeAsyncRequestIdentity = BridgeIdentity<AsyncRequestIdentityTag>;
pub(super) type BridgeAsyncInFlightRequestIdentityHandle =
    BridgeIdentity<AsyncInFlightRequestIdentityTag>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BridgeAsyncRequestRuntimeIdentity {
    key: u64,
}

impl BridgeAsyncRequestRuntimeIdentity {
    pub(crate) fn new(key: u64) -> Self {
        Self { key }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeAsyncRequestFamilyAdmission {
    RequestResponse,
    SubscriptionBacked {
        subscription_instance: BridgeAsyncRequestSubscriptionInstance,
    },
}

impl BridgeAsyncRequestFamilyAdmission {
    pub fn subscription_instance(&self) -> Option<&BridgeAsyncRequestSubscriptionInstance> {
        match self {
            Self::RequestResponse => None,
            Self::SubscriptionBacked {
                subscription_instance,
            } => Some(subscription_instance),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeAsyncRequestAdmissionRequest {
    lowered: LoweredBridgeAsyncSourceDeclaration,
    basis_binding: ValidatedBridgeAsyncRequestBasisBinding,
    family_admission: BridgeAsyncRequestFamilyAdmission,
}

impl BridgeAsyncRequestAdmissionRequest {
    pub(super) fn new(
        lowered: LoweredBridgeAsyncSourceDeclaration,
        basis_binding: ValidatedBridgeAsyncRequestBasisBinding,
        family_admission: BridgeAsyncRequestFamilyAdmission,
    ) -> Self {
        Self {
            lowered,
            basis_binding,
            family_admission,
        }
    }

    pub(crate) fn rebind(
        lowered: &LoweredBridgeAsyncSourceDeclaration,
        basis_binding: &ValidatedBridgeAsyncRequestBasisBinding,
        family_admission: &BridgeAsyncRequestFamilyAdmission,
    ) -> Result<Self, super::rejection::BridgeAsyncRequestIdentityRejection> {
        match family_admission {
            BridgeAsyncRequestFamilyAdmission::RequestResponse => {
                Self::request_response(lowered, basis_binding)
            }
            BridgeAsyncRequestFamilyAdmission::SubscriptionBacked {
                subscription_instance,
            } => Self::subscription_backed(lowered, basis_binding, subscription_instance.clone()),
        }
    }

    pub fn lowered(&self) -> &LoweredBridgeAsyncSourceDeclaration {
        &self.lowered
    }

    pub fn basis_binding(&self) -> &ValidatedBridgeAsyncRequestBasisBinding {
        &self.basis_binding
    }

    pub fn family_admission(&self) -> &BridgeAsyncRequestFamilyAdmission {
        &self.family_admission
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeAsyncInFlightRequestIdentity {
    in_flight_identity: BridgeAsyncInFlightRequestIdentityHandle,
    request_identity: BridgeAsyncRequestIdentity,
    in_flight: InFlightResourceRequest,
    counters: BridgeAsyncRequestIdentityCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeAsyncInFlightRequestIdentity {
    pub(super) fn new(
        request_identity: &BridgeAsyncRequestIdentity,
        in_flight: InFlightResourceRequest,
        counters: BridgeAsyncRequestIdentityCounters,
    ) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-async-in-flight-request|request={}|handle={}#{}|attempt={}|status={:?}|intent={}",
            request_identity.as_str(),
            in_flight.handle().request_id().get(),
            in_flight.handle().generation().get(),
            in_flight.attempt().get(),
            in_flight.status(),
            in_flight.request_intent_digest().as_str(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            in_flight_identity: BridgeAsyncInFlightRequestIdentityHandle::admit_bridge_owned(
                format!("bridge-async-in-flight-request-id:sha256:{digest:x}"),
            ),
            request_identity: request_identity.clone(),
            in_flight,
            counters,
            canonical_basis,
            digest: Arc::from(format!("bridge-async-in-flight-request:sha256:{digest:x}")),
        }
    }

    pub fn in_flight_identity(&self) -> &BridgeAsyncInFlightRequestIdentityHandle {
        &self.in_flight_identity
    }

    pub fn request_identity(&self) -> &BridgeAsyncRequestIdentity {
        &self.request_identity
    }

    pub fn in_flight(&self) -> &InFlightResourceRequest {
        &self.in_flight
    }

    pub fn counters(&self) -> &BridgeAsyncRequestIdentityCounters {
        &self.counters
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedBridgeAsyncRequestIdentity {
    bridge_runtime_key: u64,
    // External Signal-owner admission has external custody; Bridge-owned
    // requests retain their local runtime through finalization and retries.
    runtime_custody: Option<super::BridgeSignalRuntimeCustody>,
    request_identity: BridgeAsyncRequestIdentity,
    lowered: LoweredBridgeAsyncSourceDeclaration,
    basis_binding: ValidatedBridgeAsyncRequestBasisBinding,
    family_admission: BridgeAsyncRequestFamilyAdmission,
    signal_admission: AdmittedResourceRequest,
    request_intent_digest: Arc<str>,
    in_flight_identity: BridgeAsyncInFlightRequestIdentity,
    counters: BridgeAsyncRequestIdentityCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl AdmittedBridgeAsyncRequestIdentity {
    pub(super) fn new(
        bridge_runtime_key: u64,
        request_identity: BridgeAsyncRequestIdentity,
        lowered: LoweredBridgeAsyncSourceDeclaration,
        basis_binding: ValidatedBridgeAsyncRequestBasisBinding,
        family_admission: BridgeAsyncRequestFamilyAdmission,
        signal_admission: AdmittedResourceRequest,
        request_intent_digest: Arc<str>,
        in_flight_identity: BridgeAsyncInFlightRequestIdentity,
        counters: BridgeAsyncRequestIdentityCounters,
        canonical_basis: Arc<str>,
        digest: Arc<str>,
    ) -> Self {
        Self {
            bridge_runtime_key,
            runtime_custody: None,
            request_identity,
            lowered,
            basis_binding,
            family_admission,
            signal_admission,
            request_intent_digest,
            in_flight_identity,
            counters,
            canonical_basis,
            digest,
        }
    }

    pub(crate) fn bridge_runtime_key(&self) -> u64 {
        self.bridge_runtime_key
    }

    pub(crate) fn lifecycle_observation_custody(
        &self,
    ) -> Option<super::BridgeSignalRuntimeCustody> {
        self.runtime_custody.clone()
    }

    pub(crate) fn retain_runtime(mut self, custody: super::BridgeSignalRuntimeCustody) -> Self {
        assert_eq!(custody.key(), self.bridge_runtime_key);
        self.runtime_custody = Some(custody);
        self
    }

    pub(crate) fn inherit_runtime(self, prior: &Self) -> Option<Self> {
        if self.bridge_runtime_key != prior.bridge_runtime_key {
            return None;
        }
        Some(match &prior.runtime_custody {
            Some(custody) => self.retain_runtime(custody.clone()),
            None => self,
        })
    }

    pub fn bridge_runtime_identity(&self) -> BridgeAsyncRequestRuntimeIdentity {
        BridgeAsyncRequestRuntimeIdentity::new(self.bridge_runtime_key)
    }

    pub fn request_identity(&self) -> &BridgeAsyncRequestIdentity {
        &self.request_identity
    }

    pub fn request_identity_for_reporting(&self) -> &str {
        self.request_identity.as_str()
    }

    pub fn lowered(&self) -> &LoweredBridgeAsyncSourceDeclaration {
        &self.lowered
    }

    pub fn basis_binding(&self) -> &ValidatedBridgeAsyncRequestBasisBinding {
        &self.basis_binding
    }

    pub fn family_admission(&self) -> &BridgeAsyncRequestFamilyAdmission {
        &self.family_admission
    }

    pub fn subscription_instance(&self) -> Option<&BridgeAsyncRequestSubscriptionInstance> {
        self.family_admission.subscription_instance()
    }

    pub(crate) fn subscription_instance_digest(&self) -> Option<&str> {
        self.subscription_instance()
            .map(|instance| instance.digest())
    }

    pub fn request_handle(&self) -> ResourceRequestHandle {
        self.signal_admission.handle()
    }

    pub fn attempt(&self) -> ResourceAttemptId {
        self.signal_admission.attempt()
    }

    pub(crate) fn signal_admission(&self) -> AdmittedResourceRequest {
        self.signal_admission
    }

    pub fn request_intent_digest(&self) -> &str {
        self.request_intent_digest.as_ref()
    }

    pub fn in_flight_identity(&self) -> &BridgeAsyncInFlightRequestIdentity {
        &self.in_flight_identity
    }

    pub fn counters(&self) -> &BridgeAsyncRequestIdentityCounters {
        &self.counters
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
