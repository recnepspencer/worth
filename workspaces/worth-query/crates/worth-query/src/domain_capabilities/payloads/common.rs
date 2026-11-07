use worth_foundational::FoundationalDiagnosticOutcomeKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDomainCapabilityCategory {
    Admission,
    SupportTraceability,
    InvariantCapability,
    WorkflowPreview,
    ContinuityLineage,
    ConsequenceAftermath,
    ExplanationInspection,
}

impl WorthQueryDomainCapabilityCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admission => "admission",
            Self::SupportTraceability => "support-traceability",
            Self::InvariantCapability => "invariant-capability",
            Self::WorkflowPreview => "workflow-preview",
            Self::ContinuityLineage => "continuity-lineage",
            Self::ConsequenceAftermath => "consequence-aftermath",
            Self::ExplanationInspection => "explanation-inspection",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDomainCapabilitySemanticPosture {
    AdmissionAdvisory,
    AdmissionViolation,
    AdmissionSupportOnly,
    SupportDeclarationSupport,
    SupportDeclarationTraceability,
    SupportNarrowedSupport,
    InvariantCapabilityGap,
    InvariantDenial,
    InvariantSupportSummary,
    InvariantRegistration,
    WorkflowPreviewOnly,
    WorkflowPromotionEligible,
    WorkflowConfirmationRequired,
    WorkflowDiscardRequired,
    ContinuityPreserved,
    ContinuitySplit,
    ContinuityReplaced,
    ContinuityCorrespondenceOnly,
    AftermathEstablishesFact,
    AftermathConsumesFact,
    AftermathDeclaresResidue,
    ExplanationRequiresContext,
    ExplanationExplainsFallback,
    ExplanationExplainsAmbiguity,
}

impl WorthQueryDomainCapabilitySemanticPosture {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AdmissionAdvisory => "advisory",
            Self::AdmissionViolation => "violation",
            Self::AdmissionSupportOnly => "support-only",
            Self::SupportDeclarationSupport => "declaration-support",
            Self::SupportDeclarationTraceability => "declaration-traceability",
            Self::SupportNarrowedSupport => "narrowed-support",
            Self::InvariantCapabilityGap => "capability-gap",
            Self::InvariantDenial => "invariant-denial",
            Self::InvariantSupportSummary => "support-summary",
            Self::InvariantRegistration => "invariant-registration",
            Self::WorkflowPreviewOnly => "preview-only",
            Self::WorkflowPromotionEligible => "promotion-eligible",
            Self::WorkflowConfirmationRequired => "confirmation-required",
            Self::WorkflowDiscardRequired => "discard-required",
            Self::ContinuityPreserved => "preserved",
            Self::ContinuitySplit => "split",
            Self::ContinuityReplaced => "replaced",
            Self::ContinuityCorrespondenceOnly => "correspondence-only",
            Self::AftermathEstablishesFact => "establishes-fact",
            Self::AftermathConsumesFact => "consumes-fact",
            Self::AftermathDeclaresResidue => "declares-residue",
            Self::ExplanationRequiresContext => "requires-context",
            Self::ExplanationExplainsFallback => "explains-fallback",
            Self::ExplanationExplainsAmbiguity => "explains-ambiguity",
        }
    }

    pub const fn outcome_kind(self) -> FoundationalDiagnosticOutcomeKind {
        match self {
            Self::AdmissionAdvisory => FoundationalDiagnosticOutcomeKind::Advisory,
            Self::AdmissionViolation => FoundationalDiagnosticOutcomeKind::Violation,
            Self::AdmissionSupportOnly => FoundationalDiagnosticOutcomeKind::Partial,
            Self::SupportDeclarationSupport => FoundationalDiagnosticOutcomeKind::Accepted,
            Self::SupportDeclarationTraceability => FoundationalDiagnosticOutcomeKind::Accepted,
            Self::SupportNarrowedSupport => FoundationalDiagnosticOutcomeKind::Partial,
            Self::InvariantCapabilityGap => FoundationalDiagnosticOutcomeKind::Unsupported,
            Self::InvariantDenial => FoundationalDiagnosticOutcomeKind::Violation,
            Self::InvariantSupportSummary => FoundationalDiagnosticOutcomeKind::Accepted,
            Self::InvariantRegistration => FoundationalDiagnosticOutcomeKind::Accepted,
            Self::WorkflowPreviewOnly => FoundationalDiagnosticOutcomeKind::Advisory,
            Self::WorkflowPromotionEligible => FoundationalDiagnosticOutcomeKind::Accepted,
            Self::WorkflowConfirmationRequired => FoundationalDiagnosticOutcomeKind::Advisory,
            Self::WorkflowDiscardRequired => FoundationalDiagnosticOutcomeKind::Denied,
            Self::ContinuityPreserved => FoundationalDiagnosticOutcomeKind::Accepted,
            Self::ContinuitySplit => FoundationalDiagnosticOutcomeKind::Advisory,
            Self::ContinuityReplaced => FoundationalDiagnosticOutcomeKind::Advisory,
            Self::ContinuityCorrespondenceOnly => FoundationalDiagnosticOutcomeKind::Partial,
            Self::AftermathEstablishesFact => FoundationalDiagnosticOutcomeKind::Accepted,
            Self::AftermathConsumesFact => FoundationalDiagnosticOutcomeKind::Accepted,
            Self::AftermathDeclaresResidue => FoundationalDiagnosticOutcomeKind::Partial,
            Self::ExplanationRequiresContext => FoundationalDiagnosticOutcomeKind::Advisory,
            Self::ExplanationExplainsFallback => FoundationalDiagnosticOutcomeKind::Advisory,
            Self::ExplanationExplainsAmbiguity => FoundationalDiagnosticOutcomeKind::Advisory,
        }
    }

    pub const fn is_policy_or_inferred(self) -> bool {
        !matches!(
            self,
            Self::AdmissionAdvisory
                | Self::AdmissionViolation
                | Self::AdmissionSupportOnly
                | Self::SupportDeclarationSupport
                | Self::InvariantRegistration
        )
    }
}

