use crate::basis::BasisPreflightError;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::live::{LiveExecutionError, LivePromotionError};

use super::counters::ViewShapeLiveCounters;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ViewShapeLiveFailureClass {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    BasisPreflightRejected,
    BasisInvariantRejected,
    LivePromotionRejected,
    LiveExecutionRejected,
    UnderlyingLiveFamilyMismatch,
    InspectorIdentityBindingRejected,
    FocusedInspectorWideningDenied,
    FocusedInspectorRefreshForbidden,
    GroupedRefreshForbidden,
    GroupedBaselineRequired,
    GroupedBaselineMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ViewShapeLiveError {
    failure_class: ViewShapeLiveFailureClass,
    message: String,
    counters: ViewShapeLiveCounters,
}

impl ViewShapeLiveError {
    pub(crate) fn identity_refused(
        cause: IdentityDerivationDenial,
        counters: ViewShapeLiveCounters,
    ) -> Self {
        Self::new(
            ViewShapeLiveFailureClass::EvidenceIdentityDerivationDenied(cause),
            "grouped execution evidence identity refused",
            counters,
        )
    }
    pub(crate) fn new(
        failure_class: ViewShapeLiveFailureClass,
        message: impl Into<String>,
        counters: ViewShapeLiveCounters,
    ) -> Self {
        Self {
            failure_class,
            message: message.into(),
            counters,
        }
    }

    pub fn failure_class(&self) -> &ViewShapeLiveFailureClass {
        &self.failure_class
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn counters(&self) -> &ViewShapeLiveCounters {
        &self.counters
    }
}

impl From<BasisPreflightError> for ViewShapeLiveError {
    fn from(value: BasisPreflightError) -> Self {
        Self::new(
            ViewShapeLiveFailureClass::BasisPreflightRejected,
            format!("view-shape live preflight rejected: {:?}", value),
            ViewShapeLiveCounters::default(),
        )
    }
}

impl From<LivePromotionError> for ViewShapeLiveError {
    fn from(value: LivePromotionError) -> Self {
        Self::new(
            ViewShapeLiveFailureClass::LivePromotionRejected,
            format!("view-shape live promotion rejected: {:?}", value),
            ViewShapeLiveCounters::default(),
        )
    }
}

impl From<LiveExecutionError> for ViewShapeLiveError {
    fn from(value: LiveExecutionError) -> Self {
        Self::new(
            ViewShapeLiveFailureClass::LiveExecutionRejected,
            format!("view-shape live execution rejected: {:?}", value),
            ViewShapeLiveCounters::default(),
        )
    }
}
