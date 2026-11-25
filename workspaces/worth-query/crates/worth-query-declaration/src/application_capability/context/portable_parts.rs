use crate::application_schema::ApplicationAuthorizationTraversalDirection;
use crate::archive_decode_authority::ArchiveDecodeAuthority;
use crate::portable_identity::WorthQueryPortableTypeIdentity;

use super::{
    ApplicationCapabilityContextEntitySlotBinding, ApplicationCapabilityPathContextAnchor,
};
use crate::application_capability::ApplicationCapabilityRelationBinding;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationCapabilityContextEntitySlotBindingParts {
    pub context: String,
    pub context_identity: WorthQueryPortableTypeIdentity,
    pub slot: String,
    pub slot_identity: WorthQueryPortableTypeIdentity,
    pub entity: String,
}

impl ApplicationCapabilityContextEntitySlotBinding {
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableApplicationCapabilityContextEntitySlotBindingParts,
    ) -> Self {
        Self {
            context: parts.context,
            context_identity: parts.context_identity,
            slot: parts.slot,
            slot_identity: parts.slot_identity,
            entity: parts.entity,
        }
    }

    pub fn parts(&self) -> WorthQueryPortableApplicationCapabilityContextEntitySlotBindingParts {
        WorthQueryPortableApplicationCapabilityContextEntitySlotBindingParts {
            context: self.context.clone(),
            context_identity: self.context_identity.clone(),
            slot: self.slot.clone(),
            slot_identity: self.slot_identity.clone(),
            entity: self.entity.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationCapabilityPathContextAnchorParts {
    pub relation: ApplicationCapabilityRelationBinding,
    pub direction: ApplicationAuthorizationTraversalDirection,
    pub slot: ApplicationCapabilityContextEntitySlotBinding,
}

impl ApplicationCapabilityPathContextAnchor {
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableApplicationCapabilityPathContextAnchorParts,
    ) -> Self {
        Self {
            relation: parts.relation,
            direction: parts.direction,
            slot: parts.slot,
        }
    }

    pub fn parts(&self) -> WorthQueryPortableApplicationCapabilityPathContextAnchorParts {
        WorthQueryPortableApplicationCapabilityPathContextAnchorParts {
            relation: self.relation.clone(),
            direction: self.direction,
            slot: self.slot.clone(),
        }
    }
}
