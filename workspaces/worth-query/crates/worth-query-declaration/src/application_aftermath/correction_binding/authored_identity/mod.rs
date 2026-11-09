//! Family-declared logical identity for authored lifecycle inverses.
mod authoring;
mod key;
mod owned_members;
pub use authoring::ApplicationAuthoredCorrection;
pub use key::ApplicationAuthoredLogicalKey;

use crate::application_operation::ApplicationMutationOutputPosture;
use crate::application_schema::{
    ApplicationEntityMarkerIdentity, ApplicationSchema, ApplicationSchemaMember,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;

/// One authored entity family declares the key preserved when deletion is corrected.
pub trait ApplicationAuthoredCorrectionFamily<Schema: ApplicationSchema>: 'static {
    const IDENTITY: &'static str;
    type Entity: ApplicationEntityMarkerIdentity<Schema>;
    fn logical_key() -> ApplicationAuthoredLogicalKey<Schema, Self::Entity>;
}

/// Descriptive family and role meaning; installation resolves every part.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DeclaredAuthoredCorrection {
    parts: DeclaredAuthoredCorrectionParts,
    co_anchors: Vec<DeclaredAuthoredCorrection>,
    optional: bool,
    owner: Option<crate::application_aftermath::PortablePreImageRelationLocus>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DeclaredAuthoredCorrectionParts {
    pub family: WorthQueryPortableTypeIdentity,
    pub logical_key: Box<ApplicationSchemaMember>,
    pub binding_unit: Option<String>,
    pub source_role: String,
    pub inverse_role: String,
    pub source_posture: ApplicationMutationOutputPosture,
}

impl DeclaredAuthoredCorrection {
    pub fn from_untrusted_parts(parts: DeclaredAuthoredCorrectionParts) -> Self {
        Self {
            parts,
            co_anchors: Vec::new(),
            optional: false,
            owner: None,
        }
    }
    pub fn co_anchors(&self) -> &[Self] {
        &self.co_anchors
    }
    pub fn is_optional(&self) -> bool {
        self.optional
    }
    /// A member family uses role prefixes; its live roster is the sources of
    /// this exactly-one relation into the enclosing authored anchor.
    pub fn member_owner(
        &self,
    ) -> Option<&crate::application_aftermath::PortablePreImageRelationLocus> {
        self.owner.as_ref()
    }
    /// Match a member output against its complete declared family descriptor.
    pub fn member_role_suffix<'a>(
        &self,
        name: &'a str,
        posture: ApplicationMutationOutputPosture,
        entity: &str,
        inverse: bool,
    ) -> Option<&'a str> {
        self.member_owner()?;
        let ApplicationSchemaMember::Field {
            entity: declared_entity,
            ..
        } = self.parts.logical_key.as_ref()
        else {
            return None;
        };
        let expected = if inverse {
            match self.parts.source_posture {
                ApplicationMutationOutputPosture::Create => {
                    ApplicationMutationOutputPosture::Retire
                }
                ApplicationMutationOutputPosture::Retire => {
                    ApplicationMutationOutputPosture::Create
                }
                ApplicationMutationOutputPosture::Preserve => return None,
            }
        } else {
            self.parts.source_posture
        };
        if declared_entity != entity || posture != expected {
            return None;
        }
        name.strip_prefix(if inverse {
            &self.parts.inverse_role
        } else {
            &self.parts.source_role
        })
        .filter(|suffix| !suffix.is_empty())
    }
    pub fn with_member_owner(
        mut self,
        owner: crate::application_aftermath::PortablePreImageRelationLocus,
    ) -> Self {
        self.owner = Some(owner);
        self
    }
    pub fn with_co_anchor(mut self, mut anchor: Self, optional: bool) -> Self {
        anchor.optional = optional;
        self.co_anchors.push(anchor);
        self.co_anchors.sort_by(|a, b| {
            a.parts
                .source_role
                .cmp(&b.parts.source_role)
                .then_with(|| a.parts.inverse_role.cmp(&b.parts.inverse_role))
        });
        self
    }
    pub fn parts(&self) -> &DeclaredAuthoredCorrectionParts {
        &self.parts
    }
    pub fn same_family_as(&self, other: &Self) -> bool {
        self.parts.family == other.parts.family
            && self.parts.logical_key == other.parts.logical_key
            && self.parts.binding_unit == other.parts.binding_unit
    }
}
