use super::{
    ApplicationCapabilityDelegationActivationDefinition, ApplicationCapabilityRevocationDefinition,
};
use crate::application_capability::{
    ApplicationCapabilityFieldBinding, ApplicationCapabilityOperationBinding,
    ApplicationCapabilityRelationBinding, ApplicationCapabilityValueBinding,
};
use crate::archive_decode_authority::ArchiveDecodeAuthority;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationCapabilityDelegationActivationParts {
    pub operation: ApplicationCapabilityOperationBinding,
    pub identity: ApplicationCapabilityFieldBinding,
    pub context_relations: Vec<ApplicationCapabilityRelationBinding>,
}

impl ApplicationCapabilityDelegationActivationDefinition {
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableApplicationCapabilityDelegationActivationParts,
    ) -> Self {
        Self {
            operation: parts.operation,
            identity: parts.identity,
            context_relations: parts.context_relations,
        }
    }

    pub fn parts(&self) -> WorthQueryPortableApplicationCapabilityDelegationActivationParts {
        WorthQueryPortableApplicationCapabilityDelegationActivationParts {
            operation: self.operation.clone(),
            identity: self.identity.clone(),
            context_relations: self.context_relations.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationCapabilityRevocationParts {
    pub operation: ApplicationCapabilityOperationBinding,
    pub identity: ApplicationCapabilityFieldBinding,
    pub revoked_status: ApplicationCapabilityValueBinding,
}

impl ApplicationCapabilityRevocationDefinition {
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableApplicationCapabilityRevocationParts,
    ) -> Self {
        Self {
            operation: parts.operation,
            identity: parts.identity,
            revoked_status: parts.revoked_status,
        }
    }

    pub fn parts(&self) -> WorthQueryPortableApplicationCapabilityRevocationParts {
        WorthQueryPortableApplicationCapabilityRevocationParts {
            operation: self.operation.clone(),
            identity: self.identity.clone(),
            revoked_status: self.revoked_status.clone(),
        }
    }
}
