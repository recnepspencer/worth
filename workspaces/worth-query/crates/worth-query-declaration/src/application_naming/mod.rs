//! Persistent naming declarations: opaque kind-typed names, canonical
//! discriminators, declared name schemes, and the resolution vocabulary a
//! held name resolves through.
//!
//! Public WORTH owns name minting and lineage authority. Applications declare
//! naming policy here through [`ApplicationNameSchemeBuilder`] and
//! [`ApplicationNameDiscriminator`]; handlers and callers can never forge or
//! mutate a [`WorthQueryName`].

mod candidate;
mod denial;
mod discriminator;
mod erased;
mod group;
mod kind;
mod name;
mod name_ref;
mod named_target;
mod occurrence_path;
mod owner_scope;
mod resolution;
mod role;
mod same_commit;
mod scheme;
mod selection;
pub(crate) mod stored_name;
mod stored_value;

#[cfg(test)]
mod group_tests;
#[cfg(test)]
mod name_ref_tests;
#[cfg(test)]
mod occurrence_path_tests;
#[cfg(test)]
mod resolution_tests;
#[cfg(test)]
mod role_tests;
#[cfg(test)]
mod same_commit_tests;
#[cfg(test)]
mod stored_name_tests;
#[cfg(test)]
mod tests;

pub use candidate::NameCandidate;
pub use denial::{
    validate_discriminator_identity, validate_role_name, validate_scheme_identifier,
    ApplicationNameSchemeDenial,
};
pub use discriminator::{
    ApplicationNameDiscriminator, ApplicationNameDiscriminatorEncoder,
    ApplicationNameDiscriminatorField, ApplicationNameDiscriminatorValue, NestedDiscriminator,
    NAME_DISCRIMINATOR_DOMAIN_TAG,
};
pub use erased::ErasedWorthQueryName;
pub(crate) use group::validate_erased_group_scheme_identifiers;
pub use group::{
    ApplicationGroupContinuityRule, ApplicationGroupMemberRole, ApplicationGroupNameScheme,
    ApplicationGroupNameSchemeBuilder, ApplicationGroupSchemeDenial,
    ErasedApplicationGroupNameScheme,
};
pub use kind::{EntityNameSubject, NameSubjectRole, RelationNameSubject, WorthQueryNamedKind};
pub use name::WorthQueryName;
pub use name_ref::NameRef;
pub use named_target::NamedTarget;
pub use occurrence_path::{OccurrencePath, OccurrencePathDenial, OccurrenceSegment};
pub use owner_scope::{
    NameOwnerHop, NameOwnerHopDirection, NameOwnerPath, NameOwnerScope, NameOwnerStep, NoNameOwner,
    OwnedThrough, OwnerVia, OwnerViaReverse, Unowned,
};
pub use resolution::{NameResolutionDenial, ResolvedName};
pub use role::{
    IndexedRoleAlone, IndexedRoleDiscriminator, WorthQueryNameRole, NAME_ROLE_INDEX_IDENTITY,
};
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
pub use stored_name::StoredNameDenial;
pub use stored_value::{NamedTargetDelegate, WorthQueryNameDelegate};
