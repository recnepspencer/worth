use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::memory_workspace::WorthQueryWorkspaceError;

use super::super::{WorthQueryAspectTouch, WorthQueryAuthoredAspectMutation};
use super::denied_aspect_touch::WorthQueryDeniedAspectTouch;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryExistingTruthAssertionMode {
    RetainedAuthoritativeAssertion,
    BackendVerifiedAssertion,
}

impl WorthQueryExistingTruthAssertionMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RetainedAuthoritativeAssertion => "retained_authoritative_assertion",
            Self::BackendVerifiedAssertion => "backend_verified_assertion",
        }
    }
}

impl std::fmt::Display for WorthQueryExistingTruthAssertionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryExistingTruthAssertionDenialKind {
    Handle(worth_query_execution::facade::primary_graph::WorthQueryHandleDenial),
    BackendVerificationUnsupported,
    ClearAssertionUnsupported,
    MissingAssertedAspect,
    AssertedValueMismatch,
}

impl WorthQueryExistingTruthAssertionDenialKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Handle(_) => "handle_denied",
            Self::BackendVerificationUnsupported => "backend_verification_unsupported",
            Self::ClearAssertionUnsupported => "clear_assertion_unsupported",
            Self::MissingAssertedAspect => "missing_asserted_aspect",
            Self::AssertedValueMismatch => "asserted_value_mismatch",
        }
    }
}

impl std::fmt::Display for WorthQueryExistingTruthAssertionDenialKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryExistingTruthAssertionDenial {
    binding: crate::runtime::WorthQueryExistingTruthTargetBinding,
    kind: WorthQueryExistingTruthAssertionDenialKind,
    asserted_aspect_touch: Option<WorthQueryDeniedAspectTouch>,
    expected_terminal_value_digest: Option<String>,
    found_terminal_value_digest: Option<String>,
    message: String,
    denial_digest: String,
}

impl WorthQueryExistingTruthAssertionDenial {
    pub fn new(
        binding: &crate::runtime::WorthQueryExistingTruthTargetBinding,
        kind: WorthQueryExistingTruthAssertionDenialKind,
        asserted_aspect_touch: Option<WorthQueryAspectTouch>,
        expected_terminal_value_digest: Option<String>,
        found_terminal_value_digest: Option<String>,
        message: impl Into<String>,
    ) -> Self {
        let asserted_aspect_touch =
            asserted_aspect_touch.map(WorthQueryDeniedAspectTouch::Admitted);
        Self::from_denied_aspect_touch(
            binding,
            kind,
            asserted_aspect_touch,
            expected_terminal_value_digest,
            found_terminal_value_digest,
            message,
        )
    }

    fn from_denied_aspect_touch(
        binding: &crate::runtime::WorthQueryExistingTruthTargetBinding,
        kind: WorthQueryExistingTruthAssertionDenialKind,
        asserted_aspect_touch: Option<WorthQueryDeniedAspectTouch>,
        expected_terminal_value_digest: Option<String>,
        found_terminal_value_digest: Option<String>,
        message: impl Into<String>,
    ) -> Self {
        let message = message.into();
        let asserted_aspect_touch_digest = asserted_aspect_touch
            .as_ref()
            .map(WorthQueryDeniedAspectTouch::admitted_touch_digest_part);
        let denial_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
                .field_shape(
                    WorthQueryEvidenceTag::new("role"),
                    "existing-truth-assertion-denial",
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("family"),
                    binding.family().as_str(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("authoritative"),
                    binding.authoritative_identity().evidence_identity(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("resolved"),
                    &binding.resolved_target_identity().evidence_identity(),
                )
                .optional_value(
                    WorthQueryEvidenceTag::new("collection"),
                    binding.terminal_target_collection_projection(),
                )
                .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
                .optional_value(
                    WorthQueryEvidenceTag::new("aspect_touch"),
                    asserted_aspect_touch_digest.as_deref(),
                )
                .optional_value(
                    WorthQueryEvidenceTag::new("expected_terminal_value"),
                    expected_terminal_value_digest.as_deref(),
                )
                .optional_value(
                    WorthQueryEvidenceTag::new("found_terminal_value"),
                    found_terminal_value_digest.as_deref(),
                )
                .field_value(WorthQueryEvidenceTag::new("message"), &message)
                .seal()
                .as_str()
                .to_string();
        Self {
            binding: binding.clone(),
            kind,
            asserted_aspect_touch,
            expected_terminal_value_digest,
            found_terminal_value_digest,
            message,
            denial_digest,
        }
    }

    pub fn binding(&self) -> &crate::runtime::WorthQueryExistingTruthTargetBinding {
        &self.binding
    }

    pub fn kind(&self) -> WorthQueryExistingTruthAssertionDenialKind {
        self.kind
    }

    pub fn asserted_aspect_touch(&self) -> Option<&WorthQueryAspectTouch> {
        self.asserted_aspect_touch
            .as_ref()
            .and_then(WorthQueryDeniedAspectTouch::admitted_touch)
    }

    pub fn expected_terminal_value_digest(&self) -> Option<&str> {
        self.expected_terminal_value_digest.as_deref()
    }

    pub fn found_terminal_value_digest(&self) -> Option<&str> {
        self.found_terminal_value_digest.as_deref()
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn denial_digest(&self) -> &str {
        &self.denial_digest
    }
}

impl std::fmt::Display for WorthQueryExistingTruthAssertionDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "existing-truth assertion denied for authoritative `{}` during {}: {}",
            self.binding.authoritative_identity().as_str(),
            self.kind,
            self.message
        )
    }
}

