use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::policy::ConflictIsolationGranularity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ConflictIsolationPolicyId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ConflictIsolationPolicyName(String);

impl ConflictIsolationPolicyName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ConflictIsolationPolicyVersion {
    pub major: u16,
    pub minor: u16,
}

impl ConflictIsolationPolicyVersion {
    pub fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictIsolationSelectionBasis {
    RequestNamed,
    NodeOverride,
    SchemaDefault,
    BuiltInDefault,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictIsolationPolicyDescriptor {
    id: ConflictIsolationPolicyId,
    semantic_name: ConflictIsolationPolicyName,
    version: ConflictIsolationPolicyVersion,
    granularity: ConflictIsolationGranularity,
    digest: String,
}

impl ConflictIsolationPolicyDescriptor {
    pub fn new(
        id: ConflictIsolationPolicyId,
        semantic_name: ConflictIsolationPolicyName,
        version: ConflictIsolationPolicyVersion,
        granularity: ConflictIsolationGranularity,
    ) -> Self {
        let digest = descriptor_digest(id, &semantic_name, version, granularity);
        Self {
            id,
            semantic_name,
            version,
            granularity,
            digest,
        }
    }

    pub fn id(&self) -> ConflictIsolationPolicyId {
        self.id
    }

    pub fn semantic_name(&self) -> &ConflictIsolationPolicyName {
        &self.semantic_name
    }

    pub fn version(&self) -> ConflictIsolationPolicyVersion {
        self.version
    }

    pub fn granularity(&self) -> ConflictIsolationGranularity {
        self.granularity
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictIsolationPolicyRegistration {
    descriptor: ConflictIsolationPolicyDescriptor,
}

impl ConflictIsolationPolicyRegistration {
    pub fn new(
        descriptor: ConflictIsolationPolicyDescriptor,
    ) -> Result<Self, crate::data::error::SignalError> {
        if descriptor.semantic_name().as_str().trim().is_empty() {
            return Err(crate::data::error::SignalError::invalid_input(
                "conflict isolation policy semantic_name must not be empty",
            ));
        }
        Ok(Self { descriptor })
    }

    pub fn descriptor(&self) -> &ConflictIsolationPolicyDescriptor {
        &self.descriptor
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DuplicateConflictIsolationPolicyRegistration {
    Id { id: ConflictIsolationPolicyId },
    SemanticName { name: ConflictIsolationPolicyName },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FrozenConflictIsolationRegistry {
    registrations: Arc<[ConflictIsolationPolicyRegistration]>,
    index_by_id: Arc<BTreeMap<ConflictIsolationPolicyId, usize>>,
    index_by_name: Arc<BTreeMap<ConflictIsolationPolicyName, usize>>,
    registry_digest: String,
}

impl FrozenConflictIsolationRegistry {
    pub fn from_registrations(
        registrations: Vec<ConflictIsolationPolicyRegistration>,
    ) -> Result<Self, DuplicateConflictIsolationPolicyRegistration> {
        let mut ids = BTreeMap::new();
        let mut names = BTreeMap::new();
        let mut digest_basis = Vec::with_capacity(registrations.len());
        for (index, registration) in registrations.iter().enumerate() {
            let descriptor = registration.descriptor();
            if ids.insert(descriptor.id(), index).is_some() {
                return Err(DuplicateConflictIsolationPolicyRegistration::Id {
                    id: descriptor.id(),
                });
            }
            if names
                .insert(descriptor.semantic_name().clone(), index)
                .is_some()
            {
                return Err(DuplicateConflictIsolationPolicyRegistration::SemanticName {
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
            ConflictIsolationPolicyRegistration::new(ConflictIsolationPolicyDescriptor::new(
                ConflictIsolationPolicyId(1),
                ConflictIsolationPolicyName::new("signal.conflict-isolation.per-node"),
                ConflictIsolationPolicyVersion::new(1, 0),
                ConflictIsolationGranularity::PerNode,
            ))
            .expect("built-in conflict isolation policy"),
            ConflictIsolationPolicyRegistration::new(ConflictIsolationPolicyDescriptor::new(
                ConflictIsolationPolicyId(2),
                ConflictIsolationPolicyName::new("signal.conflict-isolation.per-aspect"),
                ConflictIsolationPolicyVersion::new(1, 0),
                ConflictIsolationGranularity::PerAspect,
            ))
            .expect("built-in conflict isolation policy"),
        ])
        .expect("valid built-in conflict isolation registry")
    }

    pub fn resolve_by_name(
        &self,
        name: &ConflictIsolationPolicyName,
    ) -> Option<&ConflictIsolationPolicyDescriptor> {
        self.index_by_name
            .get(name)
            .and_then(|index| self.registrations.get(*index))
            .map(ConflictIsolationPolicyRegistration::descriptor)
    }

    pub fn registry_digest(&self) -> &str {
        &self.registry_digest
    }
}

fn descriptor_digest(
    id: ConflictIsolationPolicyId,
    semantic_name: &ConflictIsolationPolicyName,
    version: ConflictIsolationPolicyVersion,
    granularity: ConflictIsolationGranularity,
) -> String {
    let canonical = serde_json::json!({
        "id": id.0,
        "semantic_name": semantic_name.as_str(),
        "version": { "major": version.major, "minor": version.minor },
        "granularity": granularity,
    });
    let bytes =
        serde_json::to_vec(&canonical).expect("conflict isolation descriptor serialization");
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn registry_digest(descriptors: &[ConflictIsolationPolicyDescriptor]) -> String {
    let mut canonical = descriptors.to_vec();
    canonical.sort_by(|left, right| {
        left.semantic_name()
            .cmp(right.semantic_name())
            .then_with(|| left.version().cmp(&right.version()))
            .then_with(|| left.id().cmp(&right.id()))
    });
    let bytes =
        serde_json::to_vec(&canonical).expect("conflict isolation policy registry serialization");
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

impl RetainedStorageMeasurement for ConflictIsolationPolicyName {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        self.0.retained_heap_charge(work)
    }
}
