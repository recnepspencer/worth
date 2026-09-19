use crate::application_schema::{ApplicationOperationRef, ApplicationStructuredValueBinding};
use crate::portable_identity::WorthQueryPortableTypeIdentity;

use super::{
    ApplicationCapabilityContextEntitySlotBinding, ApplicationCapabilityOperationBinding,
    ApplicationCapabilityRef,
};
use super::{ApplicationCapabilityLifecycleEffect, ApplicationCapabilityLifecycleEffectBinding};

mod portable_parts;
pub use portable_parts::{
    WorthQueryPortableApplicationCapabilityElevationLifecycleParts,
    WorthQueryPortableApplicationCapabilityTransitionBindingParts,
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityTransitionBinding {
    capability: String,
    capability_type: WorthQueryPortableTypeIdentity,
    operation: ApplicationCapabilityOperationBinding,
    lifecycle_effect: Option<ApplicationCapabilityLifecycleEffectBinding>,
}

impl ApplicationCapabilityTransitionBinding {
    pub fn from_references<Schema, Capability, Operation, Input>(
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
    ) -> Self {
        Self {
            capability: capability.name().to_string(),
            capability_type: capability.marker_identity(),
            operation: ApplicationCapabilityOperationBinding::from_reference(operation),
            lifecycle_effect: None,
        }
    }

    pub fn from_references_with_lifecycle_effect<Schema, Capability, Operation, Input>(
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
    ) -> Self
    where
        Input: ApplicationCapabilityLifecycleEffect<Schema, Operation>,
        <Input::PayloadBinding as ApplicationStructuredValueBinding>::Value: Send + Sync,
    {
        Self {
            capability: capability.name().to_string(),
            capability_type: capability.marker_identity(),
            operation: ApplicationCapabilityOperationBinding::from_reference(operation),
            lifecycle_effect: Some(ApplicationCapabilityLifecycleEffectBinding::from_input::<
                Schema,
                Operation,
                Input,
            >()),
        }
    }

    pub fn capability(&self) -> &str {
        &self.capability
    }

    pub fn capability_type(&self) -> &str {
        self.capability_type.as_str()
    }

    pub fn capability_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.capability_type.clone()
    }

    pub const fn operation(&self) -> &ApplicationCapabilityOperationBinding {
        &self.operation
    }

    pub const fn lifecycle_effect(&self) -> Option<&ApplicationCapabilityLifecycleEffectBinding> {
        self.lifecycle_effect.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityElevationLifecycleDefinition {
    elevation_slot: ApplicationCapabilityContextEntitySlotBinding,
    review_slot: ApplicationCapabilityContextEntitySlotBinding,
    request: ApplicationCapabilityTransitionBinding,
    approve: ApplicationCapabilityTransitionBinding,
    revoke: ApplicationCapabilityTransitionBinding,
    complete_review: ApplicationCapabilityTransitionBinding,
}

impl ApplicationCapabilityElevationLifecycleDefinition {
    pub fn new(
        elevation_slot: ApplicationCapabilityContextEntitySlotBinding,
        review_slot: ApplicationCapabilityContextEntitySlotBinding,
        request: ApplicationCapabilityTransitionBinding,
        approve: ApplicationCapabilityTransitionBinding,
        revoke: ApplicationCapabilityTransitionBinding,
        complete_review: ApplicationCapabilityTransitionBinding,
    ) -> Self {
        Self {
            elevation_slot,
            review_slot,
            request,
            approve,
            revoke,
            complete_review,
        }
    }

    pub const fn elevation_slot(&self) -> &ApplicationCapabilityContextEntitySlotBinding {
        &self.elevation_slot
    }

    pub const fn review_slot(&self) -> &ApplicationCapabilityContextEntitySlotBinding {
        &self.review_slot
    }

    pub const fn request(&self) -> &ApplicationCapabilityTransitionBinding {
        &self.request
    }

    pub const fn approve(&self) -> &ApplicationCapabilityTransitionBinding {
        &self.approve
    }

    pub const fn revoke(&self) -> &ApplicationCapabilityTransitionBinding {
        &self.revoke
    }

    pub const fn complete_review(&self) -> &ApplicationCapabilityTransitionBinding {
        &self.complete_review
    }

    pub fn transitions(&self) -> [&ApplicationCapabilityTransitionBinding; 4] {
        [
            &self.request,
            &self.approve,
            &self.revoke,
            &self.complete_review,
        ]
    }
}
