use crate::archive_decode_authority::{ArchiveDecodeAuthority, ARCHIVE_DECODE};
use crate::portable_identity::WorthQueryPortableTypeIdentity;

use super::{
    ApplicationCapabilityElevationLifecycleDefinition, ApplicationCapabilityTransitionBinding,
};
use crate::application_capability::{
    ApplicationCapabilityContextEntitySlotBinding, ApplicationCapabilityLifecycleEffectBinding,
    ApplicationCapabilityOperationBinding,
    WorthQueryPortableApplicationCapabilityLifecycleEffectParts,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationCapabilityTransitionBindingParts {
    pub capability: String,
    pub capability_type: WorthQueryPortableTypeIdentity,
    pub operation: ApplicationCapabilityOperationBinding,
    pub lifecycle_effect: Option<WorthQueryPortableApplicationCapabilityLifecycleEffectParts>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationCapabilityElevationLifecycleParts {
    pub elevation_slot: ApplicationCapabilityContextEntitySlotBinding,
    pub review_slot: ApplicationCapabilityContextEntitySlotBinding,
    pub request: WorthQueryPortableApplicationCapabilityTransitionBindingParts,
    pub approve: WorthQueryPortableApplicationCapabilityTransitionBindingParts,
    pub revoke: WorthQueryPortableApplicationCapabilityTransitionBindingParts,
    pub complete_review: WorthQueryPortableApplicationCapabilityTransitionBindingParts,
}

impl ApplicationCapabilityElevationLifecycleDefinition {
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableApplicationCapabilityElevationLifecycleParts,
    ) -> Self {
        Self {
            elevation_slot: parts.elevation_slot,
            review_slot: parts.review_slot,
            request: ApplicationCapabilityTransitionBinding::from_untrusted_parts(
                ARCHIVE_DECODE,
                parts.request,
            ),
            approve: ApplicationCapabilityTransitionBinding::from_untrusted_parts(
                ARCHIVE_DECODE,
                parts.approve,
            ),
            revoke: ApplicationCapabilityTransitionBinding::from_untrusted_parts(
                ARCHIVE_DECODE,
                parts.revoke,
            ),
            complete_review: ApplicationCapabilityTransitionBinding::from_untrusted_parts(
                ARCHIVE_DECODE,
                parts.complete_review,
            ),
        }
    }

    pub fn parts(&self) -> WorthQueryPortableApplicationCapabilityElevationLifecycleParts {
        WorthQueryPortableApplicationCapabilityElevationLifecycleParts {
            elevation_slot: self.elevation_slot.clone(),
            review_slot: self.review_slot.clone(),
            request: self.request.parts(),
            approve: self.approve.parts(),
            revoke: self.revoke.parts(),
            complete_review: self.complete_review.parts(),
        }
    }
}

impl ApplicationCapabilityTransitionBinding {
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableApplicationCapabilityTransitionBindingParts,
    ) -> Self {
        Self {
            capability: parts.capability,
            capability_type: parts.capability_type,
            operation: parts.operation,
            lifecycle_effect: parts.lifecycle_effect.map(|parts| {
                ApplicationCapabilityLifecycleEffectBinding::from_untrusted_parts(
                    ARCHIVE_DECODE,
                    parts,
                )
            }),
        }
    }

    pub fn parts(&self) -> WorthQueryPortableApplicationCapabilityTransitionBindingParts {
        WorthQueryPortableApplicationCapabilityTransitionBindingParts {
            capability: self.capability.clone(),
            capability_type: self.capability_type.clone(),
            operation: self.operation.clone(),
            lifecycle_effect: self
                .lifecycle_effect
                .as_ref()
                .map(ApplicationCapabilityLifecycleEffectBinding::parts),
        }
    }
}
