//! Continue one actual unpublished completion without synthesizing a Performed carrier.
use crate::domain_computation::primary_graph::provider::WorthQueryCanonicalInboundCompletion;
use worth_runtime_world::facade::{
    CompositeCommitIdentity, ProductUnpublishedRecoveryHandle, WorldRecoveredUnpublishedEffects,
};

use crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanup;

#[derive(Clone)]
pub enum OriginalCompletionState {
    Unpublished,
    Releasing,
    Cleanup(WorthQueryProductBranchOwnerCleanup),
    Released,
    Published(CompositeCommitIdentity),
}

pub enum RecordedCompletionProgress {
    Completed(WorthQueryCanonicalInboundCompletion),
    StaleReleased,
}

pub use super::super::recorded_completion_denial::WorthQueryRecordedCompletionDenial;

#[derive(Clone)]
pub struct RecordedCompletionContinuation {
    pub handle: ProductUnpublishedRecoveryHandle,
    pub subject: worth_relational::facade::durability::RelationalAssociatedEffectSubject,
    pub entity: worth_relational::facade::identity::EntityId,
    pub state: OriginalCompletionState,
    pub last_denial: Option<std::sync::Arc<WorthQueryRecordedCompletionDenial>>,
}

impl RecordedCompletionContinuation {
    pub fn same_original(&self, other: &Self) -> bool {
        self.handle == other.handle
            && self.subject.receipt() == other.subject.receipt()
            && self.subject.position() == other.subject.position()
            && self.subject.artifact_digest() == other.subject.artifact_digest()
            && self.entity == other.entity
    }
    pub fn from_original(
        effects: &WorldRecoveredUnpublishedEffects,
        entity: worth_relational::facade::identity::EntityId,
    ) -> Option<Self> {
        Some(Self {
            handle: effects.recovery_handle(),
            subject: effects.native_effect()?.subject().clone(),
            entity,
            state: OriginalCompletionState::Unpublished,
            last_denial: None,
        })
    }

    pub fn last_denial(&self) -> Option<std::sync::Arc<WorthQueryRecordedCompletionDenial>> {
        self.last_denial.clone()
    }
}

pub mod denial;
