use std::sync::Arc;

use sha2::{Digest, Sha256};

use worth_signal::facade::AdmittedResourceCompletion;

use super::super::request_identity::AdmittedBridgeAsyncRequestIdentity;
use super::completion::{
    BridgeAsyncCompletionClass, BridgeAsyncCompletionIdentity, BridgeAsyncCompletionState,
};
use super::counters::BridgeAsyncCompletionCounters;
use super::envelope::ValidatedBridgeAsyncCompletionEnvelope;
use super::receipt::BridgeAsyncCompletionReceipt;
use super::writeback_receipt::AsyncCompletionWritebackReceipt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedBridgeAsyncCompletion {
    completion_identity: BridgeAsyncCompletionIdentity,
    writeback_receipt: AsyncCompletionWritebackReceipt,
    request_identity: AdmittedBridgeAsyncRequestIdentity,
    validated_envelope: ValidatedBridgeAsyncCompletionEnvelope,
    admitted_completion: AdmittedResourceCompletion,
    completion_class: BridgeAsyncCompletionClass,
    counters: BridgeAsyncCompletionCounters,
    receipt: BridgeAsyncCompletionReceipt,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl AdmittedBridgeAsyncCompletion {
    pub(crate) fn new(
        request_identity: AdmittedBridgeAsyncRequestIdentity,
        validated_envelope: ValidatedBridgeAsyncCompletionEnvelope,
        admitted_completion: AdmittedResourceCompletion,
        counters: BridgeAsyncCompletionCounters,
    ) -> Self {
        Self::new_with_class(
            request_identity,
            validated_envelope,
            admitted_completion,
            counters,
            BridgeAsyncCompletionClass::Fulfilled,
        )
    }

    pub(crate) fn from_owner_effects_indeterminate(
        self,
        _observation: super::super::BridgeAsyncEffectsIndeterminateCompletion,
    ) -> Self {
        Self::new_with_class(
            self.request_identity,
            self.validated_envelope,
            self.admitted_completion,
            self.counters,
            BridgeAsyncCompletionClass::EffectsIndeterminate,
        )
    }

    fn new_with_class(
        request_identity: AdmittedBridgeAsyncRequestIdentity,
        validated_envelope: ValidatedBridgeAsyncCompletionEnvelope,
        admitted_completion: AdmittedResourceCompletion,
        counters: BridgeAsyncCompletionCounters,
        completion_class: BridgeAsyncCompletionClass,
    ) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-async-completion|request={}|envelope={}|admitted-handle={}#{}|attempt={}|node={}|descriptor={}|ordinal={}|transition={:?}->{:?}|payload-bytes={}|class={:?}",
            request_identity.request_identity().as_str(),
            validated_envelope.envelope().digest(),
            admitted_completion.handle().request_id().get(),
            admitted_completion.handle().generation().get(),
            validated_envelope.raw().attempt().get(),
            admitted_completion.node().node(),
            admitted_completion.descriptor_id().get(),
            admitted_completion.completion_ordinal().get(),
            admitted_completion.lifecycle_transition().from(),
            admitted_completion.lifecycle_transition().to(),
            admitted_completion.payload_byte_len(),
            completion_class,
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        let completion_identity = BridgeAsyncCompletionIdentity::admit_bridge_owned(format!(
            "bridge-async-completion-id:sha256:{digest:x}"
        ));
        let receipt = BridgeAsyncCompletionReceipt::admitted(
            &completion_identity,
            BridgeAsyncCompletionState::Admitted(completion_class),
            Arc::from(format!(
                "bridge-async-completion-receipt|completion={}|request={}|transition={:?}->{:?}|ordinal={}",
                completion_identity.as_str(),
                request_identity.request_identity().as_str(),
                admitted_completion.lifecycle_transition().from(),
                admitted_completion.lifecycle_transition().to(),
                admitted_completion.completion_ordinal().get(),
            )),
        );

        Self {
            completion_identity,
            writeback_receipt: Default::default(),
            request_identity,
            validated_envelope,
            admitted_completion,
            completion_class,
            counters,
            receipt,
            canonical_basis,
            digest: Arc::from(format!("bridge-async-completion:sha256:{digest:x}")),
        }
    }

    pub(crate) fn writeback_receipt(&self) -> &AsyncCompletionWritebackReceipt {
        &self.writeback_receipt
    }

    pub fn completion_identity(&self) -> &str {
        self.completion_identity.as_str()
    }

    pub fn request_identity(&self) -> &AdmittedBridgeAsyncRequestIdentity {
        &self.request_identity
    }

    pub fn validated_envelope(&self) -> &ValidatedBridgeAsyncCompletionEnvelope {
        &self.validated_envelope
    }

    pub fn admitted_completion(&self) -> AdmittedResourceCompletion {
        self.admitted_completion
    }

    pub fn completion_class(&self) -> BridgeAsyncCompletionClass {
        self.completion_class
    }

    pub fn state(&self) -> BridgeAsyncCompletionState {
        BridgeAsyncCompletionState::Admitted(self.completion_class)
    }

    pub fn counters(&self) -> &BridgeAsyncCompletionCounters {
        &self.counters
    }

    pub fn receipt(&self) -> &BridgeAsyncCompletionReceipt {
        &self.receipt
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
