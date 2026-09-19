use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::execution::{InvariantExecutionPoint, InvariantFailureEffect};
use super::groups::{InvariantCostClass, InvariantGroupSet};
use super::rule_id::{CustomInvariantSemanticIdentity, InvariantRuleId};

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CustomInvariantAccessContract {
    pub read_entity_kinds: Vec<crate::identity::data::KindId>,
    pub read_relation_kinds: Vec<crate::identity::data::KindId>,
    pub affected_entity_kinds: Vec<crate::identity::data::KindId>,
    pub affected_relation_kinds: Vec<crate::identity::data::KindId>,
}

impl CustomInvariantAccessContract {
    pub fn canonicalize(mut self) -> Self {
        self.read_entity_kinds.sort();
        self.read_entity_kinds.dedup();
        self.read_relation_kinds.sort();
        self.read_relation_kinds.dedup();
        self.affected_entity_kinds.sort();
        self.affected_entity_kinds.dedup();
        self.affected_relation_kinds.sort();
        self.affected_relation_kinds.dedup();
        self
    }
    pub(crate) fn reads_entity(&self, kind: crate::identity::data::KindId) -> bool {
        self.read_entity_kinds.binary_search(&kind).is_ok()
    }
    pub(crate) fn reads_relation(&self, kind: crate::identity::data::KindId) -> bool {
        self.read_relation_kinds.binary_search(&kind).is_ok()
    }
    pub(crate) fn affects_entity(&self, kind: crate::identity::data::KindId) -> bool {
        self.affected_entity_kinds.binary_search(&kind).is_ok()
    }
    pub(crate) fn affects_relation(&self, kind: crate::identity::data::KindId) -> bool {
        self.affected_relation_kinds.binary_search(&kind).is_ok()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvariantSemanticsClass {
    NativeAlwaysOn,
    NativeSchemaLowered,
    CustomStructural,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupportedExecutionPoints {
    mask: u8,
}

impl SupportedExecutionPoints {
    pub const fn empty() -> Self {
        Self { mask: 0 }
    }

    pub const fn only(point: InvariantExecutionPoint) -> Self {
        Self {
            mask: 1 << (point as u8),
        }
    }

    pub const fn union(self, other: Self) -> Self {
        Self {
            mask: self.mask | other.mask,
        }
    }

    pub const fn supports(self, point: InvariantExecutionPoint) -> bool {
        (self.mask & (1 << (point as u8))) != 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomInvariantOperationalMetadata {
    pub maximum_work_units: std::num::NonZeroU64,
    pub execution_point: InvariantExecutionPoint,
    pub groups: InvariantGroupSet,
    pub cost_class: InvariantCostClass,
    pub failure_effect: InvariantFailureEffect,
    pub access: CustomInvariantAccessContract,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvariantRuleDescriptor {
    pub id: InvariantRuleId,
    pub execution_points: SupportedExecutionPoints,
    pub groups: InvariantGroupSet,
    pub cost_class: InvariantCostClass,
    pub failure_effect: InvariantFailureEffect,
    pub semantics: InvariantSemanticsClass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomInvariantDescriptor {
    pub identity: CustomInvariantSemanticIdentity,
    pub display_name: Arc<str>,
    pub operational: CustomInvariantOperationalMetadata,
}
