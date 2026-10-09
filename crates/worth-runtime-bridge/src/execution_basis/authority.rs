use std::sync::Arc;

use sha2::{Digest, Sha256};

use super::posture::{BridgeExecutionBasisPosture, BridgeExecutionPosture};
use crate::input::envelope::BridgeAuthoritativeSourceProfile;
use crate::snapshot::MaterializedTruthViewObservation;
use crate::source::AdmittedBridgeAsyncRequestIdentity;

use super::reservation::BridgeExecutionBasisReservation;
use super::{
    BridgeExecutionBasisCounters, BridgeManagedExecutionIntent, BridgeManagedExecutionStepContract,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeExecutionBasisIdentity(Arc<str>);

impl BridgeExecutionBasisIdentity {
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

pub struct BridgeBoundExecutionBasis {
    pub(super) identity: BridgeExecutionBasisIdentity,
    pub(super) bridge_runtime_key: u64,
    pub(super) managed_intent: BridgeManagedExecutionIntent,
    pub(super) posture: BridgeExecutionBasisPosture,
    pub(super) request: AdmittedBridgeAsyncRequestIdentity,
    pub(super) observation: Option<MaterializedTruthViewObservation>,
    pub(super) authoritative_source_profile: Option<BridgeAuthoritativeSourceProfile>,
    pub(super) reservation: Option<BridgeExecutionBasisReservation>,
    pub(super) signal_terminalized: bool,
    pub(super) counters: BridgeExecutionBasisCounters,
}

pub(super) struct BridgeBoundExecutionBasisParts {
    pub bridge_runtime_key: u64,
    pub managed_intent: BridgeManagedExecutionIntent,
    pub posture: BridgeExecutionBasisPosture,
    pub request: AdmittedBridgeAsyncRequestIdentity,
    pub observation: MaterializedTruthViewObservation,
    pub authoritative_source_profile: Option<BridgeAuthoritativeSourceProfile>,
    pub reservation: BridgeExecutionBasisReservation,
    pub counters: BridgeExecutionBasisCounters,
}

impl std::fmt::Debug for BridgeBoundExecutionBasis {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BridgeBoundExecutionBasis")
            .field("identity", &self.identity)
            .field("managed_intent", &self.managed_intent.identity())
            .field("request", &self.request.request_identity())
            .field("signal_terminalized", &self.signal_terminalized)
            .finish()
    }
}

impl BridgeBoundExecutionBasis {
    pub(super) fn new(parts: BridgeBoundExecutionBasisParts) -> Self {
        let canonical_basis = match &parts.posture {
            BridgeExecutionBasisPosture::Atomic => format!(
                "bridge-atomic-execution-basis-v1|runtime={}|intent={}|request={}|truth-view={}|snapshot-token={}",
                parts.bridge_runtime_key,
                parts.managed_intent.identity().as_str(),
                parts.request.digest(),
                parts.observation.planned().digest(),
                parts.observation.snapshot_token().token_value(),
            ),
            BridgeExecutionBasisPosture::Managed(managed) => format!(
            "bridge-execution-basis|runtime={bridge_runtime_key}|managed-intent={}|step-contract={}|request={}|truth-view={}|snapshot-token={}",
            parts.managed_intent.identity().as_str(),
            managed.step_contract.identity(),
            parts.request.digest(),
            parts.observation.planned().digest(),
            parts.observation.snapshot_token().token_value(),
            bridge_runtime_key = parts.bridge_runtime_key,
            ),
        };
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            identity: BridgeExecutionBasisIdentity(Arc::from(format!(
                "bridge-execution-basis:sha256:{digest:x}"
            ))),
            bridge_runtime_key: parts.bridge_runtime_key,
            managed_intent: parts.managed_intent,
            posture: parts.posture,
            request: parts.request,
            observation: Some(parts.observation),
            authoritative_source_profile: parts.authoritative_source_profile,
            reservation: Some(parts.reservation),
            signal_terminalized: false,
            counters: parts.counters,
        }
    }

    pub fn identity(&self) -> &BridgeExecutionBasisIdentity {
        &self.identity
    }

    pub fn bridge_runtime_key(&self) -> u64 {
        self.bridge_runtime_key
    }

    pub fn managed_intent(&self) -> &BridgeManagedExecutionIntent {
        &self.managed_intent
    }

    pub fn execution_posture(&self) -> BridgeExecutionPosture {
        match self.posture {
            BridgeExecutionBasisPosture::Atomic => BridgeExecutionPosture::Atomic,
            BridgeExecutionBasisPosture::Managed(_) => BridgeExecutionPosture::Managed,
        }
    }

    pub fn step_contract(&self) -> Option<&BridgeManagedExecutionStepContract> {
        self.posture.managed().map(|managed| &managed.step_contract)
    }

    pub fn request(&self) -> &AdmittedBridgeAsyncRequestIdentity {
        &self.request
    }

    pub fn observation(&self) -> &MaterializedTruthViewObservation {
        self.observation
            .as_ref()
            .expect("active and yielded bridge bases retain their truth observation")
    }

    pub fn authoritative_source_profile(&self) -> Option<&BridgeAuthoritativeSourceProfile> {
        self.authoritative_source_profile.as_ref()
    }

    pub fn counters(&self) -> &BridgeExecutionBasisCounters {
        &self.counters
    }

    pub(super) fn take_observation(&mut self) -> MaterializedTruthViewObservation {
        self.observation
            .take()
            .expect("bridge truth observation transfers into one readmitted basis")
    }
}

impl Drop for BridgeBoundExecutionBasis {
    fn drop(&mut self) {
        if !self.signal_terminalized {
            let _ = crate::source::with_async_request_signal_runtime(
                self.bridge_runtime_key,
                |runtime| {
                    runtime.cancel_resource_request(
                        self.request.request_handle(),
                        worth_signal::facade::ResourceCancellationReason::RuntimePolicy,
                    )
                },
            );
        }
    }
}
