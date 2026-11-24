use serde::{Deserialize, Serialize};

use crate::identity::data::{EntityId, KindId, RelationId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RelationJoinSharedEndpoint {
    Source,
    Target,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RelationJoinLeg {
    relation_kind: KindId,
    shared_endpoint: RelationJoinSharedEndpoint,
    external_entity_kind: KindId,
}

impl RelationJoinLeg {
    pub const fn new(
        relation_kind: KindId,
        shared_endpoint: RelationJoinSharedEndpoint,
        external_entity_kind: KindId,
    ) -> Self {
        Self {
            relation_kind,
            shared_endpoint,
            external_entity_kind,
        }
    }

    pub const fn relation_kind(self) -> KindId {
        self.relation_kind
    }

    pub const fn shared_endpoint(self) -> RelationJoinSharedEndpoint {
        self.shared_endpoint
    }

    pub const fn external_entity_kind(self) -> KindId {
        self.external_entity_kind
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RelationJoinDefinition {
    left: RelationJoinLeg,
    right: RelationJoinLeg,
    shared_entity_kind: KindId,
}

impl RelationJoinDefinition {
    pub const fn new(
        left: RelationJoinLeg,
        right: RelationJoinLeg,
        shared_entity_kind: KindId,
    ) -> Self {
        Self {
            left,
            right,
            shared_entity_kind,
        }
    }

    pub const fn left(self) -> RelationJoinLeg {
        self.left
    }

    pub const fn right(self) -> RelationJoinLeg {
        self.right
    }

    pub const fn shared_entity_kind(self) -> KindId {
        self.shared_entity_kind
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RelationJoinKey {
    left_entity_id: EntityId,
    right_entity_id: EntityId,
}

impl RelationJoinKey {
    pub const fn new(left_entity_id: EntityId, right_entity_id: EntityId) -> Self {
        Self {
            left_entity_id,
            right_entity_id,
        }
    }

    pub const fn left_entity_id(self) -> EntityId {
        self.left_entity_id
    }

    pub const fn right_entity_id(self) -> EntityId {
        self.right_entity_id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RelationJoinEntry {
    shared_entity_id: EntityId,
    left_relation_id: RelationId,
    right_relation_id: RelationId,
}

impl RelationJoinEntry {
    pub const fn new(
        shared_entity_id: EntityId,
        left_relation_id: RelationId,
        right_relation_id: RelationId,
    ) -> Self {
        Self {
            shared_entity_id,
            left_relation_id,
            right_relation_id,
        }
    }

    pub const fn shared_entity_id(self) -> EntityId {
        self.shared_entity_id
    }

    pub const fn left_relation_id(self) -> RelationId {
        self.left_relation_id
    }

    pub const fn right_relation_id(self) -> RelationId {
        self.right_relation_id
    }
}
