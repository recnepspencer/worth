use std::collections::BTreeMap;
use std::sync::Arc;

use super::super::policy::ResourcePolicyName;
use super::descriptor::ResourcePolicyDescriptor;
use super::digest::registry_digest;
use super::errors::{ResourcePolicyRegistryError, ResourcePolicyResolutionError};
use super::families::built_in_policy_registrations;
use super::freeze_report::ResourcePolicyRegistryFreezeReport;
use super::identity::{
    ResourcePolicyCompatibilityPosture, ResourcePolicyDescriptorId, ResourcePolicyDigest,
    ResourcePolicyKind, ResourcePolicySelectionBasis,
};
use super::reference::ValidatedResourcePolicyReference;
use super::registration::ResourcePolicyRegistration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrozenResourcePolicyRegistry {
    descriptors: Arc<Vec<ResourcePolicyDescriptor>>,
    index_by_id: Arc<BTreeMap<ResourcePolicyDescriptorId, usize>>,
    index_by_kind_name: Arc<BTreeMap<(ResourcePolicyKind, ResourcePolicyName), usize>>,
    freeze_report: Arc<ResourcePolicyRegistryFreezeReport>,
}

impl FrozenResourcePolicyRegistry {
    pub fn new(
        registrations: Vec<ResourcePolicyRegistration>,
    ) -> Result<Self, ResourcePolicyRegistryError> {
        let mut descriptors = Vec::with_capacity(registrations.len());
        let mut index_by_id = BTreeMap::new();
        let mut index_by_kind_name = BTreeMap::new();

        for registration in registrations {
            validate_policy_registration_name(registration.kind(), registration.semantic_name())
                .map_err(|(kind, name, reason)| {
                    ResourcePolicyRegistryError::MalformedDescriptor { kind, name, reason }
                })?;
            let descriptor = ResourcePolicyDescriptor::new(
                registration.id(),
                registration.kind(),
                registration.semantic_name().clone(),
                registration.version(),
                registration.cost_contract(),
                registration.compatibility_posture(),
            );
            let index = descriptors.len();
            if index_by_id.insert(descriptor.id(), index).is_some() {
                return Err(ResourcePolicyRegistryError::DuplicateId(descriptor.id()));
            }
            let kind_name = (descriptor.kind(), descriptor.semantic_name().clone());
            if index_by_kind_name
                .insert(kind_name.clone(), index)
                .is_some()
            {
                let (kind, name) = kind_name;
                return Err(ResourcePolicyRegistryError::DuplicateName { kind, name });
            }
            descriptors.push(descriptor);
        }

        let registry_digest = registry_digest(&descriptors);
        let freeze_report = ResourcePolicyRegistryFreezeReport::new(
            descriptors.len(),
            index_by_id.len(),
            index_by_kind_name.len(),
            registry_digest,
        );

        Ok(Self {
            descriptors: Arc::new(descriptors),
            index_by_id: Arc::new(index_by_id),
            index_by_kind_name: Arc::new(index_by_kind_name),
            freeze_report: Arc::new(freeze_report),
        })
    }

    pub fn built_in() -> Self {
        Self::new(built_in_policy_registrations()).expect("built-in resource policy registry")
    }

    pub fn resolve_by_name(
        &self,
        kind: ResourcePolicyKind,
        name: &ResourcePolicyName,
    ) -> Option<&ResourcePolicyDescriptor> {
        self.index_by_kind_name
            .get(&(kind, name.clone()))
            .and_then(|index| self.descriptors.get(*index))
    }

    pub fn resolve_by_id(
        &self,
        id: ResourcePolicyDescriptorId,
    ) -> Option<&ResourcePolicyDescriptor> {
        self.index_by_id
            .get(&id)
            .and_then(|index| self.descriptors.get(*index))
    }

    pub fn descriptor_count(&self) -> usize {
        self.descriptors.len()
    }

    pub fn registry_digest(&self) -> &ResourcePolicyDigest {
        self.freeze_report.registry_digest()
    }

    pub fn freeze_report(&self) -> &ResourcePolicyRegistryFreezeReport {
        &self.freeze_report
    }

    #[cfg(test)]
    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.descriptors, &other.descriptors)
            && Arc::ptr_eq(&self.index_by_id, &other.index_by_id)
            && Arc::ptr_eq(&self.index_by_kind_name, &other.index_by_kind_name)
            && Arc::ptr_eq(&self.freeze_report, &other.freeze_report)
    }

    pub(super) fn resolve_named(
        &self,
        kind: ResourcePolicyKind,
        name: &ResourcePolicyName,
    ) -> Result<ValidatedResourcePolicyReference, ResourcePolicyResolutionError> {
        validate_named_policy_name(kind, name)?;
        let descriptor = self.resolve_by_name(kind, name).cloned().ok_or_else(|| {
            ResourcePolicyResolutionError::UnknownPolicy {
                kind,
                name: name.clone(),
            }
        })?;
        ensure_compatible_descriptor(kind, &descriptor)?;
        Ok(ValidatedResourcePolicyReference::new(
            descriptor,
            ResourcePolicySelectionBasis::DeclaredName,
            ResourcePolicyDigest::new(format!("named:{}", name.as_str())),
        ))
    }

    pub(super) fn built_in_policy(
        &self,
        kind: ResourcePolicyKind,
        name: &str,
        selection_basis: ResourcePolicySelectionBasis,
        parameter_digest: ResourcePolicyDigest,
    ) -> Result<ValidatedResourcePolicyReference, ResourcePolicyResolutionError> {
        let descriptor = self
            .resolve_by_name(kind, &ResourcePolicyName::new(name))
            .cloned()
            .ok_or_else(|| ResourcePolicyResolutionError::MissingDescriptor {
                kind,
                name: ResourcePolicyName::new(name),
            })?;
        ensure_compatible_descriptor(kind, &descriptor)?;
        Ok(ValidatedResourcePolicyReference::new(
            descriptor,
            selection_basis,
            parameter_digest,
        ))
    }
}

fn validate_named_policy_name(
    kind: ResourcePolicyKind,
    name: &ResourcePolicyName,
) -> Result<(), ResourcePolicyResolutionError> {
    validate_policy_registration_name(kind, name).map_err(|(kind, name, reason)| {
        ResourcePolicyResolutionError::MalformedDescriptor { kind, name, reason }
    })?;
    Ok(())
}

fn validate_policy_registration_name(
    kind: ResourcePolicyKind,
    name: &ResourcePolicyName,
) -> Result<(), (ResourcePolicyKind, ResourcePolicyName, &'static str)> {
    if name.as_str().trim().is_empty() {
        return Err((kind, name.clone(), "resource policy name must not be empty"));
    }
    Ok(())
}

fn ensure_compatible_descriptor(
    kind: ResourcePolicyKind,
    descriptor: &ResourcePolicyDescriptor,
) -> Result<(), ResourcePolicyResolutionError> {
    if descriptor.compatibility_posture() == ResourcePolicyCompatibilityPosture::IncompatibleVersion
    {
        return Err(ResourcePolicyResolutionError::IncompatibleDescriptor {
            kind,
            name: descriptor.semantic_name().clone(),
            version: descriptor.version(),
            compatibility_posture: descriptor.compatibility_posture(),
        });
    }
    Ok(())
}
