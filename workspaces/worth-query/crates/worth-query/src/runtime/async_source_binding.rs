use std::sync::Arc;

use worth_runtime_bridge::facade::{
    AdmittedBridgeAsyncRequestIdentity, BridgeAsyncCompletionState, BridgeAsyncRequestIdentity,
    BridgeAsyncRequestRuntimeIdentity, BridgeAsyncSourceDeclarationIdentity,
    BridgeMixedCauseAsyncResultCause, BridgeMixedCauseAsyncResultTransition,
};

use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

use super::WorthQueryRuntimeAsyncResultState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryAsyncSourceBindingErrorKind {
    MissingLiveSubscription,
    MissingBinding,
    ForeignBridgeRuntime,
    ForeignDeclaration,
    ForeignRequest,
    PredecessorMismatch,
    InadmissibleDisposition,
    IllegalResultTransition,
    ProjectionFailed,
    InitialStateAlreadyDelivered,
    InitialStateNoLongerPending,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAsyncSourceBindingError {
    kind: WorthQueryAsyncSourceBindingErrorKind,
    detail: Arc<str>,
}

impl WorthQueryAsyncSourceBindingError {
    pub(super) fn new(
        kind: WorthQueryAsyncSourceBindingErrorKind,
        detail: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    pub fn kind(&self) -> WorthQueryAsyncSourceBindingErrorKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        self.detail.as_ref()
    }
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAsyncResultTransitionBatch {
    runtime_provenance: super::WorthQueryRuntimeProvenance,
    view_name: Arc<str>,
    binding_identity: WorthQueryEvidenceIdentity,
    expected_basis_identity: WorthQueryEvidenceIdentity,
    expected_checkpoint_identity: WorthQueryEvidenceIdentity,
    remask_posture: Option<super::WorthQueryRuntimeRemaskPosture>,
    states: Vec<WorthQueryRuntimeAsyncResultState>,
    suppressed_duplicate_count: usize,
}

impl WorthQueryAsyncResultTransitionBatch {
    pub(super) fn admitted(
        runtime_provenance: super::WorthQueryRuntimeProvenance,
        view_name: impl Into<Arc<str>>,
        binding_identity: WorthQueryEvidenceIdentity,
        expected_basis_identity: WorthQueryEvidenceIdentity,
        expected_checkpoint_identity: WorthQueryEvidenceIdentity,
        remask_posture: Option<super::WorthQueryRuntimeRemaskPosture>,
        states: Vec<WorthQueryRuntimeAsyncResultState>,
        suppressed_duplicate_count: usize,
    ) -> Self {
        Self {
            runtime_provenance,
            view_name: view_name.into(),
            binding_identity,
            expected_basis_identity,
            expected_checkpoint_identity,
            remask_posture,
            states,
            suppressed_duplicate_count,
        }
    }

    pub fn runtime_provenance(&self) -> super::WorthQueryRuntimeProvenance {
        self.runtime_provenance
    }

    pub fn view_name(&self) -> &str {
        self.view_name.as_ref()
    }

    pub fn binding_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.binding_identity
    }

    pub fn expected_basis_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.expected_basis_identity
    }

    pub fn expected_checkpoint_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.expected_checkpoint_identity
    }

    pub fn remask_posture(&self) -> Option<&super::WorthQueryRuntimeRemaskPosture> {
        self.remask_posture.as_ref()
    }

    pub fn states(&self) -> &[WorthQueryRuntimeAsyncResultState] {
        &self.states
    }

    pub fn suppressed_duplicate_count(&self) -> usize {
        self.suppressed_duplicate_count
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct WorthQueryRuntimeAsyncSourceBinding {
    binding_identity: WorthQueryEvidenceIdentity,
    bridge_runtime_identity: BridgeAsyncRequestRuntimeIdentity,
    declaration_identity: BridgeAsyncSourceDeclarationIdentity,
    declaration_identity_reporting: Arc<str>,
    current_request_identity: BridgeAsyncRequestIdentity,
    truth_view_basis_digest: Arc<str>,
    request_generation: u64,
    request_attempt: u64,
    initial_state_delivered: bool,
}

impl WorthQueryRuntimeAsyncSourceBinding {
    pub(super) fn admit(view_name: &str, request: &AdmittedBridgeAsyncRequestIdentity) -> Self {
        let declaration_identity = request.lowered().declaration_identity();
        let current_request_identity = request.request_identity();
        let binding_identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::RuntimeStateSnapshot)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "worth_query_async_source_binding_v1",
                )
                .field_shape(WorthQueryEvidenceTag::new("live_target"), view_name)
                .field_shape(
                    WorthQueryEvidenceTag::new("bridge_declaration"),
                    request.lowered().declaration_identity_for_reporting(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("bridge_request"),
                    request.request_identity_for_reporting(),
                )
                .seal();
        Self {
            binding_identity,
            bridge_runtime_identity: request.bridge_runtime_identity(),
            declaration_identity: declaration_identity.clone(),
            declaration_identity_reporting: Arc::from(
                request.lowered().declaration_identity_for_reporting(),
            ),
            current_request_identity: current_request_identity.clone(),
            truth_view_basis_digest: Arc::from(
                request
                    .basis_binding()
                    .truth_view_basis()
                    .digest()
                    .to_owned(),
            ),
            request_generation: request.request_handle().generation().get(),
            request_attempt: request.attempt().get(),
            initial_state_delivered: false,
        }
    }

    pub(super) fn binding_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.binding_identity
    }

    pub(super) fn declaration_identity_reference(&self) -> &BridgeAsyncSourceDeclarationIdentity {
        &self.declaration_identity
    }

    pub(super) fn current_request_identity_reference(&self) -> &BridgeAsyncRequestIdentity {
        &self.current_request_identity
    }

    pub(super) fn current_basis_identity(&self) -> WorthQueryEvidenceIdentity {
        async_source_basis_identity(
            self.declaration_identity_reporting.as_ref(),
            self.truth_view_basis_digest.as_ref(),
        )
    }

    pub(super) fn current_generation_identity(&self) -> WorthQueryEvidenceIdentity {
        async_source_generation_identity(
            self.declaration_identity_reporting.as_ref(),
            self.request_generation,
        )
    }

    pub(super) fn take_initial_state_delivery(&mut self) -> bool {
        if self.initial_state_delivered {
            return false;
        }
        self.initial_state_delivered = true;
        true
    }

    pub(super) fn validate_and_advance(
        &mut self,
        transition: &BridgeMixedCauseAsyncResultTransition,
    ) -> Result<(), WorthQueryAsyncSourceBindingError> {
        if transition.bridge_runtime_identity() != self.bridge_runtime_identity {
            return Err(binding_error(
                WorthQueryAsyncSourceBindingErrorKind::ForeignBridgeRuntime,
                transition,
            ));
        }
        if transition.declaration_identity_reference() != &self.declaration_identity {
            return Err(binding_error(
                WorthQueryAsyncSourceBindingErrorKind::ForeignDeclaration,
                transition,
            ));
        }
        match transition.predecessor_request_identity_reference() {
            Some(predecessor) if predecessor != &self.current_request_identity => {
                return Err(binding_error(
                    WorthQueryAsyncSourceBindingErrorKind::PredecessorMismatch,
                    transition,
                ));
            }
            Some(_) => {
                self.current_request_identity = transition.request_identity_reference().clone();
                self.truth_view_basis_digest =
                    Arc::from(transition.truth_view_basis_digest().to_owned());
                self.request_generation = transition.request_generation();
                self.request_attempt = transition.request_attempt();
            }
            None if transition.request_identity_reference() != &self.current_request_identity
                && !permits_retired_request_observation(transition) =>
            {
                return Err(binding_error(
                    WorthQueryAsyncSourceBindingErrorKind::ForeignRequest,
                    transition,
                ));
            }
            None => {}
        }
        Ok(())
    }
}

