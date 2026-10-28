//! Finite read-only member meaning; a domain is never a writable entity kind.
mod authoring;
mod definition;
mod reference;
mod validation;

pub use definition::{
    ApplicationProjectedMemberDomainDefinition, ApplicationProjectedMemberDomainDefinitionDenial,
    ErasedApplicationProjectedMemberDomainDefinition, WorthQueryPortableProjectedMemberDomainParts,
};
pub use reference::{ApplicationProjectedMemberCaseRef, ApplicationProjectedMemberDomainRef};
pub(super) use validation::{validate_closure, validate_member_identifiers};