mod sealed {
    pub trait Sealed {}
}

pub(crate) use sealed::Sealed as SealedPayload;

pub trait WorthQueryDomainCapabilityPayload: Clone + SealedPayload {
    fn category(&self) -> WorthQueryDomainCapabilityCategory;
    fn posture_label(&self) -> &'static str;
    fn semantic_posture(&self) -> WorthQueryDomainCapabilitySemanticPosture;
    fn semantic_code(&self) -> &str;
    fn detail(&self) -> &str;
    fn payload_digest(&self) -> &str;
    fn payload_identity(&self) -> &crate::WorthQueryEvidenceIdentity;
}

macro_rules! define_payload_family {
    (
        $posture:ident,
        $payload:ident,
        $category:expr,
        { $($variant:ident => $label:literal),+ $(,)? }
    ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $posture {
            $($variant),+
        }

        impl $posture {
            pub fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $label),+
                }
            }
        }

        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $payload {
            posture: $posture,
            semantic_code: String,
            detail: String,
            payload_identity: $crate::WorthQueryEvidenceIdentity,
        }

        impl $payload {
            pub fn new(
                posture: $posture,
                semantic_code: impl Into<String>,
                detail: impl Into<String>,
            ) -> Result<Self, $crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
                let semantic_code = semantic_code.into();
                let detail = detail.into();
                let payload_identity = $crate::domain_capabilities::identity::domain_capability_scope_encoder(
                    "worth_query_domain_capability_payload_v3",
                )
                    .field_shape($crate::WorthQueryEvidenceTag::new("category"), $category.as_str())
                    .field_shape($crate::WorthQueryEvidenceTag::new("posture"), posture.as_str())
                    .field_shape($crate::WorthQueryEvidenceTag::new("semantic_code"), &semantic_code)
                    .field_shape($crate::WorthQueryEvidenceTag::new("detail"), &detail)
                    .seal()?;
                Ok(Self {
                    posture,
                    semantic_code,
                    detail,
                    payload_identity,
                })
            }

            pub fn category(&self) -> WorthQueryDomainCapabilityCategory {
                $category
            }

            pub fn posture(&self) -> $posture {
                self.posture
            }

            pub fn semantic_code(&self) -> &str {
                &self.semantic_code
            }

            pub fn detail(&self) -> &str {
                &self.detail
            }

            pub fn payload_digest(&self) -> &str {
                self.payload_identity.as_str()
            }

            pub fn payload_for_reporting(&self) -> &str {
                self.payload_identity.as_str()
            }

            pub fn payload_identity(&self) -> &$crate::WorthQueryEvidenceIdentity {
                &self.payload_identity
            }
        }

        impl $crate::domain_capabilities::payloads::common::SealedPayload for $payload {}

        impl $crate::domain_capabilities::payloads::common::WorthQueryDomainCapabilityPayload
            for $payload
        {
            fn category(&self) -> WorthQueryDomainCapabilityCategory {
                self.category()
            }

            fn posture_label(&self) -> &'static str {
                self.posture().as_str()
            }

            fn semantic_posture(&self) -> WorthQueryDomainCapabilitySemanticPosture {
                self.posture().semantic_posture()
            }

            fn semantic_code(&self) -> &str {
                self.semantic_code()
            }

            fn detail(&self) -> &str {
                self.detail()
            }

            fn payload_digest(&self) -> &str {
                self.payload_digest()
            }

            fn payload_identity(&self) -> &crate::WorthQueryEvidenceIdentity {
                &self.payload_identity
            }
        }
    };
}

pub(crate) use define_payload_family;
