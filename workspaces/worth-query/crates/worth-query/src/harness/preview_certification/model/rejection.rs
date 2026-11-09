#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreviewFailureClass {
    EvidenceIdentityDerivationDenied(
        crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
    ),
    UnsupportedPreviewFamily,
    InvalidPreviewBasis,
    StoreBackedRouteForbidden,
    StaleOrInactivePreviewLifecycle,
    PreviewLiveDriftDenied,
    PreviewLiveBroadFallbackForbidden,
    WorkflowFoundationAuthorityDenied,
    PromotionLinkageMismatch,
    PreviewShapeMismatchDenied,
}

impl PreviewFailureClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EvidenceIdentityDerivationDenied(_) => "evidence-identity-derivation-denied",
            Self::UnsupportedPreviewFamily => "unsupported-preview-family",
            Self::InvalidPreviewBasis => "invalid-preview-basis",
            Self::StoreBackedRouteForbidden => "store-backed-route-forbidden",
            Self::StaleOrInactivePreviewLifecycle => "stale-or-inactive-preview-lifecycle",
            Self::PreviewLiveDriftDenied => "preview-live-drift-denied",
            Self::PreviewLiveBroadFallbackForbidden => "preview-live-broad-fallback-forbidden",
            Self::WorkflowFoundationAuthorityDenied => "workflow-foundation-authority-denied",
            Self::PromotionLinkageMismatch => "promotion-linkage-mismatch",
            Self::PreviewShapeMismatchDenied => "preview-shape-mismatch-denied",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewCertificationRejection {
    pub failure_class: PreviewFailureClass,
    pub counters: Option<crate::preview::PreviewBindingCounters>,
    pub execution_counters: Option<crate::preview::PreviewExecutionCounters>,
    pub comparison_counters: Option<crate::preview::PreviewComparisonCounters>,
    pub preview_live_counters: Option<crate::preview::PreviewLiveCounters>,
}
