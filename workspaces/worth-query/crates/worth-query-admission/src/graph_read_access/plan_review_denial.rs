#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphReadPlanReviewDenialKind {
    RequiredAsyncMaterialization,
    RequiredAccessCapabilityRegistration,
    RequiredPersistentIndex,
    UnsupportedGraphIndexSupport,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadPlanReviewDenial {
    kind: WorthQueryGraphReadPlanReviewDenialKind,
}

impl WorthQueryGraphReadPlanReviewDenial {
    pub(super) const fn new(kind: WorthQueryGraphReadPlanReviewDenialKind) -> Self {
        Self { kind }
    }

    pub const fn kind(&self) -> WorthQueryGraphReadPlanReviewDenialKind {
        self.kind
    }

    pub const fn as_str(&self) -> &'static str {
        match self.kind {
            WorthQueryGraphReadPlanReviewDenialKind::RequiredAsyncMaterialization => {
                "required_async_materialization"
            }
            WorthQueryGraphReadPlanReviewDenialKind::RequiredAccessCapabilityRegistration => {
                "required_access_capability_registration"
            }
            WorthQueryGraphReadPlanReviewDenialKind::RequiredPersistentIndex => {
                "required_persistent_index"
            }
            WorthQueryGraphReadPlanReviewDenialKind::UnsupportedGraphIndexSupport => {
                "unsupported_graph_index_support"
            }
        }
    }
}
