use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct IdentityMatcherId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct IdentityMatcherName(String);

impl IdentityMatcherName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct IdentityMatcherVersion {
    pub major: u16,
    pub minor: u16,
}

impl IdentityMatcherVersion {
    pub fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentityMatcherSelectionBasis {
    RequestNamed,
    NodeOverride,
    SchemaDefault,
    BuiltInDefault,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentityMatchPolicy {
    ExactNodeId,
    OutputIdentityWithinTargetJournal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityMatcherDescriptor {
    id: IdentityMatcherId,
    semantic_name: IdentityMatcherName,
    version: IdentityMatcherVersion,
    policy: IdentityMatchPolicy,
    digest: String,
}

impl IdentityMatcherDescriptor {
    pub fn new(
        id: IdentityMatcherId,
        semantic_name: IdentityMatcherName,
        version: IdentityMatcherVersion,
        policy: IdentityMatchPolicy,
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

    pub fn id(&self) -> IdentityMatcherId {
        self.id
    }

    pub fn semantic_name(&self) -> &IdentityMatcherName {
        &self.semantic_name
    }

    pub fn version(&self) -> IdentityMatcherVersion {
        self.version
    }

    pub fn policy(&self) -> IdentityMatchPolicy {
        self.policy
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityMatcherRegistration {
    descriptor: IdentityMatcherDescriptor,
}

impl IdentityMatcherRegistration {
    pub fn new(
        descriptor: IdentityMatcherDescriptor,
    ) -> Result<Self, crate::data::error::SignalError> {
        if descriptor.semantic_name().as_str().trim().is_empty() {
            return Err(crate::data::error::SignalError::invalid_input(
                "identity matcher semantic_name must not be empty",
            ));
        }
        Ok(Self { descriptor })
    }

    pub fn descriptor(&self) -> &IdentityMatcherDescriptor {
        &self.descriptor
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DuplicateIdentityMatcherRegistration {
    Id { id: IdentityMatcherId },
    SemanticName { name: IdentityMatcherName },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FrozenIdentityMatcherRegistry {
    registrations: Arc<[IdentityMatcherRegistration]>,
    index_by_id: Arc<BTreeMap<IdentityMatcherId, usize>>,
    index_by_name: Arc<BTreeMap<IdentityMatcherName, usize>>,
    registry_digest: String,
}

impl FrozenIdentityMatcherRegistry {
    pub fn from_registrations(
        registrations: Vec<IdentityMatcherRegistration>,
    ) -> Result<Self, DuplicateIdentityMatcherRegistration> {
        let mut ids = BTreeMap::new();
        let mut names = BTreeMap::new();
        let mut digest_basis = Vec::with_capacity(registrations.len());

        for (index, registration) in registrations.iter().enumerate() {
            let descriptor = registration.descriptor();
            if ids.insert(descriptor.id(), index).is_some() {
                return Err(DuplicateIdentityMatcherRegistration::Id {
                    id: descriptor.id(),
                });
            }
            if names
                .insert(descriptor.semantic_name().clone(), index)
                .is_some()
            {
                return Err(DuplicateIdentityMatcherRegistration::SemanticName {
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
            IdentityMatcherRegistration::new(IdentityMatcherDescriptor::new(
                IdentityMatcherId(1),
                IdentityMatcherName::new("signal.identity.exact-node-id"),
                IdentityMatcherVersion::new(1, 0),
                IdentityMatchPolicy::ExactNodeId,
            ))
            .expect("built-in identity matcher"),
            IdentityMatcherRegistration::new(IdentityMatcherDescriptor::new(
                IdentityMatcherId(2),
                IdentityMatcherName::new("signal.identity.output-identity-in-target-journal"),
                IdentityMatcherVersion::new(1, 0),
                IdentityMatchPolicy::OutputIdentityWithinTargetJournal,
            ))
            .expect("built-in identity matcher"),
        ])
        .expect("valid built-in identity matcher registry")
    }

    pub fn resolve_by_name(
        &self,
        name: &IdentityMatcherName,
    ) -> Option<&IdentityMatcherDescriptor> {
        self.index_by_name
            .get(name)
            .and_then(|index| self.registrations.get(*index))
            .map(IdentityMatcherRegistration::descriptor)
    }

    pub fn first_matching_policy(
        &self,
        policy: IdentityMatchPolicy,
    ) -> Option<&IdentityMatcherDescriptor> {
        self.registrations
            .iter()
            .find(|registration| registration.descriptor().policy() == policy)
            .map(IdentityMatcherRegistration::descriptor)
    }

    pub fn registry_digest(&self) -> &str {
        &self.registry_digest
    }
}

fn descriptor_digest(
    id: IdentityMatcherId,
    semantic_name: &IdentityMatcherName,
    version: IdentityMatcherVersion,
    policy: IdentityMatchPolicy,
) -> String {
    let canonical = serde_json::json!({
        "id": id.0,
        "semantic_name": semantic_name.as_str(),
        "version": { "major": version.major, "minor": version.minor },
        "policy": policy,
    });
    let bytes = serde_json::to_vec(&canonical).expect("identity matcher descriptor serialization");
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn registry_digest(descriptors: &[IdentityMatcherDescriptor]) -> String {
    let mut canonical = descriptors.to_vec();
    canonical.sort_by(|left, right| {
        left.semantic_name()
            .cmp(right.semantic_name())
            .then_with(|| left.version().cmp(&right.version()))
            .then_with(|| left.id().cmp(&right.id()))
    });
    let bytes = serde_json::to_vec(&canonical).expect("identity matcher registry serialization");
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

impl RetainedStorageMeasurement for IdentityMatcherName {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        self.0.retained_heap_charge(work)
    }
}
