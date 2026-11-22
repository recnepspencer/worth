//! Persistent naming declarations: opaque kind-typed names, canonical
//! discriminators, and declared name schemes.
//!
//! Public WORTH owns name minting and lineage authority. Applications declare
//! naming policy here through [`ApplicationNameSchemeBuilder`] and
//! [`ApplicationNameDiscriminator`]; handlers and callers can never forge or
//! mutate a [`WorthQueryName`].

mod denial;
mod discriminator;
mod name;
mod scheme;

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
pub use name::WorthQueryName;
pub(crate) use scheme::validate_erased_scheme_identifiers;
pub use scheme::{
    ApplicationNameRoleDependency, ApplicationNameScheme, ApplicationNameSchemeBuilder,
    ApplicationNameSubjectKind, ErasedApplicationNameScheme,
};
