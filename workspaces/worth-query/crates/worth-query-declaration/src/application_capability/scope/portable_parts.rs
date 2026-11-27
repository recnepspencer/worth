use crate::archive_decode_authority::ArchiveDecodeAuthority;
use crate::portable_identity::WorthQueryPortableTypeIdentity;
use worth_foundational::facade::{AspectValue, ScalarAspectType};

use super::{
    ApplicationCapabilityConstraintDefinition, ApplicationCapabilityDelegationDefinition,
    ApplicationCapabilityFieldBinding, ApplicationCapabilityRelationBinding,
    ApplicationCapabilityValueBinding,
};
use super::{ApplicationCapabilityDelegationLineage, ApplicationCapabilitySelfGrantRule};
use crate::application_capability::{
    ApplicationCapabilityCardinalityDimension, ApplicationCapabilityCurrentnessDefinition,
    ApplicationCapabilityDelegationActivationDefinition, ApplicationCapabilityFieldDimension,
    ApplicationCapabilityRevocationDefinition,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationCapabilityFieldBindingParts {
    pub entity: String,
    pub aspect: String,
    pub field: String,
    pub scalar_family: ScalarAspectType,
    pub value_type: String,
}

impl ApplicationCapabilityFieldBinding {
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableApplicationCapabilityFieldBindingParts,
    ) -> Self {
        Self {
            entity: parts.entity,
            aspect: parts.aspect,
            field: parts.field,
            scalar_family: parts.scalar_family,
            value_type: parts.value_type,
        }
    }

    pub fn parts(&self) -> WorthQueryPortableApplicationCapabilityFieldBindingParts {
        WorthQueryPortableApplicationCapabilityFieldBindingParts {
            entity: self.entity.clone(),
            aspect: self.aspect.clone(),
            field: self.field.clone(),
            scalar_family: self.scalar_family,
            value_type: self.value_type.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationCapabilityValueBindingParts {
    pub field: ApplicationCapabilityFieldBinding,
    pub value: AspectValue,
}

impl ApplicationCapabilityValueBinding {
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableApplicationCapabilityValueBindingParts,
    ) -> Self {
        Self {
            field: parts.field,
            value: parts.value,
        }
    }

    pub fn parts(&self) -> WorthQueryPortableApplicationCapabilityValueBindingParts {
        WorthQueryPortableApplicationCapabilityValueBindingParts {
            field: self.field.clone(),
            value: self.value.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationCapabilityRelationBindingParts {
    pub relation: String,
    pub from: String,
    pub to: String,
}

impl ApplicationCapabilityRelationBinding {
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableApplicationCapabilityRelationBindingParts,
    ) -> Self {
        Self {
            relation: parts.relation,
            from: parts.from,
            to: parts.to,
        }
    }

    pub fn parts(&self) -> WorthQueryPortableApplicationCapabilityRelationBindingParts {
        WorthQueryPortableApplicationCapabilityRelationBindingParts {
            relation: self.relation.clone(),
            from: self.from.clone(),
            to: self.to.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationCapabilityConstraintParts {
    pub magnitude: ApplicationCapabilityFieldDimension,
    pub cardinality: ApplicationCapabilityCardinalityDimension,
    pub currentness: ApplicationCapabilityCurrentnessDefinition,
    pub context: String,
    pub context_type: WorthQueryPortableTypeIdentity,
}

impl ApplicationCapabilityConstraintDefinition {
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableApplicationCapabilityConstraintParts,
    ) -> Self {
        Self {
            magnitude: parts.magnitude,
            cardinality: parts.cardinality,
            currentness: parts.currentness,
            context: parts.context,
            context_type: parts.context_type,
        }
    }

    pub fn parts(&self) -> WorthQueryPortableApplicationCapabilityConstraintParts {
        WorthQueryPortableApplicationCapabilityConstraintParts {
            magnitude: self.magnitude.clone(),
            cardinality: self.cardinality,
            currentness: self.currentness.clone(),
            context: self.context.clone(),
            context_type: self.context_type.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationCapabilityDelegationParts {
    pub lineage: Option<ApplicationCapabilityDelegationLineage>,
    pub grantee: ApplicationCapabilityRelationBinding,
    pub provenance: String,
    pub provenance_type: WorthQueryPortableTypeIdentity,
    pub activation: Option<ApplicationCapabilityDelegationActivationDefinition>,
    pub revocation: Option<ApplicationCapabilityRevocationDefinition>,
    pub self_grant: ApplicationCapabilitySelfGrantRule,
}

impl ApplicationCapabilityDelegationDefinition {
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableApplicationCapabilityDelegationParts,
    ) -> Self {
        Self {
            lineage: parts.lineage,
            grantee: parts.grantee,
            provenance: parts.provenance,
            provenance_type: parts.provenance_type,
            activation: parts.activation,
            revocation: parts.revocation,
            self_grant: parts.self_grant,
        }
    }

    pub fn parts(&self) -> WorthQueryPortableApplicationCapabilityDelegationParts {
        WorthQueryPortableApplicationCapabilityDelegationParts {
            lineage: self.lineage.clone(),
            grantee: self.grantee.clone(),
            provenance: self.provenance.clone(),
            provenance_type: self.provenance_type.clone(),
            activation: self.activation.clone(),
            revocation: self.revocation.clone(),
            self_grant: self.self_grant,
        }
    }
}
