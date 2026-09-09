use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MergeBaseStrategyId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MergeBaseStrategyName(String);

impl MergeBaseStrategyName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MergeBaseStrategyVersion {
    pub major: u16,
    pub minor: u16,
}

impl MergeBaseStrategyVersion {
    pub fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeBaseSelectionBasis {
    RequestNamed,
    BuiltInDefault,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeBaseSelectionPolicy {
    ForkPointSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeBaseStrategyDescriptor {
    id: MergeBaseStrategyId,
    semantic_name: MergeBaseStrategyName,
    version: MergeBaseStrategyVersion,
    policy: MergeBaseSelectionPolicy,
    digest: String,
}

impl MergeBaseStrategyDescriptor {
    pub fn new(
        id: MergeBaseStrategyId,
        semantic_name: MergeBaseStrategyName,
        version: MergeBaseStrategyVersion,
        policy: MergeBaseSelectionPolicy,
    ) -> Self {
        let digest = descriptor_digest(id, &semantic_name, version, policy);
        Self {
            id,
            semantic_name,
            version,
            policy,
            digest,
        }
    }

    pub fn semantic_name(&self) -> &MergeBaseStrategyName {
        &self.semantic_name
    }

    pub fn version(&self) -> MergeBaseStrategyVersion {
        self.version
    }

    pub fn policy(&self) -> MergeBaseSelectionPolicy {
        self.policy
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeBaseStrategyRegistration {
    descriptor: MergeBaseStrategyDescriptor,
}

impl MergeBaseStrategyRegistration {
    pub fn new(
        descriptor: MergeBaseStrategyDescriptor,
    ) -> Result<Self, crate::data::error::SignalError> {
        if descriptor.semantic_name().as_str().trim().is_empty() {
            return Err(crate::data::error::SignalError::invalid_input(
                "merge-base strategy semantic_name must not be empty",
            ));
        }
        Ok(Self { descriptor })
    }

    pub fn descriptor(&self) -> &MergeBaseStrategyDescriptor {
        &self.descriptor
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DuplicateMergeBaseStrategyRegistration {
    Id { id: MergeBaseStrategyId },
    SemanticName { name: MergeBaseStrategyName },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FrozenMergeBaseStrategyRegistry {
    registrations: Arc<[MergeBaseStrategyRegistration]>,
    index_by_name: Arc<BTreeMap<MergeBaseStrategyName, usize>>,
    registry_digest: String,
}

impl FrozenMergeBaseStrategyRegistry {
    pub fn from_registrations(
        registrations: Vec<MergeBaseStrategyRegistration>,
    ) -> Result<Self, DuplicateMergeBaseStrategyRegistration> {
        let mut ids = BTreeMap::new();
        let mut names = BTreeMap::new();
        let mut digest_basis = Vec::with_capacity(registrations.len());
        for (index, registration) in registrations.iter().enumerate() {
            let descriptor = registration.descriptor();
            if ids.insert(descriptor.id, index).is_some() {
                return Err(DuplicateMergeBaseStrategyRegistration::Id { id: descriptor.id });
            }
            if names
                .insert(descriptor.semantic_name().clone(), index)
                .is_some()
            {
                return Err(DuplicateMergeBaseStrategyRegistration::SemanticName {
                    name: descriptor.semantic_name().clone(),
                });
            }
            digest_basis.push(descriptor.clone());
        }
        Ok(Self {
            registrations: registrations.into(),
            index_by_name: Arc::new(names),
            registry_digest: registry_digest(&digest_basis),
        })
    }

    pub fn built_in() -> Self {
        Self::from_registrations(vec![MergeBaseStrategyRegistration::new(
            MergeBaseStrategyDescriptor::new(
                MergeBaseStrategyId(1),
                MergeBaseStrategyName::new("signal.merge-base.fork-point"),
                MergeBaseStrategyVersion::new(1, 0),
                MergeBaseSelectionPolicy::ForkPointSnapshot,
            ),
        )
        .expect("built-in merge-base strategy")])
        .expect("valid built-in merge-base strategy registry")
    }

    pub fn resolve_by_name(
        &self,
        name: &MergeBaseStrategyName,
    ) -> Option<&MergeBaseStrategyDescriptor> {
        self.index_by_name
            .get(name)
            .and_then(|index| self.registrations.get(*index))
            .map(MergeBaseStrategyRegistration::descriptor)
    }

    pub fn first_matching_policy(
        &self,
        policy: MergeBaseSelectionPolicy,
    ) -> Option<&MergeBaseStrategyDescriptor> {
        self.registrations
            .iter()
            .find(|registration| registration.descriptor().policy() == policy)
            .map(MergeBaseStrategyRegistration::descriptor)
    }

    pub fn registry_digest(&self) -> &str {
        &self.registry_digest
    }
}

fn descriptor_digest(
    id: MergeBaseStrategyId,
    semantic_name: &MergeBaseStrategyName,
    version: MergeBaseStrategyVersion,
    policy: MergeBaseSelectionPolicy,
) -> String {
    let canonical = serde_json::json!({
        "id": id.0,
        "semantic_name": semantic_name.as_str(),
        "version": { "major": version.major, "minor": version.minor },
        "policy": policy,
    });
    let bytes =
        serde_json::to_vec(&canonical).expect("merge-base strategy descriptor serialization");
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn registry_digest(descriptors: &[MergeBaseStrategyDescriptor]) -> String {
    let mut canonical = descriptors.to_vec();
    canonical.sort_by(|left, right| {
        left.semantic_name
            .cmp(&right.semantic_name)
            .then(left.version.major.cmp(&right.version.major))
            .then(left.version.minor.cmp(&right.version.minor))
    });
    let bytes = serde_json::to_vec(&canonical).expect("merge-base strategy registry serialization");
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for MergeBaseStrategyName {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self(name) = self;
        name.retained_heap_charge(work)
    }
}
