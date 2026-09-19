use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::policy::ConflictMergePolicy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ConflictPolicyId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ConflictPolicyName(String);

impl ConflictPolicyName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ConflictPolicyVersion {
    pub major: u16,
    pub minor: u16,
}

impl ConflictPolicyVersion {
    pub fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictPolicySelectionBasis {
    RequestNamed,
    NodeOverride,
    SchemaDefault,
    BuiltInDefault,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictPolicyDescriptor {
    id: ConflictPolicyId,
    semantic_name: ConflictPolicyName,
    version: ConflictPolicyVersion,
    policy: ConflictMergePolicy,
    digest: String,
}

impl ConflictPolicyDescriptor {
    pub fn new(
        id: ConflictPolicyId,
        semantic_name: ConflictPolicyName,
        version: ConflictPolicyVersion,
        policy: ConflictMergePolicy,
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

    pub fn id(&self) -> ConflictPolicyId {
        self.id
    }

    pub fn semantic_name(&self) -> &ConflictPolicyName {
        &self.semantic_name
    }

    pub fn version(&self) -> ConflictPolicyVersion {
        self.version
    }

    pub fn policy(&self) -> ConflictMergePolicy {
        self.policy
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictPolicyRegistration {
    descriptor: ConflictPolicyDescriptor,
}

impl ConflictPolicyRegistration {
    pub fn new(
        descriptor: ConflictPolicyDescriptor,
    ) -> Result<Self, crate::data::error::SignalError> {
        if descriptor.semantic_name().as_str().trim().is_empty() {
            return Err(crate::data::error::SignalError::invalid_input(
                "conflict policy semantic_name must not be empty",
            ));
        }
        Ok(Self { descriptor })
    }

    pub fn descriptor(&self) -> &ConflictPolicyDescriptor {
        &self.descriptor
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DuplicateConflictPolicyRegistration {
    Id { id: ConflictPolicyId },
    SemanticName { name: ConflictPolicyName },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FrozenConflictPolicyRegistry {
    registrations: Arc<[ConflictPolicyRegistration]>,
    index_by_id: Arc<BTreeMap<ConflictPolicyId, usize>>,
    index_by_name: Arc<BTreeMap<ConflictPolicyName, usize>>,
    registry_digest: String,
}

impl FrozenConflictPolicyRegistry {
    pub fn from_registrations(
        registrations: Vec<ConflictPolicyRegistration>,
    ) -> Result<Self, DuplicateConflictPolicyRegistration> {
        let mut ids = BTreeMap::new();
        let mut names = BTreeMap::new();
        let mut digest_basis = Vec::with_capacity(registrations.len());

        for (index, registration) in registrations.iter().enumerate() {
            let descriptor = registration.descriptor();
            if ids.insert(descriptor.id(), index).is_some() {
                return Err(DuplicateConflictPolicyRegistration::Id {
                    id: descriptor.id(),
                });
            }
            if names
                .insert(descriptor.semantic_name().clone(), index)
                .is_some()
            {
                return Err(DuplicateConflictPolicyRegistration::SemanticName {
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
            ConflictPolicyRegistration::new(ConflictPolicyDescriptor::new(
                ConflictPolicyId(1),
                ConflictPolicyName::new("signal.conflict.reject-shared-state"),
                ConflictPolicyVersion::new(1, 0),
                ConflictMergePolicy::RejectSharedStateConflict,
            ))
            .expect("built-in conflict policy"),
            ConflictPolicyRegistration::new(ConflictPolicyDescriptor::new(
                ConflictPolicyId(2),
                ConflictPolicyName::new("signal.conflict.resolve-source-when-structure-matches"),
                ConflictPolicyVersion::new(1, 0),
                ConflictMergePolicy::ResolveSourceStateWhenStructureMatches,
            ))
            .expect("built-in conflict policy"),
        ])
        .expect("valid built-in conflict policy registry")
    }

    pub fn resolve_by_name(&self, name: &ConflictPolicyName) -> Option<&ConflictPolicyDescriptor> {
        self.index_by_name
            .get(name)
            .and_then(|index| self.registrations.get(*index))
            .map(ConflictPolicyRegistration::descriptor)
    }

    pub fn first_matching_policy(
        &self,
        policy: ConflictMergePolicy,
    ) -> Option<&ConflictPolicyDescriptor> {
        self.registrations
            .iter()
            .find(|registration| registration.descriptor().policy() == policy)
            .map(ConflictPolicyRegistration::descriptor)
    }

    pub fn registry_digest(&self) -> &str {
        &self.registry_digest
    }
}

fn descriptor_digest(
    id: ConflictPolicyId,
    semantic_name: &ConflictPolicyName,
    version: ConflictPolicyVersion,
    policy: ConflictMergePolicy,
) -> String {
    let canonical = serde_json::json!({
        "id": id.0,
        "semantic_name": semantic_name.as_str(),
        "version": { "major": version.major, "minor": version.minor },
        "policy": policy,
    });
    let bytes = serde_json::to_vec(&canonical).expect("conflict policy descriptor serialization");
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn registry_digest(descriptors: &[ConflictPolicyDescriptor]) -> String {
    let mut canonical = descriptors.to_vec();
    canonical.sort_by(|left, right| {
        left.semantic_name()
            .cmp(right.semantic_name())
            .then_with(|| left.version().cmp(&right.version()))
            .then_with(|| left.id().cmp(&right.id()))
    });
    let bytes = serde_json::to_vec(&canonical).expect("conflict policy registry serialization");
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

impl RetainedStorageMeasurement for ConflictPolicyName {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        self.0.retained_heap_charge(work)
    }
}
