use std::sync::Arc;

use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::branch::{ProductBranchObservation, ProductBranchReferenceSnapshot};

use crate::identity::{
    CompositePublicationAttemptIdentity, ProductBranchIdentity, ProductBranchIncarnation,
    ProductUnpublishedOwnerEffectsIdentity,
};
use crate::lifecycle::RuntimeWorldInstant;
use crate::publication::{CompositeAttemptProgress, CompositeOwnerExecutionResults};

use super::{ProductUnpublishedLiveObligations, ProductUnpublishedNextAction};

#[path = "product_unpublished/custody.rs"]
mod custody;

#[path = "product_unpublished/handle.rs"]
mod handle;
pub use handle::ProductUnpublishedRecoveryHandle;

#[path = "product_unpublished/record_inputs.rs"]
mod record_inputs;
pub(crate) use record_inputs::RetainedAttemptFacts;

#[path = "product_unpublished/actions.rs"]
mod actions;
pub(crate) use actions::next_actions_for_progress;
pub(crate) use actions::RetainedNextActions;

mod cause;
pub use cause::ProductUnpublishedCause;

mod retention_custody;

pub use retention_custody::ProductUnpublishedRetentionPosture;
mod abandoned;

/// Catalog-owned recovery custody. The public returned value below is only a
/// view over this allocation, so dropping the caller capability cannot drop
/// the component pins, history protection, or deferred owner route.
#[derive(Debug)]
pub(crate) struct ProductUnpublishedOwnerEffectsRecord {
    identity: ProductUnpublishedOwnerEffectsIdentity,
    attempt_identity: CompositePublicationAttemptIdentity,
    expected_head: ProductBranchObservation,
    last_observed_head: Option<ProductBranchReferenceSnapshot>,
    progress: CompositeAttemptProgress,
    successor_basis: Option<AdmittedCompositeRuntimeWorldBasis>,
    component_results: CompositeOwnerExecutionResults,
    retention: crate::publication::ActiveAttemptResources,
    /// The exact product-branch occurrence whose creation charged this
    /// attempt's custody, when the attempt was a creation at all. A publication
    /// moves an existing head and creates no occurrence, so it carries `None`.
    /// The pair is the custody key: the name-keyed identity outlives
    /// retirement, so only identity plus incarnation names the occurrence whose
    /// component branches this record is answerable for.
    destination: Option<(ProductBranchIdentity, ProductBranchIncarnation)>,
    catalog_affinity: usize,
    live_obligations: ProductUnpublishedLiveObligations,
    cause: ProductUnpublishedCause,
    next_actions: RetainedNextActions,
    deadline: Option<RuntimeWorldInstant>,
    admitted_at: RuntimeWorldInstant,
    owner_effect_count: usize,
    metadata_bytes: usize,
}

/// Retained record for owner-local effects that were not represented by a
/// product-reference movement. It is not a commit, rollback token, or replay
/// artifact.
#[must_use = "product-unpublished owner effects must remain retained or be explicitly cleaned up"]
pub struct ProductUnpublishedOwnerEffects {
    record: Arc<ProductUnpublishedOwnerEffectsRecord>,
}

impl std::fmt::Debug for ProductUnpublishedOwnerEffects {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProductUnpublishedOwnerEffects")
            .field("identity", &self.record.identity)
            .field("attempt_identity", &self.record.attempt_identity)
            .field("cause", &self.record.cause)
            .field("next_actions", &self.record.next_actions)
            .field("live_obligations", &self.record.live_obligations)
            .field("owner_effect_count", &self.record.owner_effect_count)
            .field("metadata_bytes", &self.record.metadata_bytes)
            .finish_non_exhaustive()
    }
}

impl ProductUnpublishedOwnerEffects {
    /// Exact reservation charge for the inline retained-record representation.
    /// Actions are stored in a fixed bounded array, so no allocator capacity or
    /// lower-bound vector hint can undercharge an installed record.
    pub(crate) const fn metadata_charge_hint() -> usize {
        let retained = std::mem::size_of::<ProductUnpublishedOwnerEffectsRecord>();
        let active = crate::publication::ActiveAttemptRecord::metadata_charge_hint();
        (if active > retained { active } else { retained })
            + super::catalog::ProductUnpublishedRecoveryCatalog::slot_metadata_charge_hint()
    }

