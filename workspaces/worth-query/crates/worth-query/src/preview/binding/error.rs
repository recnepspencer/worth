use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::preview::binding::PreviewBindingCounters;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreviewBindingFailureClass {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    InvalidPreviewBasis,
    UnsupportedPreviewQueryFamily,
    StaleOrInactivePreviewLifecycle,
    RawBranchAliasPreviewForbidden,
    MissingExecutionRecordIdentity,
    PromotionLinkageMismatch,
    StoreBackedRouteForbidden,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewBindingError {
    failure_class: PreviewBindingFailureClass,
    message: &'static str,
    counters: PreviewBindingCounters,
}

impl PreviewBindingError {
    pub(super) fn identity_refused(cause: IdentityDerivationDenial) -> Self {
        Self::new(
            PreviewBindingFailureClass::EvidenceIdentityDerivationDenied(cause),
            "preview binding evidence identity refused",
            PreviewBindingCounters::for_identity_refusal(),
        )
    }
    pub(super) fn new(
        failure_class: PreviewBindingFailureClass,
        message: &'static str,
        counters: PreviewBindingCounters,
    ) -> Self {
        Self {
            failure_class,
            message,
            counters,
        }
    }

    pub fn failure_class(&self) -> &PreviewBindingFailureClass {
        &self.failure_class
    }

    pub fn message(&self) -> &'static str {
        self.message
    }

    pub fn counters(&self) -> &PreviewBindingCounters {
        &self.counters
    }
}
