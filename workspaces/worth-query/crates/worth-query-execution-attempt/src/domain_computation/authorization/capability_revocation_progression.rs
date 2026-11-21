//! Command/target binding for one exact capability revocation.

use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_installation::facade::WorthQueryOperationFieldTouchScope;
use worth_relational::facade::identity::{EntityId, KindId};

pub struct WorthQueryCapabilityRevocationBinding {
    pub proposal_identity: [u8; 32],
    pub required_field_touch: WorthQueryOperationFieldTouchScope,
    pub target_kind: KindId,
    pub target_entity: String,
    pub resource: EntityId,
    pub resource_relation: KindId,
    pub identity: AspectFieldLocator,
    pub identity_value: AspectValue,
    pub status: AspectFieldLocator,
    pub active: AspectValue,
    pub revoked: AspectValue,
}

impl WorthQueryCapabilityRevocationBinding {
    pub const fn proposal_identity(&self) -> &[u8; 32] {
        &self.proposal_identity
    }
    pub const fn required_field_touch(&self) -> &WorthQueryOperationFieldTouchScope {
        &self.required_field_touch
    }
    pub const fn target_kind(&self) -> KindId {
        self.target_kind
    }
    pub fn target_entity(&self) -> &str {
        &self.target_entity
    }
    pub const fn resource(&self) -> EntityId {
        self.resource
    }
    pub const fn resource_relation(&self) -> KindId {
        self.resource_relation
    }
    pub const fn identity(&self) -> &AspectFieldLocator {
        &self.identity
    }
    pub const fn identity_value(&self) -> &AspectValue {
        &self.identity_value
    }
    pub const fn status(&self) -> &AspectFieldLocator {
        &self.status
    }
    pub const fn active(&self) -> &AspectValue {
        &self.active
    }
    pub const fn revoked(&self) -> &AspectValue {
        &self.revoked
    }
}
