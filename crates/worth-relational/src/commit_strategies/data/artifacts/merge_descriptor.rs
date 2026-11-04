use std::sync::Arc;

use serde::{Deserialize, Serialize};
use worth_foundational::facade::AspectFieldLocator;

mod intent_scope_targets;

use crate::commit_strategies::data::canonical_digest::serial_intent_scope_digest;
use crate::commit_strategies::data::native_strategy_intent_scope::{
    native_strategy_intent_scope_digest, native_strategy_intent_scope_targets,
};
use crate::commit_strategies::data::{
    CommitStrategyDescriptor, CommitStrategyDescriptorDigest, CommitStrategyFamilyName,
    CommitStrategyId, CommitStrategySemanticName, CommitStrategyVersion, LoweredStrategyCommitPlan,
    StrategyIntentName,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyMergeConflictClass {
    IntentReconciliation,
    ReplicaConvergence,
    EntityReplacement,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StrategyIntentScopeDigest([u8; 32]);

impl StrategyIntentScopeDigest {
    pub fn new(digest: [u8; 32]) -> Self {
        Self(digest)
    }

    pub fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyMergeSemantics {
    conflict_class: StrategyMergeConflictClass,
    requires_causal_comparison: bool,
    respects_aspect_merge_policies: bool,
}

impl StrategyMergeSemantics {
    pub fn new(
        conflict_class: StrategyMergeConflictClass,
        requires_causal_comparison: bool,
        respects_aspect_merge_policies: bool,
    ) -> Self {
        Self {
            conflict_class,
            requires_causal_comparison,
            respects_aspect_merge_policies,
        }
    }

    pub fn conflict_class(&self) -> StrategyMergeConflictClass {
        self.conflict_class
    }

    pub fn requires_causal_comparison(&self) -> bool {
        self.requires_causal_comparison
    }

    pub fn respects_aspect_merge_policies(&self) -> bool {
        self.respects_aspect_merge_policies
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StrategyMergeDescriptor {
    pub(super) strategy_id: CommitStrategyId,
    pub(super) descriptor_digest: CommitStrategyDescriptorDigest,
    pub(super) semantic_name: CommitStrategySemanticName,
    pub(super) family_name: CommitStrategyFamilyName,
    pub(super) version: CommitStrategyVersion,
    pub(super) intent_name: StrategyIntentName,
    pub(super) intent_scope_digest: StrategyIntentScopeDigest,
    #[serde(with = "crate::aspect_wire::serde_canonical_aspect_field_locator_arc_slice")]
    pub(super) intent_scope_targets: Arc<[AspectFieldLocator]>,
    pub(super) merge_semantics: StrategyMergeSemantics,
    pub(super) lowering_summary_digest: worth_foundational::CanonicalDigestId,
    // Last: original locators and their nested data precede their cold grant.
    #[serde(skip)]
    pub(super) intent_scope_targets_memory:
        Option<Arc<worth_execution::ExecutionMemoryReservation>>,
}

impl StrategyMergeDescriptor {
    pub fn from_descriptor_and_lowered(
        descriptor: &CommitStrategyDescriptor,
        lowered: &LoweredStrategyCommitPlan,
    ) -> Self {
        Self {
            strategy_id: descriptor.id(),
            descriptor_digest: descriptor.digest(),
            semantic_name: descriptor.semantic_name().clone(),
            family_name: descriptor.family_name().clone(),
            version: descriptor.version(),
            intent_name: descriptor.intent_name().clone(),
            intent_scope_digest: strategy_intent_scope_digest(descriptor, lowered),
            intent_scope_targets: strategy_intent_scope_targets(descriptor, lowered),
            merge_semantics: merge_semantics_for_descriptor(descriptor),
            lowering_summary_digest: worth_foundational::CanonicalDigestId::new(
                crate::commit_strategies::data::canonical_digest::lowering_summary_digest(
                    lowered.lowering_summary(),
                ),
            ),
            intent_scope_targets_memory: None,
        }
    }

    pub fn strategy_id(&self) -> CommitStrategyId {
        self.strategy_id
    }

    pub fn descriptor_digest(&self) -> CommitStrategyDescriptorDigest {
        self.descriptor_digest
    }

    pub fn semantic_name(&self) -> &CommitStrategySemanticName {
        &self.semantic_name
    }

    pub fn family_name(&self) -> &CommitStrategyFamilyName {
        &self.family_name
    }

    pub fn version(&self) -> CommitStrategyVersion {
        self.version
    }

    pub fn intent_name(&self) -> &StrategyIntentName {
        &self.intent_name
    }

    pub fn intent_scope_digest(&self) -> StrategyIntentScopeDigest {
        self.intent_scope_digest
    }

    pub fn intent_scope_targets(&self) -> &[AspectFieldLocator] {
        &self.intent_scope_targets
    }

    pub fn merge_semantics(&self) -> StrategyMergeSemantics {
        self.merge_semantics
    }

    pub fn lowering_summary_digest(&self) -> &[u8; 32] {
        self.lowering_summary_digest.bytes()
    }
}

fn merge_conflict_class_for_descriptor(
    descriptor: &CommitStrategyDescriptor,
) -> StrategyMergeConflictClass {
    match descriptor.family_name().as_str() {
        "strategy.intent" => StrategyMergeConflictClass::IntentReconciliation,
        "strategy.replica" => StrategyMergeConflictClass::ReplicaConvergence,
        "strategy.replace" => StrategyMergeConflictClass::EntityReplacement,
        _ => StrategyMergeConflictClass::Custom,
    }
}

fn merge_semantics_for_descriptor(descriptor: &CommitStrategyDescriptor) -> StrategyMergeSemantics {
    let requires_causal_comparison =
        !matches!(descriptor.family_name().as_str(), "strategy.aspect");
    StrategyMergeSemantics::new(
        merge_conflict_class_for_descriptor(descriptor),
        requires_causal_comparison,
        true,
    )
}

fn strategy_intent_scope_digest(
    descriptor: &CommitStrategyDescriptor,
    lowered: &LoweredStrategyCommitPlan,
) -> StrategyIntentScopeDigest {
    if let Some(digest) = native_strategy_intent_scope_digest(descriptor, lowered) {
        return StrategyIntentScopeDigest::new(digest);
    }
    let mut targets = lowered
        .merged_plan()
        .merged_intents
        .iter()
        .filter_map(|intent| intent.existing_record_target())
        .collect::<Vec<_>>();
    targets.sort();
    targets.dedup();
    StrategyIntentScopeDigest::new(serial_intent_scope_digest(
        lowered.request().strategy_id(),
        lowered.request().canonical_input().schema_name(),
        lowered.request().canonical_input().schema_version(),
        lowered.request().canonical_input().digest(),
        &targets,
    ))
}

fn strategy_intent_scope_targets(
    descriptor: &CommitStrategyDescriptor,
    lowered: &LoweredStrategyCommitPlan,
) -> Arc<[AspectFieldLocator]> {
    native_strategy_intent_scope_targets(descriptor, lowered)
        .unwrap_or_default()
        .into()
}

impl Serialize for StrategyIntentScopeDigest {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        worth_foundational::CanonicalDigestId::new(self.0).serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for StrategyIntentScopeDigest {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        worth_foundational::CanonicalDigestId::deserialize(deserializer)
            .map(|digest| Self(*digest.bytes()))
    }
}
