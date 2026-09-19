use std::time::Duration;

use super::{ApplicationCapabilityElevationDefinition, ApplicationCapabilityElevationRule};
use crate::application_capability::{
    ApplicationCapabilityElevationLifecycleDefinition, ApplicationCapabilityElevationStates,
    ApplicationCapabilityFieldBinding, ApplicationCapabilityMandatoryReviewDefinition,
    ApplicationCapabilityRelationBinding, ApplicationCapabilityValidityDefinition,
    WorthQueryPortableApplicationCapabilityElevationLifecycleParts,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationCapabilityElevationDefinitionParts {
    pub identity: ApplicationCapabilityFieldBinding,
    pub reason: ApplicationCapabilityFieldBinding,
    pub status: ApplicationCapabilityFieldBinding,
    pub closed_at: ApplicationCapabilityFieldBinding,
    pub states: ApplicationCapabilityElevationStates,
    pub validity: ApplicationCapabilityValidityDefinition,
    pub maximum_duration: Duration,
    pub requester: ApplicationCapabilityRelationBinding,
    pub approver: ApplicationCapabilityRelationBinding,
    pub grant: ApplicationCapabilityRelationBinding,
    pub resource_relation: Option<ApplicationCapabilityRelationBinding>,
    pub lifecycle: WorthQueryPortableApplicationCapabilityElevationLifecycleParts,
    pub review: ApplicationCapabilityMandatoryReviewDefinition,
}

impl ApplicationCapabilityElevationDefinition {
    pub fn from_untrusted_parts(
        parts: WorthQueryPortableApplicationCapabilityElevationDefinitionParts,
    ) -> Self {
        Self {
            identity: parts.identity,
            reason: parts.reason,
            status: parts.status,
            closed_at: parts.closed_at,
            states: parts.states,
            validity: parts.validity,
            maximum_duration: parts.maximum_duration,
            requester: parts.requester,
            approver: parts.approver,
            grant: parts.grant,
            resource_relation: parts.resource_relation,
            lifecycle: ApplicationCapabilityElevationLifecycleDefinition::from_untrusted_parts(
                parts.lifecycle,
            ),
            review: parts.review,
        }
    }

    pub fn parts(&self) -> WorthQueryPortableApplicationCapabilityElevationDefinitionParts {
        WorthQueryPortableApplicationCapabilityElevationDefinitionParts {
            identity: self.identity.clone(),
            reason: self.reason.clone(),
            status: self.status.clone(),
            closed_at: self.closed_at.clone(),
            states: self.states.clone(),
            validity: self.validity.clone(),
            maximum_duration: self.maximum_duration,
            requester: self.requester.clone(),
            approver: self.approver.clone(),
            grant: self.grant.clone(),
            resource_relation: self.resource_relation.clone(),
            lifecycle: self.lifecycle.parts(),
            review: self.review.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryPortableApplicationCapabilityElevationRuleParts {
    NotApplicable,
    Governed(WorthQueryPortableApplicationCapabilityElevationDefinitionParts),
}

impl ApplicationCapabilityElevationRule {
    pub fn from_untrusted_parts(
        parts: WorthQueryPortableApplicationCapabilityElevationRuleParts,
    ) -> Self {
        match parts {
            WorthQueryPortableApplicationCapabilityElevationRuleParts::NotApplicable => {
                Self::NotApplicable
            }
            WorthQueryPortableApplicationCapabilityElevationRuleParts::Governed(definition) => {
                Self::Governed(
                    ApplicationCapabilityElevationDefinition::from_untrusted_parts(definition),
                )
            }
        }
    }

    pub fn parts(&self) -> WorthQueryPortableApplicationCapabilityElevationRuleParts {
        match self {
            Self::NotApplicable => {
                WorthQueryPortableApplicationCapabilityElevationRuleParts::NotApplicable
            }
            Self::Governed(definition) => {
                WorthQueryPortableApplicationCapabilityElevationRuleParts::Governed(
                    definition.parts(),
                )
            }
        }
    }
}
