use crate::branch::{ProductBranchObservation, ProductBranchReferenceSnapshot};
use crate::identity::{
    CompositePublicationAttemptIdentity, ProductBranchIdentity, ProductBranchIncarnation,
    ProductUnpublishedOwnerEffectsIdentity,
};
use crate::publication::{CompositeAttemptProgress, CompositeOwnerExecutionResults};

/// What a retained record knows about the attempt that produced it, before any
/// posture is chosen. Both posture constructors take it whole: an identity
/// without its attempt, or a progress without the owner results it was read
/// from, would describe an attempt that never ran, so no caller may name a
/// subset of it.
pub(crate) struct RetainedAttemptFacts {
    pub(crate) admitted_at: crate::lifecycle::RuntimeWorldInstant,
    pub(crate) identity: ProductUnpublishedOwnerEffectsIdentity,
    pub(crate) attempt_identity: CompositePublicationAttemptIdentity,
    pub(crate) expected_head: ProductBranchObservation,
    pub(crate) last_observed_head: Option<ProductBranchReferenceSnapshot>,
    pub(crate) progress: CompositeAttemptProgress,
    pub(crate) owner_results: CompositeOwnerExecutionResults,
    /// The product-branch occurrence whose creation charged this attempt's
    /// custody, when the attempt created one at all.
    pub(crate) destination: Option<(ProductBranchIdentity, ProductBranchIncarnation)>,
}
