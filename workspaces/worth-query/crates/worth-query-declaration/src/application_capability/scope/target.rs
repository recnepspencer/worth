//! Capability targets distinguish the indexed grant resource from an admitted descendant scope.
use super::{
    ApplicationCapabilityFieldDimension, ApplicationCapabilityRelationBinding,
    ApplicationCapabilityRelationDimension, ApplicationCapabilityValueBinding,
};
use crate::application_capability::ApplicationCapabilityResourcePath;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityTargetDefinition {
    action: ApplicationCapabilityValueBinding,
    resource: ApplicationCapabilityResourcePath,
    relation: ApplicationCapabilityRelationDimension,
    field: ApplicationCapabilityFieldDimension,
    purpose: ApplicationCapabilityValueBinding,
}

impl ApplicationCapabilityTargetDefinition {
    pub fn new(
        action: ApplicationCapabilityValueBinding,
        resource: impl Into<ApplicationCapabilityResourcePath>,
        relation: ApplicationCapabilityRelationDimension,
        field: ApplicationCapabilityFieldDimension,
        purpose: ApplicationCapabilityValueBinding,
    ) -> Self {
        Self {
            action,
            resource: resource.into(),
            relation,
            field,
            purpose,
        }
    }

    pub fn action(&self) -> &ApplicationCapabilityValueBinding {
        &self.action
    }

    /// Indexed grant-to-resource head. For a rooted extent this is not the final mutation scope.
    pub fn resource(&self) -> &ApplicationCapabilityRelationBinding {
        self.resource.head()
    }

    /// Complete declared extent; `scope_entity()` names the actual admitted request scope.
    pub fn resource_path(&self) -> &ApplicationCapabilityResourcePath {
        &self.resource
    }

    pub const fn relation(&self) -> &ApplicationCapabilityRelationDimension {
        &self.relation
    }

    pub const fn field(&self) -> &ApplicationCapabilityFieldDimension {
        &self.field
    }

    pub fn purpose(&self) -> &ApplicationCapabilityValueBinding {
        &self.purpose
    }
}