impl std::error::Error for WorthQueryExistingTruthAssertionDenial {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryVerifiedExistingTruthAssertion {
    asserted_aspect_count: usize,
    verification_digest: WorthQueryEvidenceIdentity,
    verified_assumption_set: crate::runtime::WorthQueryVerifiedAssumptionSet,
}

impl WorthQueryVerifiedExistingTruthAssertion {
    pub(in crate::runtime) fn from_snapshot_identity(
        binding: &crate::runtime::WorthQueryExistingTruthTargetBinding,
        aspects: &[WorthQueryAuthoredAspectMutation],
        snapshot_identity: &WorthQuerySnapshotIdentity,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        Self::new(binding, aspects, snapshot_identity.clone())
    }

    pub(crate) fn new(
        binding: &crate::runtime::WorthQueryExistingTruthTargetBinding,
        aspects: &[WorthQueryAuthoredAspectMutation],
        snapshot_identity: WorthQuerySnapshotIdentity,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        let asserted_aspect_count = aspects.len();
        let asserted_aspects = aspects
            .iter()
            .map(|aspect| WorthQueryAspectTouch::from_parsed_target(aspect.parsed_target().clone()))
            .collect::<Vec<_>>();
        let cleared_assertion_count = aspects
            .iter()
            .filter(|aspect| aspect.clears_existing_value())
            .count();
        let aspect_evidence_rows = aspects
            .iter()
            .map(existing_truth_assertion_aspect_evidence)
            .collect::<Vec<_>>();
        let verified_assumption_set = crate::runtime::WorthQueryVerifiedAssumptionSet::new(
            binding.binding_evidence_identity().clone(),
            asserted_aspects,
            aspect_evidence_rows.clone(),
            cleared_assertion_count,
            snapshot_identity,
        );
        let verification_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
                .field_shape(
                    WorthQueryEvidenceTag::new("role"),
                    "existing-truth-assertion-verification",
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("binding"),
                    binding.binding_evidence_identity(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("assumption_set"),
                    verified_assumption_set.verified_assumption_evidence_digest(),
                )
                .field_evidence_identity_sequence(
                    WorthQueryEvidenceTag::new("aspect"),
                    aspect_evidence_rows.iter(),
                )
                .seal();
        Ok(Self {
            asserted_aspect_count,
            verification_digest,
            verified_assumption_set,
        })
    }

    pub fn asserted_aspect_count(&self) -> usize {
        self.asserted_aspect_count
    }

    pub fn verification_digest(&self) -> &str {
        self.verification_digest.as_str()
    }

    pub fn verification_evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.verification_digest
    }

    pub fn verified_assumption_set(&self) -> &crate::runtime::WorthQueryVerifiedAssumptionSet {
        &self.verified_assumption_set
    }

    pub fn assumption_snapshot_digest(&self) -> &str {
        self.verified_assumption_set.assumption_snapshot_digest()
    }

    pub fn verified_precondition_digest(&self) -> &str {
        self.verified_assumption_set.verified_precondition_digest()
    }

    pub fn verification_read_set_breadth(
        &self,
    ) -> &crate::runtime::WorthQueryVerificationReadSetBreadth {
        self.verified_assumption_set.verification_read_set_breadth()
    }
}

fn existing_truth_assertion_aspect_evidence(
    aspect: &WorthQueryAuthoredAspectMutation,
) -> WorthQueryEvidenceIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
        .field_shape(
            WorthQueryEvidenceTag::new("role"),
            "existing-truth-assertion-aspect",
        )
        .field_value(
            WorthQueryEvidenceTag::new("aspect_touch"),
            aspect.aspect_touch().admitted_touch_digest_part(),
        )
        .field_bool(
            WorthQueryEvidenceTag::new("clears_existing_value"),
            aspect.clears_existing_value(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("value"),
            aspect.terminal_digest_material(),
        )
        .seal()
}
