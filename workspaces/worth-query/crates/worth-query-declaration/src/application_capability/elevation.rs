use super::{
    ApplicationCapabilityElevationLifecycleDefinition, ApplicationCapabilityFieldBinding,
    ApplicationCapabilityRelationBinding, ApplicationCapabilityValidityDefinition,
    ApplicationCapabilityValueBinding,
};

use std::time::Duration;

mod portable_parts;
pub use portable_parts::{
    WorthQueryPortableApplicationCapabilityElevationDefinitionParts,
    WorthQueryPortableApplicationCapabilityElevationRuleParts,
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityElevationStates {
    requested: ApplicationCapabilityValueBinding,
    approved: ApplicationCapabilityValueBinding,
    expired: ApplicationCapabilityValueBinding,
    revoked: ApplicationCapabilityValueBinding,
}

impl ApplicationCapabilityElevationStates {
    pub fn new(
        requested: ApplicationCapabilityValueBinding,
        approved: ApplicationCapabilityValueBinding,
        expired: ApplicationCapabilityValueBinding,
        revoked: ApplicationCapabilityValueBinding,
    ) -> Self {
        Self {
            requested,
            approved,
            expired,
            revoked,
        }
    }

    pub const fn requested(&self) -> &ApplicationCapabilityValueBinding {
        &self.requested
    }

    pub const fn approved(&self) -> &ApplicationCapabilityValueBinding {
        &self.approved
    }

    pub const fn expired(&self) -> &ApplicationCapabilityValueBinding {
        &self.expired
    }

    pub const fn revoked(&self) -> &ApplicationCapabilityValueBinding {
        &self.revoked
    }

    pub fn values(&self) -> [&ApplicationCapabilityValueBinding; 4] {
        [
            &self.requested,
            &self.approved,
            &self.expired,
            &self.revoked,
        ]
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityMandatoryReviewDefinition {
    relation: ApplicationCapabilityRelationBinding,
    identity: ApplicationCapabilityFieldBinding,
    kind: ApplicationCapabilityValueBinding,
    scope: ApplicationCapabilityRelationBinding,
    reviewer: ApplicationCapabilityRelationBinding,
    status: ApplicationCapabilityFieldBinding,
    reviewed_at: ApplicationCapabilityFieldBinding,
    required: ApplicationCapabilityValueBinding,
    completed: ApplicationCapabilityValueBinding,
}

impl ApplicationCapabilityMandatoryReviewDefinition {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        relation: ApplicationCapabilityRelationBinding,
        identity: ApplicationCapabilityFieldBinding,
        kind: ApplicationCapabilityValueBinding,
        scope: ApplicationCapabilityRelationBinding,
        reviewer: ApplicationCapabilityRelationBinding,
        status: ApplicationCapabilityFieldBinding,
        reviewed_at: ApplicationCapabilityFieldBinding,
        required: ApplicationCapabilityValueBinding,
        completed: ApplicationCapabilityValueBinding,
    ) -> Self {
        Self {
            relation,
            identity,
            kind,
            scope,
            reviewer,
            status,
            reviewed_at,
            required,
            completed,
        }
    }

    pub const fn relation(&self) -> &ApplicationCapabilityRelationBinding {
        &self.relation
    }

    pub const fn identity(&self) -> &ApplicationCapabilityFieldBinding {
        &self.identity
    }

    pub const fn kind(&self) -> &ApplicationCapabilityValueBinding {
        &self.kind
    }

    pub const fn scope(&self) -> &ApplicationCapabilityRelationBinding {
        &self.scope
    }

    pub const fn reviewer(&self) -> &ApplicationCapabilityRelationBinding {
        &self.reviewer
    }

    pub const fn status(&self) -> &ApplicationCapabilityFieldBinding {
        &self.status
    }

    pub const fn reviewed_at(&self) -> &ApplicationCapabilityFieldBinding {
        &self.reviewed_at
    }

    pub const fn required(&self) -> &ApplicationCapabilityValueBinding {
        &self.required
    }

    pub const fn completed(&self) -> &ApplicationCapabilityValueBinding {
        &self.completed
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityElevationDefinition {
    identity: ApplicationCapabilityFieldBinding,
    reason: ApplicationCapabilityFieldBinding,
    status: ApplicationCapabilityFieldBinding,
    closed_at: ApplicationCapabilityFieldBinding,
    states: ApplicationCapabilityElevationStates,
    validity: ApplicationCapabilityValidityDefinition,
    maximum_duration: Duration,
    requester: ApplicationCapabilityRelationBinding,
    approver: ApplicationCapabilityRelationBinding,
    grant: ApplicationCapabilityRelationBinding,
    resource_relation: Option<ApplicationCapabilityRelationBinding>,
    lifecycle: ApplicationCapabilityElevationLifecycleDefinition,
    review: ApplicationCapabilityMandatoryReviewDefinition,
}

impl ApplicationCapabilityElevationDefinition {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        identity: ApplicationCapabilityFieldBinding,
        reason: ApplicationCapabilityFieldBinding,
        status: ApplicationCapabilityFieldBinding,
        closed_at: ApplicationCapabilityFieldBinding,
        states: ApplicationCapabilityElevationStates,
        validity: ApplicationCapabilityValidityDefinition,
        maximum_duration: Duration,
        requester: ApplicationCapabilityRelationBinding,
        approver: ApplicationCapabilityRelationBinding,
        grant: ApplicationCapabilityRelationBinding,
        lifecycle: ApplicationCapabilityElevationLifecycleDefinition,
        review: ApplicationCapabilityMandatoryReviewDefinition,
    ) -> Self {
        Self {
            identity,
            reason,
            status,
            closed_at,
            states,
            validity,
            maximum_duration,
            requester,
            approver,
            grant,
            resource_relation: None,
            lifecycle,
            review,
        }
    }

    pub const fn identity(&self) -> &ApplicationCapabilityFieldBinding {
        &self.identity
    }

    pub const fn reason(&self) -> &ApplicationCapabilityFieldBinding {
        &self.reason
    }

    pub const fn status(&self) -> &ApplicationCapabilityFieldBinding {
        &self.status
    }

    pub const fn closed_at(&self) -> &ApplicationCapabilityFieldBinding {
        &self.closed_at
    }

    pub const fn states(&self) -> &ApplicationCapabilityElevationStates {
        &self.states
    }

    pub const fn validity(&self) -> &ApplicationCapabilityValidityDefinition {
        &self.validity
    }

    pub const fn maximum_duration(&self) -> Duration {
        self.maximum_duration
    }

    pub const fn requester(&self) -> &ApplicationCapabilityRelationBinding {
        &self.requester
    }

    pub const fn approver(&self) -> &ApplicationCapabilityRelationBinding {
        &self.approver
    }

    pub const fn grant(&self) -> &ApplicationCapabilityRelationBinding {
        &self.grant
    }

    pub fn with_resource_relation(
        mut self,
        resource_relation: ApplicationCapabilityRelationBinding,
    ) -> Self {
        self.resource_relation = Some(resource_relation);
        self
    }

    pub const fn resource_relation(&self) -> Option<&ApplicationCapabilityRelationBinding> {
        self.resource_relation.as_ref()
    }

    pub const fn lifecycle(&self) -> &ApplicationCapabilityElevationLifecycleDefinition {
        &self.lifecycle
    }

    pub const fn review(&self) -> &ApplicationCapabilityMandatoryReviewDefinition {
        &self.review
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationCapabilityElevationRule {
    NotApplicable,
    Governed(ApplicationCapabilityElevationDefinition),
}

impl ApplicationCapabilityElevationRule {
    pub const fn not_applicable() -> Self {
        Self::NotApplicable
    }

    pub const fn governed(definition: ApplicationCapabilityElevationDefinition) -> Self {
        Self::Governed(definition)
    }

    pub const fn definition(&self) -> Option<&ApplicationCapabilityElevationDefinition> {
        match self {
            Self::NotApplicable => None,
            Self::Governed(definition) => Some(definition),
        }
    }
}
