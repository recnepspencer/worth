use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::policy::DeletionMergePolicy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DeletionPolicyId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DeletionPolicyName(String);

impl DeletionPolicyName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DeletionPolicyVersion {
    pub major: u16,
    pub minor: u16,
}

impl DeletionPolicyVersion {
    pub fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeletionPolicySelectionBasis {
    RequestNamed,
    NodeOverride,
    SchemaDefault,
    BuiltInDefault,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeletionPolicyDescriptor {
    id: DeletionPolicyId,
    semantic_name: DeletionPolicyName,
    version: DeletionPolicyVersion,
    policy: DeletionMergePolicy,
    digest: String,
}

impl DeletionPolicyDescriptor {
    pub fn new(
        id: DeletionPolicyId,
        semantic_name: DeletionPolicyName,
        version: DeletionPolicyVersion,
        policy: DeletionMergePolicy,
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

    pub fn id(&self) -> DeletionPolicyId {
        self.id
    }

    pub fn semantic_name(&self) -> &DeletionPolicyName {
        &self.semantic_name
    }

    pub fn version(&self) -> DeletionPolicyVersion {
        self.version
    }

    pub fn policy(&self) -> DeletionMergePolicy {
        self.policy
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeletionPolicyRegistration {
    descriptor: DeletionPolicyDescriptor,
}

impl DeletionPolicyRegistration {
    pub fn new(
        descriptor: DeletionPolicyDescriptor,
    ) -> Result<Self, crate::data::error::SignalError> {
        if descriptor.semantic_name().as_str().trim().is_empty() {
            return Err(crate::data::error::SignalError::invalid_input(
                "deletion policy semantic_name must not be empty",
            ));
        }
        Ok(Self { descriptor })
    }

    pub fn descriptor(&self) -> &DeletionPolicyDescriptor {
        &self.descriptor
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DuplicateDeletionPolicyRegistration {
    Id { id: DeletionPolicyId },
    SemanticName { name: DeletionPolicyName },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FrozenDeletionPolicyRegistry {
    registrations: Arc<[DeletionPolicyRegistration]>,
    index_by_id: Arc<BTreeMap<DeletionPolicyId, usize>>,
    index_by_name: Arc<BTreeMap<DeletionPolicyName, usize>>,
    registry_digest: String,
}

impl FrozenDeletionPolicyRegistry {
    pub fn from_registrations(
        registrations: Vec<DeletionPolicyRegistration>,
    ) -> Result<Self, DuplicateDeletionPolicyRegistration> {
        let mut ids = BTreeMap::new();
        let mut names = BTreeMap::new();
        let mut digest_basis = Vec::with_capacity(registrations.len());
        for (index, registration) in registrations.iter().enumerate() {
            let descriptor = registration.descriptor();
            if ids.insert(descriptor.id(), index).is_some() {
                return Err(DuplicateDeletionPolicyRegistration::Id {
                    id: descriptor.id(),
                });
            }
            if names
                .insert(descriptor.semantic_name().clone(), index)
                .is_some()
            {
                return Err(DuplicateDeletionPolicyRegistration::SemanticName {
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
            DeletionPolicyRegistration::new(DeletionPolicyDescriptor::new(
                DeletionPolicyId(1),
                DeletionPolicyName::new("signal.deletion.preserve-target-only"),
                DeletionPolicyVersion::new(1, 0),
                DeletionMergePolicy::PreserveTargetOnly,
            ))
            .expect("built-in deletion policy"),
            DeletionPolicyRegistration::new(DeletionPolicyDescriptor::new(
                DeletionPolicyId(2),
                DeletionPolicyName::new("signal.deletion.reject-target-only-conflict"),
                DeletionPolicyVersion::new(1, 0),
                DeletionMergePolicy::RejectTargetOnlyConflict,
            ))
            .expect("built-in deletion policy"),
        ])
        .expect("valid built-in deletion policy registry")
    }

    pub fn resolve_by_name(&self, name: &DeletionPolicyName) -> Option<&DeletionPolicyDescriptor> {
        self.index_by_name
            .get(name)
            .and_then(|index| self.registrations.get(*index))
            .map(DeletionPolicyRegistration::descriptor)
    }

    pub fn first_matching_policy(
        &self,
        policy: DeletionMergePolicy,
    ) -> Option<&DeletionPolicyDescriptor> {
        self.registrations
            .iter()
            .find(|registration| registration.descriptor().policy() == policy)
            .map(DeletionPolicyRegistration::descriptor)
    }

    pub fn registry_digest(&self) -> &str {
        &self.registry_digest
    }
}

fn descriptor_digest(
    id: DeletionPolicyId,
    semantic_name: &DeletionPolicyName,
    version: DeletionPolicyVersion,
    policy: DeletionMergePolicy,
) -> String {
    let canonical = serde_json::json!({
        "id": id.0,
        "semantic_name": semantic_name.as_str(),
        "version": { "major": version.major, "minor": version.minor },
        "policy": policy,
    });
    let bytes = serde_json::to_vec(&canonical).expect("deletion policy descriptor serialization");
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn registry_digest(descriptors: &[DeletionPolicyDescriptor]) -> String {
    let mut canonical = descriptors.to_vec();
    canonical.sort_by(|left, right| {
        left.semantic_name()
            .cmp(right.semantic_name())
            .then_with(|| left.version().cmp(&right.version()))
            .then_with(|| left.id().cmp(&right.id()))
    });
    let bytes = serde_json::to_vec(&canonical).expect("deletion policy registry serialization");
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

impl RetainedStorageMeasurement for DeletionPolicyName {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        self.0.retained_heap_charge(work)
    }
}
