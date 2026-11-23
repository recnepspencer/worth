//! Persistent naming declarations: opaque kind-typed names, canonical
//! discriminators, declared name schemes, and the resolution vocabulary a
//! held name resolves through.
//!
//! Public WORTH owns name minting and lineage authority. Applications declare
//! naming policy here through [`ApplicationNameSchemeBuilder`] and
//! [`ApplicationNameDiscriminator`]; handlers and callers can never forge or
//! mutate a [`WorthQueryName`].

mod denial;
mod discriminator;
mod erased;
mod group;
mod name;
mod resolution;
mod same_commit;
mod scheme;
mod selection;

#[cfg(test)]
mod group_tests;
#[cfg(test)]
mod resolution_tests;
#[cfg(test)]
mod same_commit_tests;
#[cfg(test)]
mod tests;

pub use denial::{
    validate_discriminator_identity, validate_role_name, validate_scheme_identifier,
    ApplicationNameSchemeDenial,
};
pub use discriminator::{
    ApplicationNameDiscriminator, ApplicationNameDiscriminatorEncoder,
    ApplicationNameDiscriminatorValue, NestedDiscriminator, NAME_DISCRIMINATOR_DOMAIN_TAG,
};
pub use erased::ErasedWorthQueryName;
pub(crate) use group::validate_erased_group_scheme_identifiers;
pub use group::{
    ApplicationGroupContinuityRule, ApplicationGroupMemberRole, ApplicationGroupNameScheme,
    ApplicationGroupNameSchemeBuilder, ApplicationGroupSchemeDenial,
    ErasedApplicationGroupNameScheme,
};
pub use name::WorthQueryName;
pub use resolution::{NameResolutionDenial, ResolvedName};
pub use same_commit::{
    SameCommitNameReference, SameCommitNameSlot, WorthQuerySameCommitName,
    SAME_COMMIT_NAME_PLACEHOLDER,
};
pub(crate) use scheme::validate_erased_scheme_identifiers;
pub use scheme::{
    ApplicationNameRoleDependency, ApplicationNameScheme, ApplicationNameSchemeBuilder,
    ApplicationNameSubjectKind, ErasedApplicationNameScheme,
};
pub use selection::{NameCarrierSet, NameSelection, NameSelectionDenial, NameSideDiscriminator};