fn permits_retired_request_observation(transition: &BridgeMixedCauseAsyncResultTransition) -> bool {
    matches!(
        transition.cause(),
        BridgeMixedCauseAsyncResultCause::ClassifiedDenied { .. }
            | BridgeMixedCauseAsyncResultCause::Completion(BridgeAsyncCompletionState::Denied(_))
    )
}

pub(super) fn transition_basis_identity(
    transition: &BridgeMixedCauseAsyncResultTransition,
) -> WorthQueryEvidenceIdentity {
    async_source_basis_identity(
        transition.declaration_identity(),
        transition.truth_view_basis_digest(),
    )
}

pub(super) fn transition_generation_identity(
    transition: &BridgeMixedCauseAsyncResultTransition,
) -> WorthQueryEvidenceIdentity {
    async_source_generation_identity(
        transition.declaration_identity(),
        transition.request_generation(),
    )
}

fn async_source_basis_identity(
    declaration: &str,
    truth_view_basis_digest: &str,
) -> WorthQueryEvidenceIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::RuntimeStateSnapshot)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "worth_query_bridge_async_basis_v1",
        )
        .field_shape(
            WorthQueryEvidenceTag::new("bridge_declaration"),
            declaration,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("truth_view_basis"),
            truth_view_basis_digest,
        )
        .seal()
}

fn async_source_generation_identity(
    declaration: &str,
    request_generation: u64,
) -> WorthQueryEvidenceIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::RuntimeStateSnapshot)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "worth_query_bridge_async_generation_v1",
        )
        .field_shape(
            WorthQueryEvidenceTag::new("bridge_declaration"),
            declaration,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("request_generation"),
            request_generation.to_string(),
        )
        .seal()
}

fn binding_error(
    kind: WorthQueryAsyncSourceBindingErrorKind,
    transition: &BridgeMixedCauseAsyncResultTransition,
) -> WorthQueryAsyncSourceBindingError {
    WorthQueryAsyncSourceBindingError::new(
        kind,
        format!(
            "bridge async transition `{}` for request `{}` did not match the live binding",
            transition.source_identity(),
            transition.request_identity(),
        ),
    )
}
