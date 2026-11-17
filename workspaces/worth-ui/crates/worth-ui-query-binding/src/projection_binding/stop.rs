use std::sync::Arc;

use worth_query::facade::runtime::WorthQueryEvidenceIdentity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiProjectionBindingStopKind {
    MissingInstalledView,
    EvidenceIdentity(worth_query::facade::runtime::WorthQueryEvidenceIdentityDerivationDenial),
    ShapeMismatch,
    SchemaMismatch,
    NativeFamilyMismatch,
    PayloadShapeMismatch,
    RowIdentityMismatch,
    LifecycleMismatch,
    WrongWorld,
    RebindRequired,
    BudgetExceeded,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UiProjectionBindingStopReceipt {
    kind: UiProjectionBindingStopKind,
    attempt_identity: WorthQueryEvidenceIdentity,
    predecessor_binding: Option<crate::UiQueryIdentityReportingProjection>,
    summary: Arc<str>,
}

impl UiProjectionBindingStopReceipt {
    pub(crate) fn initial(
        kind: UiProjectionBindingStopKind,
        attempt_identity: WorthQueryEvidenceIdentity,
        summary: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            kind,
            attempt_identity,
            predecessor_binding: None,
            summary: summary.into(),
        }
    }

    pub(crate) fn replacement(
        kind: UiProjectionBindingStopKind,
        attempt_identity: WorthQueryEvidenceIdentity,
        predecessor_binding: crate::UiQueryIdentityReportingProjection,
        summary: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            kind,
            attempt_identity,
            predecessor_binding: Some(predecessor_binding),
            summary: summary.into(),
        }
    }

    pub fn kind(&self) -> UiProjectionBindingStopKind {
        self.kind
    }

    pub fn attempt_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.attempt_identity
    }

    pub fn predecessor_binding(&self) -> Option<&crate::UiQueryIdentityReportingProjection> {
        self.predecessor_binding.as_ref()
    }

    pub fn summary(&self) -> &str {
        self.summary.as_ref()
    }
}
