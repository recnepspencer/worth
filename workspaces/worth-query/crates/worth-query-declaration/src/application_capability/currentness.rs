//! When an installed grant is current: its active status, its tenure, its
//! validity bounds and the liveness of the entities its resource path crosses.
use super::{ApplicationCapabilityFieldBinding, ApplicationCapabilityValueBinding};

mod validity;
pub use validity::{
    ApplicationCapabilityValidityDefinition, ApplicationCapabilityValidityTimeline,
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityWorkflowDefinition {
    grant: ApplicationCapabilityFieldBinding,
    resource: ApplicationCapabilityFieldBinding,
}

impl ApplicationCapabilityWorkflowDefinition {
    pub fn new(
        grant: ApplicationCapabilityFieldBinding,
        resource: ApplicationCapabilityFieldBinding,
    ) -> Self {
        Self { grant, resource }
    }

    pub fn grant(&self) -> &ApplicationCapabilityFieldBinding {
        &self.grant
    }

    pub fn resource(&self) -> &ApplicationCapabilityFieldBinding {
        &self.resource
    }
}

/// What a grant belongs to.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationCapabilityGrantTenure {
    /// One workflow instance: the grant's workflow value must equal the
    /// scope's, and the grant may carry a delegation lineage.
    Workflow(ApplicationCapabilityWorkflowDefinition),
    /// A standing site, organisation, workspace or membership grant. It
    /// belongs to no workflow instance and has no delegation lineage.
    Standing,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityCurrentnessDefinition {
    active_status: ApplicationCapabilityValueBinding,
    tenure: ApplicationCapabilityGrantTenure,
    validity: ApplicationCapabilityValidityDefinition,
    extent_status: Vec<ApplicationCapabilityValueBinding>,
}

impl ApplicationCapabilityCurrentnessDefinition {
    /// Installed ceiling on [`Self::require_active`] bindings per contract.
    pub const MAXIMUM_EXTENT_STATUS: usize = 8;

    /// A workflow-tenure grant (delegable, bound to one workflow instance).
    pub fn new(
        active_status: ApplicationCapabilityValueBinding,
        workflow: ApplicationCapabilityWorkflowDefinition,
        validity: ApplicationCapabilityValidityDefinition,
    ) -> Self {
        Self::with_tenure(
            active_status,
            ApplicationCapabilityGrantTenure::Workflow(workflow),
            validity,
        )
    }

    /// A standing grant: current while active and valid, with no workflow
    /// binding. Pair it with `ApplicationCapabilityDelegationDefinition::standing`.
    pub fn standing(
        active_status: ApplicationCapabilityValueBinding,
        validity: ApplicationCapabilityValidityDefinition,
    ) -> Self {
        Self::with_tenure(
            active_status,
            ApplicationCapabilityGrantTenure::Standing,
            validity,
        )
    }

    fn with_tenure(
        active_status: ApplicationCapabilityValueBinding,
        tenure: ApplicationCapabilityGrantTenure,
        validity: ApplicationCapabilityValidityDefinition,
    ) -> Self {
        Self {
            active_status,
            tenure,
            validity,
            extent_status: Vec::new(),
        }
    }

    /// Requires every entity of `status`'s kind on the grant's resource path
    /// (its head or root and each later step) to hold `status`. Deactivating a
    /// Site therefore revokes every grant reaching through it, without touching
    /// one grant row. Installation refuses a kind the path never crosses.
    pub fn require_active(mut self, status: ApplicationCapabilityValueBinding) -> Self {
        self.extent_status.push(status);
        self
    }

    pub fn active_status(&self) -> &ApplicationCapabilityValueBinding {
        &self.active_status
    }

    pub const fn tenure(&self) -> &ApplicationCapabilityGrantTenure {
        &self.tenure
    }

    /// The workflow binding of a workflow-tenure grant; `None` when standing.
    pub const fn workflow(&self) -> Option<&ApplicationCapabilityWorkflowDefinition> {
        match &self.tenure {
            ApplicationCapabilityGrantTenure::Workflow(workflow) => Some(workflow),
            ApplicationCapabilityGrantTenure::Standing => None,
        }
    }

    pub const fn validity(&self) -> &ApplicationCapabilityValidityDefinition {
        &self.validity
    }

    pub fn extent_status(&self) -> &[ApplicationCapabilityValueBinding] {
        &self.extent_status
    }
}
