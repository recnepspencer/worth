use crate::archive_decode_authority::{ArchiveDecodeAuthority, ARCHIVE_DECODE};
use crate::portable_identity::WorthQueryPortableTypeIdentity;

use super::ErasedApplicationCapabilityContract;
use crate::application_capability::{
    ApplicationCapabilityComposition, ApplicationCapabilityConstraintDefinition,
    ApplicationCapabilityDelegationDefinition, ApplicationCapabilityElevationRule,
    ApplicationCapabilityTargetDefinition,
    WorthQueryPortableApplicationCapabilityElevationRuleParts,
};

/// Owned, authority-free meaning needed to reconstruct one capability contract.
///
/// Construction deliberately performs no sorting or validation. The enclosing
/// application-schema readmission boundary must inspect these untrusted parts
/// exactly as supplied.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationCapabilityContractParts {
    pub name: String,
    pub capability_type: WorthQueryPortableTypeIdentity,
    pub operation: String,
    pub operation_type: WorthQueryPortableTypeIdentity,
    pub input_type: WorthQueryPortableTypeIdentity,
    pub grant_entity: String,
    pub target: ApplicationCapabilityTargetDefinition,
    pub constraints: ApplicationCapabilityConstraintDefinition,
    pub delegation: ApplicationCapabilityDelegationDefinition,
    pub composition: ApplicationCapabilityComposition,
    pub elevation: WorthQueryPortableApplicationCapabilityElevationRuleParts,
}

impl ErasedApplicationCapabilityContract {
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableApplicationCapabilityContractParts,
    ) -> Self {
        Self {
            name: parts.name,
            capability_type: parts.capability_type,
            operation: parts.operation,
            operation_type: parts.operation_type,
            input_type: parts.input_type,
            grant_entity: parts.grant_entity,
            target: parts.target,
            constraints: parts.constraints,
            delegation: parts.delegation,
            composition: parts.composition,
            elevation: ApplicationCapabilityElevationRule::from_untrusted_parts(
                ARCHIVE_DECODE,
                parts.elevation,
            ),
        }
    }

    pub fn parts(&self) -> WorthQueryPortableApplicationCapabilityContractParts {
        WorthQueryPortableApplicationCapabilityContractParts {
            name: self.name.clone(),
            capability_type: self.capability_type.clone(),
            operation: self.operation.clone(),
            operation_type: self.operation_type.clone(),
            input_type: self.input_type.clone(),
            grant_entity: self.grant_entity.clone(),
            target: self.target.clone(),
            constraints: self.constraints.clone(),
            delegation: self.delegation.clone(),
            composition: self.composition.clone(),
            elevation: self.elevation.parts(),
        }
    }

    pub fn into_parts(self) -> WorthQueryPortableApplicationCapabilityContractParts {
        WorthQueryPortableApplicationCapabilityContractParts {
            name: self.name,
            capability_type: self.capability_type,
            operation: self.operation,
            operation_type: self.operation_type,
            input_type: self.input_type,
            grant_entity: self.grant_entity,
            target: self.target,
            constraints: self.constraints,
            delegation: self.delegation,
            composition: self.composition,
            elevation: self.elevation.parts(),
        }
    }
}