    pub(crate) fn from_catalog_record(record: Arc<ProductUnpublishedOwnerEffectsRecord>) -> Self {
        Self { record }
    }

    pub fn identity(&self) -> &ProductUnpublishedOwnerEffectsIdentity {
        &self.record.identity
    }

    pub fn attempt_identity(&self) -> &CompositePublicationAttemptIdentity {
        &self.record.attempt_identity
    }

    pub fn expected_head(&self) -> &ProductBranchObservation {
        &self.record.expected_head
    }

    pub fn last_observed_head(&self) -> Option<&ProductBranchReferenceSnapshot> {
        self.record.last_observed_head.as_ref()
    }

    pub fn progress(&self) -> &CompositeAttemptProgress {
        &self.record.progress
    }

    pub fn successor_basis(&self) -> Option<&AdmittedCompositeRuntimeWorldBasis> {
        self.record.successor_basis.as_ref()
    }

    pub fn component_results(&self) -> &CompositeOwnerExecutionResults {
        &self.record.component_results
    }

    /// Exact installed successor occurrence retained by this recovery record,
    /// when the attempt installed one at all. `None` is a record whose attempt
    /// lost before the product reference moved: it released its history slot
    /// and names its successor by basis alone. The identity is evidence only;
    /// it grants no History or publication authority.
    pub fn successor_commit(&self) -> Option<&crate::identity::CompositeCommitIdentity> {
        self.record.retention.successor_commit()
    }

    /// The product-branch occurrence this record's owner effects were created
    /// for, when the attempt created one. It names the custody a cleanup must
    /// drain; it is not authority to install or retire that branch.
    pub fn destination_branch(&self) -> Option<(&ProductBranchIdentity, ProductBranchIncarnation)> {
        self.record.destination()
    }

    pub fn retention_posture(&self) -> ProductUnpublishedRetentionPosture {
        self.record.retention.retention_posture()
    }

    /// Every obligation this record still holds live, counted from its own
    /// custody when it was installed.
    pub fn live_obligation_count(&self) -> usize {
        self.record.live_obligations.total()
    }

    /// The same count divided by scope, for a report that must never
    /// contradict the record it describes.
    pub(crate) fn live_obligations(&self) -> ProductUnpublishedLiveObligations {
        self.record.live_obligations
    }

    pub fn cause(&self) -> ProductUnpublishedCause {
        self.record.cause
    }

    pub fn next_actions(&self) -> &[ProductUnpublishedNextAction] {
        self.record.next_actions.as_slice()
    }

    pub fn deadline(&self) -> Option<RuntimeWorldInstant> {
        self.record.deadline
    }

    /// Owner-clock instant at which this attempt completed admission.
    pub fn admitted_at(&self) -> RuntimeWorldInstant {
        self.record.admitted_at
    }

    /// Costs carried from an attempted final cell comparison. Earlier partials
    /// have no comparison snapshot, rather than invented zero-cost evidence.
    pub fn product_comparison_costs(
        &self,
    ) -> Option<crate::publication::CompositePublicationCostCounters> {
        self.record.retention.product_comparison_costs
    }

    pub fn owner_effect_count(&self) -> usize {
        self.record.owner_effect_count
    }

    pub fn metadata_bytes(&self) -> usize {
        self.record.metadata_bytes
    }

    pub fn recovery_handle(&self) -> ProductUnpublishedRecoveryHandle {
        ProductUnpublishedRecoveryHandle::new(
            self.record.identity.clone(),
            self.record.catalog_affinity,
        )
    }
}

impl ProductUnpublishedOwnerEffectsRecord {
    pub(crate) fn admitted_at(&self) -> RuntimeWorldInstant {
        self.admitted_at
    }
    pub(crate) fn deadline(&self) -> Option<RuntimeWorldInstant> {
        self.deadline
    }
}
