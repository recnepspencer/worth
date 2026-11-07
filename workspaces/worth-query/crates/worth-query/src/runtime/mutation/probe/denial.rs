use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExistingTruthProbeDenialKind {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    BackendProbeUnsupported,
    ResolvedTargetUnavailable,
    MissingProbedAspect,
    UnsupportedProbedAspectValue,
}

impl WorthQueryExistingTruthProbeDenialKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EvidenceIdentityDerivationDenied(_) => "evidence_identity_derivation_denied",
            Self::BackendProbeUnsupported => "backend_probe_unsupported",
            Self::ResolvedTargetUnavailable => "resolved_target_unavailable",
            Self::MissingProbedAspect => "missing_probed_aspect",
            Self::UnsupportedProbedAspectValue => "unsupported_probed_aspect_value",
        }
    }
}

impl std::fmt::Display for WorthQueryExistingTruthProbeDenialKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryExistingTruthProbeDenial {
    binding: WorthQueryExistingTruthTargetBinding,
    kind: WorthQueryExistingTruthProbeDenialKind,
    probed_aspect_touch: Option<WorthQueryDeniedAspectTouch>,
    message: String,
    denial_digest: Result<String, IdentityDerivationDenial>,
}

impl WorthQueryExistingTruthProbeDenial {
    pub fn new(
        binding: &WorthQueryExistingTruthTargetBinding,
        kind: WorthQueryExistingTruthProbeDenialKind,
        probed_aspect_touch: Option<WorthQueryAspectTouch>,
        message: impl Into<String>,
    ) -> Self {
        let probed_aspect_touch = probed_aspect_touch.map(WorthQueryDeniedAspectTouch::Admitted);
        Self::from_denied_aspect_touch(binding, kind, probed_aspect_touch, message)
    }

    fn from_admitted_touch(
        binding: &WorthQueryExistingTruthTargetBinding,
        kind: WorthQueryExistingTruthProbeDenialKind,
        probed_aspect_touch: WorthQueryAspectTouch,
        message: impl Into<String>,
    ) -> Self {
        Self::from_denied_aspect_touch(
            binding,
            kind,
            Some(WorthQueryDeniedAspectTouch::Admitted(probed_aspect_touch)),
            message,
        )
    }

    fn from_denied_aspect_touch(
        binding: &WorthQueryExistingTruthTargetBinding,
        kind: WorthQueryExistingTruthProbeDenialKind,
        probed_aspect_touch: Option<WorthQueryDeniedAspectTouch>,
        message: impl Into<String>,
    ) -> Self {
        let message = message.into();
        let probed_aspect_touch_digest = probed_aspect_touch
            .as_ref()
            .map(WorthQueryDeniedAspectTouch::admitted_touch_digest_part);
        let target_collection_identity = binding.target_collection_identity();
        let denial_digest = (|| {
            Ok(worth_query_evidence_identity(
                WorthQueryEvidenceScope::MutationEvidenceAggregateDigest,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("role"),
                "existing-truth-probe-denial",
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
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("collection"),
                target_collection_identity
                    .map(WorthQueryMutationTargetCollectionIdentity::evidence_identity),
            )
            .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
            .optional_value(
                WorthQueryEvidenceTag::new("aspect_touch"),
                probed_aspect_touch_digest.as_deref(),
            )
            .field_value(WorthQueryEvidenceTag::new("message"), &message)
            .seal()?
            .as_str()
            .to_string())
        })();
        Self {
            binding: binding.clone(),
            kind,
            probed_aspect_touch,
            message,
            denial_digest,
        }
    }

    pub fn binding(&self) -> &WorthQueryExistingTruthTargetBinding {
        &self.binding
    }

    pub fn kind(&self) -> WorthQueryExistingTruthProbeDenialKind {
        self.kind
    }

    pub fn probed_aspect_touch(&self) -> Option<&WorthQueryAspectTouch> {
        self.probed_aspect_touch
            .as_ref()
            .and_then(WorthQueryDeniedAspectTouch::admitted_touch)
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn denial_digest(&self) -> Result<&str, IdentityDerivationDenial> {
        Ok(self.denial_digest.as_ref().map_err(|cause| *cause)?)
    }
}

impl std::fmt::Display for WorthQueryExistingTruthProbeDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "existing-truth probe denied for authoritative `{}` during {}: {}",
            self.binding.authoritative_identity().as_str(),
            self.kind,
            self.message
        )
    }
}

impl std::error::Error for WorthQueryExistingTruthProbeDenial {}
