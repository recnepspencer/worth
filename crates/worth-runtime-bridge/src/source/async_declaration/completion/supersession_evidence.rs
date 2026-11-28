use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::identity::{AsyncCompletionSupersessionIdentityTag, BridgeIdentity};

use super::super::request_identity::{
    AdmittedBridgeAsyncRequestIdentity, BridgeAsyncRequestSubscriptionInstance,
    BridgeAsyncRequestTruthViewBasis,
};

pub type BridgeAsyncCompletionSupersessionIdentity =
    BridgeIdentity<AsyncCompletionSupersessionIdentityTag>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeAsyncCompletionSupersessionClass {
    TruthBasisSuperseded,
    BranchDrifted,
    PreviewBasisDrifted,
    PreviewDiscarded,
    SubscriptionInstanceSuperseded,
    SignalGenerationSuperseded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeAsyncCompletionSupersessionClassificationRequest {
    denied_completion: super::BridgeAsyncDeniedCompletion,
    current_truth_view_basis: BridgeAsyncRequestTruthViewBasis,
    current_subscription_instance: Option<BridgeAsyncRequestSubscriptionInstance>,
    displacing_request_identity: Option<AdmittedBridgeAsyncRequestIdentity>,
}

impl BridgeAsyncCompletionSupersessionClassificationRequest {
    pub fn request_response(
        denied_completion: &super::BridgeAsyncDeniedCompletion,
        current_truth_view_basis: BridgeAsyncRequestTruthViewBasis,
    ) -> Self {
        Self {
            denied_completion: denied_completion.clone(),
            current_truth_view_basis,
            current_subscription_instance: None,
            displacing_request_identity: None,
        }
    }

    pub fn subscription_backed(
        denied_completion: &super::BridgeAsyncDeniedCompletion,
        current_truth_view_basis: BridgeAsyncRequestTruthViewBasis,
        current_subscription_instance: BridgeAsyncRequestSubscriptionInstance,
    ) -> Self {
        Self {
            denied_completion: denied_completion.clone(),
            current_truth_view_basis,
            current_subscription_instance: Some(current_subscription_instance),
            displacing_request_identity: None,
        }
    }

    pub fn with_displacing_request_identity(
        mut self,
        displacing_request_identity: &AdmittedBridgeAsyncRequestIdentity,
    ) -> Self {
        self.displacing_request_identity = Some(displacing_request_identity.clone());
        self
    }

    pub fn denied_completion(&self) -> &super::BridgeAsyncDeniedCompletion {
        &self.denied_completion
    }

    pub fn current_truth_view_basis(&self) -> &BridgeAsyncRequestTruthViewBasis {
        &self.current_truth_view_basis
    }

    pub fn current_subscription_instance(&self) -> Option<&BridgeAsyncRequestSubscriptionInstance> {
        self.current_subscription_instance.as_ref()
    }

    pub fn displacing_request_identity(&self) -> Option<&AdmittedBridgeAsyncRequestIdentity> {
        self.displacing_request_identity.as_ref()
    }

    pub fn preview_discarded(&self) -> bool {
        self.denied_completion
            .request_identity()
            .basis_binding()
            .truth_view_basis()
            .preview_session()
            .is_some_and(|session| session.admit_active_session().is_none())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeAsyncCompletionSupersessionEvidence {
    supersession_identity: BridgeAsyncCompletionSupersessionIdentity,
    denied_completion_identity: Arc<str>,
    original_truth_view_basis_digest: Arc<str>,
    current_truth_view_basis_digest: Arc<str>,
    original_subscription_instance_digest: Option<Arc<str>>,
    current_subscription_instance_digest: Option<Arc<str>>,
    displacing_request_identity: Option<Arc<str>>,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeAsyncCompletionSupersessionEvidence {
    pub(crate) fn new(
        request: &BridgeAsyncCompletionSupersessionClassificationRequest,
        supersession_class: super::BridgeAsyncCompletionSupersessionClass,
        preview_discarded: bool,
    ) -> Self {
        let denied_completion = request.denied_completion();
        let request_identity = denied_completion.request_identity();
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-async-completion-supersession-evidence|denied={}|class={supersession_class:?}|original-truth-view={}|current-truth-view={}|original-subscription-instance={}|current-subscription-instance={}|displacing-request={}|preview-discarded={}",
            denied_completion.denial_identity(),
            request_identity.basis_binding().truth_view_basis().digest(),
            request.current_truth_view_basis().digest(),
            request_identity
                .subscription_instance()
                .map(BridgeAsyncRequestSubscriptionInstance::digest)
                .unwrap_or("-"),
            request
                .current_subscription_instance()
                .map(BridgeAsyncRequestSubscriptionInstance::digest)
                .unwrap_or("-"),
            request
                .displacing_request_identity()
                .map(|request| request.request_identity().as_str())
                .unwrap_or("-"),
            preview_discarded,
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            supersession_identity: BridgeAsyncCompletionSupersessionIdentity::admit_bridge_owned(
                format!("bridge-async-completion-supersession-id:sha256:{digest:x}"),
            ),
            denied_completion_identity: Arc::from(denied_completion.denial_identity().to_owned()),
            original_truth_view_basis_digest: Arc::from(
                request_identity
                    .basis_binding()
                    .truth_view_basis()
                    .digest()
                    .to_owned(),
            ),
            current_truth_view_basis_digest: Arc::from(
                request.current_truth_view_basis().digest().to_owned(),
            ),
            original_subscription_instance_digest: request
                .denied_completion()
                .request_identity()
                .subscription_instance()
                .map(|instance| Arc::from(instance.digest().to_owned())),
            current_subscription_instance_digest: request
                .current_subscription_instance()
                .map(|instance| Arc::from(instance.digest().to_owned())),
            displacing_request_identity: request
                .displacing_request_identity()
                .map(|request| Arc::from(request.request_identity().as_str().to_owned())),
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-async-completion-supersession-evidence:sha256:{digest:x}"
            )),
        }
    }

    pub fn supersession_identity(&self) -> &BridgeAsyncCompletionSupersessionIdentity {
        &self.supersession_identity
    }

    pub fn denied_completion_identity(&self) -> &str {
        self.denied_completion_identity.as_ref()
    }

    pub fn original_truth_view_basis_digest(&self) -> &str {
        self.original_truth_view_basis_digest.as_ref()
    }

    pub fn current_truth_view_basis_digest(&self) -> &str {
        self.current_truth_view_basis_digest.as_ref()
    }

    pub fn original_subscription_instance_digest(&self) -> Option<&str> {
        self.original_subscription_instance_digest.as_deref()
    }

    pub fn current_subscription_instance_digest(&self) -> Option<&str> {
        self.current_subscription_instance_digest.as_deref()
    }

    pub fn displacing_request_identity(&self) -> Option<&str> {
        self.displacing_request_identity.as_deref()
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
