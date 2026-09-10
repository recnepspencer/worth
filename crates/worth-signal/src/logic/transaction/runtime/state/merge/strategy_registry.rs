use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::core::BranchMergeStrategy;
use super::policy::BranchMergeReconciliationPolicy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MergeStrategyId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MergeStrategyName(String);

impl MergeStrategyName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MergeStrategyVersion {
    pub major: u16,
    pub minor: u16,
}

impl MergeStrategyVersion {
    pub fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeStrategyDescriptor {
    id: MergeStrategyId,
    semantic_name: MergeStrategyName,
    version: MergeStrategyVersion,
    merge_strategy: BranchMergeStrategy,
    reconciliation_policy: BranchMergeReconciliationPolicy,
    digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeStrategySelectionBasis {
    RequestNamed,
    RequestHint,
    NodeOverride,
    SchemaDefault,
    DivergenceDefault,
}

impl MergeStrategyDescriptor {
    pub fn new(
        id: MergeStrategyId,
        semantic_name: MergeStrategyName,
        version: MergeStrategyVersion,
        merge_strategy: BranchMergeStrategy,
        reconciliation_policy: BranchMergeReconciliationPolicy,
    ) -> Self {
        let digest = merge_strategy_digest(
            id,
            &semantic_name,
            version,
            merge_strategy,
            &reconciliation_policy,
        );
        Self {
            id,
            semantic_name,
            version,
            merge_strategy,
            reconciliation_policy,
            digest,
        }
    }

    pub fn id(&self) -> MergeStrategyId {
        self.id
    }

    pub fn semantic_name(&self) -> &MergeStrategyName {
        &self.semantic_name
    }

    pub fn version(&self) -> MergeStrategyVersion {
        self.version
    }

    pub fn merge_strategy(&self) -> BranchMergeStrategy {
        self.merge_strategy
    }

    pub fn reconciliation_policy(&self) -> &BranchMergeReconciliationPolicy {
        &self.reconciliation_policy
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeStrategyRegistration {
    descriptor: MergeStrategyDescriptor,
}

impl MergeStrategyRegistration {
    pub fn new(
        descriptor: MergeStrategyDescriptor,
    ) -> Result<Self, crate::data::error::SignalError> {
        if descriptor.semantic_name().as_str().trim().is_empty() {
            return Err(crate::data::error::SignalError::invalid_input(
                "merge strategy semantic_name must not be empty",
            ));
        }
        Ok(Self { descriptor })
    }

    pub fn descriptor(&self) -> &MergeStrategyDescriptor {
        &self.descriptor
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DuplicateMergeStrategyRegistration {
    Id { id: MergeStrategyId },
    SemanticName { name: MergeStrategyName },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FrozenMergeStrategyRegistry {
    registrations: Arc<[MergeStrategyRegistration]>,
    index_by_id: Arc<BTreeMap<MergeStrategyId, usize>>,
    index_by_name: Arc<BTreeMap<MergeStrategyName, usize>>,
    registry_digest: String,
}

impl FrozenMergeStrategyRegistry {
    pub fn from_registrations(
        registrations: Vec<MergeStrategyRegistration>,
    ) -> Result<Self, DuplicateMergeStrategyRegistration> {
        let mut ids = BTreeMap::new();
        let mut names = BTreeMap::new();
        let mut digest_basis = Vec::with_capacity(registrations.len());

        for (index, registration) in registrations.iter().enumerate() {
            let descriptor = registration.descriptor();
            if ids.insert(descriptor.id(), index).is_some() {
                return Err(DuplicateMergeStrategyRegistration::Id {
                    id: descriptor.id(),
                });
            }
            if names
                .insert(descriptor.semantic_name().clone(), index)
                .is_some()
            {
                return Err(DuplicateMergeStrategyRegistration::SemanticName {
                    name: descriptor.semantic_name().clone(),
                });
            }
            digest_basis.push(descriptor.clone());
        }

        Ok(Self {
            registrations: registrations.into(),
            index_by_id: Arc::new(ids),
            index_by_name: Arc::new(names),
            registry_digest: registry_digest(&digest_basis),
        })
    }

    pub fn built_in() -> Self {
        Self::from_registrations(vec![
            MergeStrategyRegistration::new(MergeStrategyDescriptor::new(
                MergeStrategyId(1),
                MergeStrategyName::new("signal.merge.adopt-source-head"),
                MergeStrategyVersion::new(1, 0),
                BranchMergeStrategy::AdoptSourceHead,
                BranchMergeReconciliationPolicy::built_in_default(),
            ))
            .expect("built-in merge strategy registration"),
            MergeStrategyRegistration::new(MergeStrategyDescriptor::new(
                MergeStrategyId(2),
                MergeStrategyName::new("signal.merge.adopt-source-subset"),
                MergeStrategyVersion::new(1, 0),
                BranchMergeStrategy::AdoptSourceSubset,
                BranchMergeReconciliationPolicy::built_in_default(),
            ))
            .expect("built-in merge strategy registration"),
            MergeStrategyRegistration::new(MergeStrategyDescriptor::new(
                MergeStrategyId(3),
                MergeStrategyName::new("signal.merge.replay-source-delta"),
                MergeStrategyVersion::new(1, 0),
                BranchMergeStrategy::ReplaySourceDeltaOntoTarget,
                BranchMergeReconciliationPolicy::built_in_default(),
            ))
            .expect("built-in merge strategy registration"),
            MergeStrategyRegistration::new(MergeStrategyDescriptor::new(
                MergeStrategyId(4),
                MergeStrategyName::new("signal.merge.rebase-source-onto-target"),
                MergeStrategyVersion::new(1, 0),
                BranchMergeStrategy::RebaseSourceOntoTarget,
                BranchMergeReconciliationPolicy::built_in_default(),
            ))
            .expect("built-in merge strategy registration"),
        ])
        .expect("valid built-in merge strategy registry")
    }

    pub fn resolve_by_name(&self, name: &MergeStrategyName) -> Option<&MergeStrategyDescriptor> {
        self.index_by_name
            .get(name)
            .and_then(|index| self.registrations.get(*index))
            .map(MergeStrategyRegistration::descriptor)
    }

    pub fn first_matching_strategy(
        &self,
        merge_strategy: BranchMergeStrategy,
    ) -> Option<&MergeStrategyDescriptor> {
        self.registrations
            .iter()
            .find(|registration| registration.descriptor().merge_strategy() == merge_strategy)
            .map(MergeStrategyRegistration::descriptor)
    }

    pub fn registry_digest(&self) -> &str {
        &self.registry_digest
    }
}

fn registry_digest(descriptors: &[MergeStrategyDescriptor]) -> String {
    let mut canonical = descriptors.to_vec();
    canonical.sort_by(|left, right| {
        left.semantic_name()
            .cmp(right.semantic_name())
            .then_with(|| left.version().cmp(&right.version()))
            .then_with(|| left.id().cmp(&right.id()))
    });
    let bytes = serde_json::to_vec(&canonical).expect("merge strategy registry serialization");
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn merge_strategy_digest(
    id: MergeStrategyId,
    semantic_name: &MergeStrategyName,
    version: MergeStrategyVersion,
    merge_strategy: BranchMergeStrategy,
    reconciliation_policy: &BranchMergeReconciliationPolicy,
) -> String {
    let canonical = serde_json::json!({
        "id": id.0,
        "semantic_name": semantic_name.as_str(),
        "version": { "major": version.major, "minor": version.minor },
        "merge_strategy": merge_strategy,
        "reconciliation_policy": reconciliation_policy,
    });
    let bytes = serde_json::to_vec(&canonical).expect("merge strategy descriptor serialization");
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

impl RetainedStorageMeasurement for MergeStrategyName {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        self.0.retained_heap_charge(work)
    }
}
