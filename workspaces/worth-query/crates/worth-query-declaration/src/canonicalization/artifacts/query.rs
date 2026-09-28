use crate::authoring::{QueryFamily, RootEntityKey};
use crate::binding::IdentityBindingDescriptor;
use crate::identity::{CanonicalEquivalence, CanonicalQueryDigest};
use crate::identity_authority::QueryCanonicalAuthority;

use super::entries::{
    CanonicalOrderingEntry, CanonicalPredicateEntry, CanonicalProjectionEntry,
    CanonicalTraversalEntry,
};

/// The canonical form of one authored query: its family, root entity,
/// projection, predicates, ordering, traversal, and identity bindings in
/// canonical order, with the digest and canonical identity derived from them.
///
/// You get one from a `CanonicalQueryBundle` after canonicalization. It is
/// descriptive meaning, not installation or execution authority; compare two
/// artifacts with `equivalence_to`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalQueryArtifact {
    pub(crate) digest: CanonicalQueryDigest,
    pub(crate) authority: QueryCanonicalAuthority,
    pub(crate) family: QueryFamily,
    pub(crate) root: RootEntityKey,
    pub(crate) projection: Vec<CanonicalProjectionEntry>,
    pub(crate) predicates: Vec<CanonicalPredicateEntry>,
    pub(crate) ordering: Vec<CanonicalOrderingEntry>,
    pub(crate) traversal: Vec<CanonicalTraversalEntry>,
    pub(crate) identity_bindings: Vec<IdentityBindingDescriptor>,
}

impl CanonicalQueryArtifact {
    pub fn authority(&self) -> QueryCanonicalAuthority {
        self.authority.clone()
    }

    pub fn digest(&self) -> &CanonicalQueryDigest {
        &self.digest
    }

    pub fn family(&self) -> &QueryFamily {
        &self.family
    }

    pub fn root(&self) -> &RootEntityKey {
        &self.root
    }

    pub fn projection(&self) -> &[CanonicalProjectionEntry] {
        &self.projection
    }

    pub fn predicates(&self) -> &[CanonicalPredicateEntry] {
        &self.predicates
    }

    pub fn ordering(&self) -> &[CanonicalOrderingEntry] {
        &self.ordering
    }

    pub fn traversal(&self) -> &[CanonicalTraversalEntry] {
        &self.traversal
    }

    pub fn identity_bindings(&self) -> &[IdentityBindingDescriptor] {
        &self.identity_bindings
    }

    pub fn equivalence_to(&self, other: &Self) -> CanonicalEquivalence {
        if self.family == other.family
            && self.root == other.root
            && self.projection == other.projection
            && self.predicates == other.predicates
            && self.ordering == other.ordering
            && self.traversal == other.traversal
            && self.identity_bindings == other.identity_bindings
            && self.digest == other.digest
        {
            CanonicalEquivalence::Equivalent
        } else {
            CanonicalEquivalence::Distinct
        }
    }
}
