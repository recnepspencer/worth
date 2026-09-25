use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::execution::{InvariantExecutionPoint, InvariantFailureEffect};
use super::groups::{InvariantCostClass, InvariantGroupSet};
use super::rule_id::{CustomInvariantSemanticIdentity, InvariantRuleId};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CustomInvariantAccessContract {
    pub read_entity_kinds: Vec<crate::identity::data::KindId>,
    pub read_relation_kinds: Vec<crate::identity::data::KindId>,
    pub affected_entity_kinds: Vec<crate::identity::data::KindId>,
    pub affected_relation_kinds: Vec<crate::identity::data::KindId>,
    /// Whether a relation mutation can admit this rule through its entity
    /// endpoints even when the relation kind itself is outside its access.
    /// Older registrations retain the enabled behavior.
    #[serde(
        default = "endpoint_touches_enabled",
        skip_serializing_if = "endpoint_touches_enabled_ref"
    )]
    pub include_relation_endpoint_entity_touches: bool,
}

fn endpoint_touches_enabled() -> bool {
    true
}

fn endpoint_touches_enabled_ref(enabled: &bool) -> bool {
    *enabled
}

impl Default for CustomInvariantAccessContract {
    fn default() -> Self {
        Self {
            read_entity_kinds: Vec::new(),
            read_relation_kinds: Vec::new(),
            affected_entity_kinds: Vec::new(),
            affected_relation_kinds: Vec::new(),
            include_relation_endpoint_entity_touches: true,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::CustomInvariantAccessContract;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct LegacyAccessContract {
        read_entity_kinds: Vec<crate::identity::data::KindId>,
        read_relation_kinds: Vec<crate::identity::data::KindId>,
        affected_entity_kinds: Vec<crate::identity::data::KindId>,
        affected_relation_kinds: Vec<crate::identity::data::KindId>,
    }

    #[test]
    fn endpoint_applicability_preserves_legacy_default_and_encodes_opt_out() {
        let legacy = LegacyAccessContract {
            read_entity_kinds: Vec::new(),
            read_relation_kinds: Vec::new(),
            affected_entity_kinds: Vec::new(),
            affected_relation_kinds: Vec::new(),
        };
        let prior = rmp_serde::to_vec_named(&legacy).expect("encode legacy access contract");
        let restored: CustomInvariantAccessContract =
            rmp_serde::from_slice(&prior).expect("read legacy access contract");
        assert!(restored.include_relation_endpoint_entity_touches);
        assert_eq!(
            rmp_serde::to_vec_named(&restored).expect("encode default access contract"),
            prior
        );
        let prior_sequence = rmp_serde::to_vec(&legacy).expect("encode legacy sequence");
        let restored_sequence: CustomInvariantAccessContract =
            rmp_serde::from_slice(&prior_sequence).expect("read legacy sequence");
        assert!(restored_sequence.include_relation_endpoint_entity_touches);
        assert_eq!(
            rmp_serde::to_vec(&restored_sequence).expect("encode default sequence"),
            prior_sequence
        );
        let mut opt_out = restored;
        opt_out.include_relation_endpoint_entity_touches = false;
        let bytes = rmp_serde::to_vec_named(&opt_out).expect("encode opt-out access contract");
        let restored: CustomInvariantAccessContract =
            rmp_serde::from_slice(&bytes).expect("read opt-out access contract");
        assert!(!restored.include_relation_endpoint_entity_touches);
    }
}
