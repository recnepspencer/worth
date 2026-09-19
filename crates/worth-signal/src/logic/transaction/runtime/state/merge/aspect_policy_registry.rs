use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::data::aspect::Aspect;

use super::policy::AspectMergePolicy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AspectMergePolicyId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AspectMergePolicyName(String);

impl AspectMergePolicyName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AspectMergePolicyVersion {
    pub major: u16,
    pub minor: u16,
}

impl AspectMergePolicyVersion {
    pub fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AspectMergePolicySelectionBasis {
    RequestNamed,
    NodeOverride,
    SchemaDefault,
    BuiltInDefault,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AspectMergePolicyBinding {
    pub aspect: Aspect,
    pub policy_name: AspectMergePolicyName,
}

impl AspectMergePolicyBinding {
    pub fn new(aspect: Aspect, policy_name: AspectMergePolicyName) -> Self {
        Self {
            aspect,
            policy_name,
        }
    }

    pub fn aspect(&self) -> Aspect {
        self.aspect
    }

    pub fn policy_name(&self) -> &AspectMergePolicyName {
        &self.policy_name
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AspectMergePolicyDescriptor {
    id: AspectMergePolicyId,
    semantic_name: AspectMergePolicyName,
    version: AspectMergePolicyVersion,
    policy: AspectMergePolicy,
    digest: String,
}

impl AspectMergePolicyDescriptor {
    pub fn new(
        id: AspectMergePolicyId,
        semantic_name: AspectMergePolicyName,
        version: AspectMergePolicyVersion,
        policy: AspectMergePolicy,
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

    pub fn id(&self) -> AspectMergePolicyId {
        self.id
    }

    pub fn semantic_name(&self) -> &AspectMergePolicyName {
        &self.semantic_name
    }

    pub fn version(&self) -> AspectMergePolicyVersion {
        self.version
    }

    pub fn policy(&self) -> AspectMergePolicy {
        self.policy
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AspectMergePolicyRegistration {
    descriptor: AspectMergePolicyDescriptor,
}

impl AspectMergePolicyRegistration {
    pub fn new(
        descriptor: AspectMergePolicyDescriptor,
    ) -> Result<Self, crate::data::error::SignalError> {
        if descriptor.semantic_name().as_str().trim().is_empty() {
            return Err(crate::data::error::SignalError::invalid_input(
                "aspect merge policy semantic_name must not be empty",
            ));
        }
        Ok(Self { descriptor })
    }

    pub fn descriptor(&self) -> &AspectMergePolicyDescriptor {
        &self.descriptor
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DuplicateAspectMergePolicyRegistration {
    Id { id: AspectMergePolicyId },
    SemanticName { name: AspectMergePolicyName },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FrozenAspectMergePolicyRegistry {
    registrations: Arc<[AspectMergePolicyRegistration]>,
    index_by_id: Arc<BTreeMap<AspectMergePolicyId, usize>>,
    index_by_name: Arc<BTreeMap<AspectMergePolicyName, usize>>,
    registry_digest: String,
}

impl FrozenAspectMergePolicyRegistry {
    pub fn from_registrations(
        registrations: Vec<AspectMergePolicyRegistration>,
    ) -> Result<Self, DuplicateAspectMergePolicyRegistration> {
        let mut ids = BTreeMap::new();
        let mut names = BTreeMap::new();
        let mut digest_basis = Vec::with_capacity(registrations.len());
        for (index, registration) in registrations.iter().enumerate() {
            let descriptor = registration.descriptor();
            if ids.insert(descriptor.id(), index).is_some() {
                return Err(DuplicateAspectMergePolicyRegistration::Id {
                    id: descriptor.id(),
                });
            }
            if names
                .insert(descriptor.semantic_name().clone(), index)
                .is_some()
            {
                return Err(DuplicateAspectMergePolicyRegistration::SemanticName {
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
            AspectMergePolicyRegistration::new(AspectMergePolicyDescriptor::new(
                AspectMergePolicyId(1),
                AspectMergePolicyName::new("signal.aspect.require-conflict"),
                AspectMergePolicyVersion::new(1, 0),
                AspectMergePolicy::RequireConflict,
            ))
            .expect("built-in aspect merge policy"),
            AspectMergePolicyRegistration::new(AspectMergePolicyDescriptor::new(
                AspectMergePolicyId(2),
                AspectMergePolicyName::new("signal.aspect.prefer-source"),
                AspectMergePolicyVersion::new(1, 0),
                AspectMergePolicy::PreferSource,
            ))
            .expect("built-in aspect merge policy"),
            AspectMergePolicyRegistration::new(AspectMergePolicyDescriptor::new(
                AspectMergePolicyId(3),
                AspectMergePolicyName::new("signal.aspect.prefer-target"),
                AspectMergePolicyVersion::new(1, 0),
                AspectMergePolicy::PreferTarget,
            ))
            .expect("built-in aspect merge policy"),
        ])
        .expect("valid built-in aspect merge policy registry")
    }

    pub fn resolve_by_name(
        &self,
        name: &AspectMergePolicyName,
    ) -> Option<&AspectMergePolicyDescriptor> {
        self.index_by_name
            .get(name)
            .and_then(|index| self.registrations.get(*index))
            .map(AspectMergePolicyRegistration::descriptor)
    }

    pub fn registry_digest(&self) -> &str {
        &self.registry_digest
    }
}

fn descriptor_digest(
    id: AspectMergePolicyId,
    semantic_name: &AspectMergePolicyName,
    version: AspectMergePolicyVersion,
    policy: AspectMergePolicy,
) -> String {
    let canonical = serde_json::json!({
        "id": id.0,
        "semantic_name": semantic_name.as_str(),
        "version": { "major": version.major, "minor": version.minor },
        "policy": policy,
    });
    let bytes =
        serde_json::to_vec(&canonical).expect("aspect merge policy descriptor serialization");
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn registry_digest(descriptors: &[AspectMergePolicyDescriptor]) -> String {
    let mut canonical = descriptors.to_vec();
    canonical.sort_by(|left, right| {
        left.semantic_name()
            .cmp(right.semantic_name())
            .then_with(|| left.version().cmp(&right.version()))
            .then_with(|| left.id().cmp(&right.id()))
    });
    let bytes = serde_json::to_vec(&canonical).expect("aspect merge policy registry serialization");
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

impl RetainedStorageMeasurement for AspectMergePolicyName {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        self.0.retained_heap_charge(work)
    }
}

impl RetainedStorageMeasurement for AspectMergePolicyBinding {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            policy_name,
            aspect: _,
        } = self;
        Ok(Charge::ZERO.checked_add(policy_name.retained_heap_charge(work)?)?)
    }
}
