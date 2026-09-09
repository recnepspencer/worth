use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::policy::SourceOnlyMergePolicy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SourceOnlyPolicyId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SourceOnlyPolicyName(String);

impl SourceOnlyPolicyName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SourceOnlyPolicyVersion {
    pub major: u16,
    pub minor: u16,
}

impl SourceOnlyPolicyVersion {
    pub fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceOnlyPolicySelectionBasis {
    RequestNamed,
    NodeOverride,
    SchemaDefault,
    BuiltInDefault,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceOnlyPolicyDescriptor {
    id: SourceOnlyPolicyId,
    semantic_name: SourceOnlyPolicyName,
    version: SourceOnlyPolicyVersion,
    policy: SourceOnlyMergePolicy,
    digest: String,
}

impl SourceOnlyPolicyDescriptor {
    pub fn new(
        id: SourceOnlyPolicyId,
        semantic_name: SourceOnlyPolicyName,
        version: SourceOnlyPolicyVersion,
        policy: SourceOnlyMergePolicy,
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

    pub fn id(&self) -> SourceOnlyPolicyId {
        self.id
    }

    pub fn semantic_name(&self) -> &SourceOnlyPolicyName {
        &self.semantic_name
    }

    pub fn version(&self) -> SourceOnlyPolicyVersion {
        self.version
    }

    pub fn policy(&self) -> SourceOnlyMergePolicy {
        self.policy
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceOnlyPolicyRegistration {
    descriptor: SourceOnlyPolicyDescriptor,
}

impl SourceOnlyPolicyRegistration {
    pub fn new(
        descriptor: SourceOnlyPolicyDescriptor,
    ) -> Result<Self, crate::data::error::SignalError> {
        if descriptor.semantic_name().as_str().trim().is_empty() {
            return Err(crate::data::error::SignalError::invalid_input(
                "source-only policy semantic_name must not be empty",
            ));
        }
        Ok(Self { descriptor })
    }

    pub fn descriptor(&self) -> &SourceOnlyPolicyDescriptor {
        &self.descriptor
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DuplicateSourceOnlyPolicyRegistration {
    Id { id: SourceOnlyPolicyId },
    SemanticName { name: SourceOnlyPolicyName },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FrozenSourceOnlyPolicyRegistry {
    registrations: Arc<[SourceOnlyPolicyRegistration]>,
    index_by_id: Arc<BTreeMap<SourceOnlyPolicyId, usize>>,
    index_by_name: Arc<BTreeMap<SourceOnlyPolicyName, usize>>,
    registry_digest: String,
}

impl FrozenSourceOnlyPolicyRegistry {
    pub fn from_registrations(
        registrations: Vec<SourceOnlyPolicyRegistration>,
    ) -> Result<Self, DuplicateSourceOnlyPolicyRegistration> {
        let mut ids = BTreeMap::new();
        let mut names = BTreeMap::new();
        let mut digest_basis = Vec::with_capacity(registrations.len());
        for (index, registration) in registrations.iter().enumerate() {
            let descriptor = registration.descriptor();
            if ids.insert(descriptor.id(), index).is_some() {
                return Err(DuplicateSourceOnlyPolicyRegistration::Id {
                    id: descriptor.id(),
                });
            }
            if names
                .insert(descriptor.semantic_name().clone(), index)
                .is_some()
            {
                return Err(DuplicateSourceOnlyPolicyRegistration::SemanticName {
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
            SourceOnlyPolicyRegistration::new(SourceOnlyPolicyDescriptor::new(
                SourceOnlyPolicyId(1),
                SourceOnlyPolicyName::new(
                    "signal.source-only.introduce-adoptable-skip-non-adoptable",
                ),
                SourceOnlyPolicyVersion::new(1, 0),
                SourceOnlyMergePolicy::IntroduceAdoptableSkipNonAdoptable,
            ))
            .expect("built-in source-only policy"),
            SourceOnlyPolicyRegistration::new(SourceOnlyPolicyDescriptor::new(
                SourceOnlyPolicyId(2),
                SourceOnlyPolicyName::new("signal.source-only.reject-introduction"),
                SourceOnlyPolicyVersion::new(1, 0),
                SourceOnlyMergePolicy::RejectIntroduction,
            ))
            .expect("built-in source-only policy"),
        ])
        .expect("valid built-in source-only policy registry")
    }

    pub fn resolve_by_name(
        &self,
        name: &SourceOnlyPolicyName,
    ) -> Option<&SourceOnlyPolicyDescriptor> {
        self.index_by_name
            .get(name)
            .and_then(|index| self.registrations.get(*index))
            .map(SourceOnlyPolicyRegistration::descriptor)
    }

    pub fn first_matching_policy(
        &self,
        policy: SourceOnlyMergePolicy,
    ) -> Option<&SourceOnlyPolicyDescriptor> {
        self.registrations
            .iter()
            .find(|registration| registration.descriptor().policy() == policy)
            .map(SourceOnlyPolicyRegistration::descriptor)
    }

    pub fn registry_digest(&self) -> &str {
        &self.registry_digest
    }
}

fn descriptor_digest(
    id: SourceOnlyPolicyId,
    semantic_name: &SourceOnlyPolicyName,
    version: SourceOnlyPolicyVersion,
    policy: SourceOnlyMergePolicy,
) -> String {
    let canonical = serde_json::json!({
        "id": id.0,
        "semantic_name": semantic_name.as_str(),
        "version": { "major": version.major, "minor": version.minor },
        "policy": policy,
    });
    let bytes =
        serde_json::to_vec(&canonical).expect("source-only policy descriptor serialization");
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn registry_digest(descriptors: &[SourceOnlyPolicyDescriptor]) -> String {
    let mut canonical = descriptors.to_vec();
    canonical.sort_by(|left, right| {
        left.semantic_name()
            .cmp(right.semantic_name())
            .then_with(|| left.version().cmp(&right.version()))
            .then_with(|| left.id().cmp(&right.id()))
    });
    let bytes = serde_json::to_vec(&canonical).expect("source-only policy registry serialization");
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

impl RetainedStorageMeasurement for SourceOnlyPolicyName {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        self.0.retained_heap_charge(work)
    }
}
