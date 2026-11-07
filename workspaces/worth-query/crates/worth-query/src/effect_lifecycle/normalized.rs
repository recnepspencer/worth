use crate::basis_lifecycle::{BasisAuthorityPosture, BasisFamily, BasisLifecyclePosture};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::workflow::{
    MergeLoweringInput, MutationLoweringInput, WorkflowContextBinding, WorkflowDeclarationRequest,
    WritebackLoweringInput,
};

use super::authoring_basis::EffectAuthoringBasis;
use super::counters::EffectLifecycleCounters;
use super::taxonomy::{EffectAuthorityLane, EffectFamily, EffectIntentDenialKind};

#[derive(Clone, Debug, PartialEq)]
pub enum EffectOperationInput {
    Mutation(MutationLoweringInput),
    Merge(MergeLoweringInput),
    Writeback(WritebackLoweringInput),
}

#[derive(Clone, Debug, PartialEq)]
pub struct NormalizedEffectIntent {
    family: EffectFamily,
    authority_lane: EffectAuthorityLane,
    basis_family: BasisFamily,
    basis_authority: BasisAuthorityPosture,
    basis_lifecycle: BasisLifecyclePosture,
    capability_identity: WorthQueryEvidenceIdentity,
    scoped_basis_identity: WorthQueryEvidenceIdentity,
    expected_lower_runtime_binding_identity: Option<WorthQueryEvidenceIdentity>,
    workflow_binding: WorkflowContextBinding,
    workflow_request: WorkflowDeclarationRequest,
    operation_input: EffectOperationInput,
    source_path: &'static str,
    normalized_identity: WorthQueryEvidenceIdentity,
    counters: EffectLifecycleCounters,
}

impl NormalizedEffectIntent {
    pub(crate) fn new(
        authoring_basis: &EffectAuthoringBasis,
        family: EffectFamily,
        authority_lane: EffectAuthorityLane,
        workflow_binding: WorkflowContextBinding,
        workflow_request: WorkflowDeclarationRequest,
        operation_input: EffectOperationInput,
        source_path: &'static str,
    ) -> Result<Self, IdentityDerivationDenial> {
        let capability_identity = authoring_basis.capability_identity()?;
        let scoped_basis_identity = authoring_basis.scoped_basis_identity()?;
        let expected_lower_runtime_binding_identity =
            authoring_basis.expected_lower_runtime_binding_identity()?;
        let mut normalized_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "normalized_effect_intent_v1",
                )
                .field_shape(WorthQueryEvidenceTag::new("family"), family.as_str())
                .field_shape(
                    WorthQueryEvidenceTag::new("authority_lane"),
                    authority_lane.as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("basis_family"),
                    authoring_basis.family().as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("basis_authority"),
                    authoring_basis.authority().as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("basis_lifecycle"),
                    authoring_basis.lifecycle().as_str(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("capability"),
                    &capability_identity,
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("scoped_basis"),
                    &scoped_basis_identity,
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("workflow_binding"),
                    workflow_binding.binding_identity(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("workflow_family"),
                    workflow_request.declaration_family().as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("workflow_target"),
                    workflow_request.authority_target_family().as_str(),
                )
                .field_shape(WorthQueryEvidenceTag::new("source_path"), source_path);
        if let Some(lower_runtime_binding_identity) =
            expected_lower_runtime_binding_identity.as_ref()
        {
            normalized_identity = normalized_identity.field_evidence_identity(
                WorthQueryEvidenceTag::new("lower_runtime_binding"),
                lower_runtime_binding_identity,
            );
        }
        let normalized_identity = normalized_identity.seal()?;

        Ok(Self {
            family,
            authority_lane,
            basis_family: authoring_basis.family(),
            basis_authority: authoring_basis.authority(),
            basis_lifecycle: authoring_basis.lifecycle(),
            capability_identity,
            scoped_basis_identity,
            expected_lower_runtime_binding_identity,
            workflow_binding,
            workflow_request,
            operation_input,
            source_path,
            normalized_identity,
            counters: EffectLifecycleCounters::normalized(1, 1, 1),
        })
    }

    pub fn family(&self) -> EffectFamily {
        self.family
    }

    pub fn authority_lane(&self) -> EffectAuthorityLane {
        self.authority_lane
    }

    pub fn basis_family(&self) -> BasisFamily {
        self.basis_family
    }

    pub fn basis_authority(&self) -> BasisAuthorityPosture {
        self.basis_authority
    }

    pub fn basis_lifecycle(&self) -> BasisLifecyclePosture {
        self.basis_lifecycle
    }

    pub fn capability_digest(&self) -> &str {
        self.capability_identity.as_str()
    }

    pub fn scoped_basis_digest(&self) -> &str {
        self.scoped_basis_identity.as_str()
    }

    pub fn capability_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.capability_identity
    }

    pub fn scoped_basis_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.scoped_basis_identity
    }

    pub fn expected_lower_runtime_binding_digest(&self) -> Option<&str> {
        self.expected_lower_runtime_binding_identity
            .as_ref()
            .map(WorthQueryEvidenceIdentity::as_str)
    }

    pub fn expected_lower_runtime_binding_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.expected_lower_runtime_binding_identity.as_ref()
    }

    pub fn workflow_binding(&self) -> &WorkflowContextBinding {
        &self.workflow_binding
    }

    pub fn workflow_request(&self) -> &WorkflowDeclarationRequest {
        &self.workflow_request
    }

    pub fn operation_input(&self) -> &EffectOperationInput {
        &self.operation_input
    }

    pub fn source_path(&self) -> &'static str {
        self.source_path
    }

    pub fn normalized_digest(&self) -> &str {
        self.normalized_identity.as_str()
    }

    pub fn normalized_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.normalized_identity
    }

    pub fn counters(&self) -> &EffectLifecycleCounters {
        &self.counters
    }

    pub(crate) fn admitted_identity(
        &self,
    ) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "admitted_effect_intent_v1",
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("normalized"),
                    &self.normalized_identity,
                )
                .seal()?,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectIntentDenial {
    denial_kind: EffectIntentDenialKind,
    message: &'static str,
    counters: EffectLifecycleCounters,
}

impl EffectIntentDenial {
    pub(crate) fn new(
        denial_kind: EffectIntentDenialKind,
        message: &'static str,
        source_path: &'static str,
        workflow_authority_target_check_count: usize,
        basis_scope_check_count: usize,
    ) -> Self {
        Self {
            denial_kind,
            message,
            counters: EffectLifecycleCounters::intent_denial(
                usize::from(!source_path.is_empty()),
                workflow_authority_target_check_count,
                basis_scope_check_count,
            ),
        }
    }

    pub fn denial_kind(&self) -> EffectIntentDenialKind {
        self.denial_kind
    }

    pub fn message(&self) -> &'static str {
        self.message
    }

    pub fn counters(&self) -> &EffectLifecycleCounters {
        &self.counters
    }
}
