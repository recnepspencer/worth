//! One live candidate of a held name, with the owners that hold it.

use std::fmt;

use super::{NoNameOwner, WorthQueryName};

/// A live candidate a resolution could not narrow, with every owner the
/// kind's declared owner chain reaches from it at the reader's basis.
///
/// `owners` is empty when the kind declares no owner scope, when the chain
/// reaches no live owner from this candidate, or when the reader that
/// refused reads no owners (`O` is then [`NoNameOwner`], which has no
/// values). Owners are in the order the reader read them; a decision reader
/// reports each owner once, in ascending record order.
///
/// ```
/// use worth_query_declaration::facade::application_naming::{NameCandidate, WorthQueryName};
///
/// struct Face;
///
/// fn owned_by(candidates: &[NameCandidate<Face, &str>], owner: &str) -> Vec<WorthQueryName<Face>> {
///     candidates
///         .iter()
///         .filter(|candidate| candidate.owners().contains(&owner))
///         .map(NameCandidate::name)
///         .collect()
/// }
///
/// let _check: fn(&[NameCandidate<Face, &str>], &str) -> Vec<WorthQueryName<Face>> = owned_by;
/// ```
pub struct NameCandidate<K, O = NoNameOwner> {
    name: WorthQueryName<K>,
    owners: Vec<O>,
}

impl<K, O> NameCandidate<K, O> {
    /// Pairs an admitted candidate name with the owners read for it.
    pub fn new(name: WorthQueryName<K>, owners: Vec<O>) -> Self {
        Self { name, owners }
    }

    /// The candidate's admitted persistent name.
    pub const fn name(&self) -> WorthQueryName<K> {
        self.name
    }

    /// Every owner holding this candidate at the reader's basis.
    pub fn owners(&self) -> &[O] {
        &self.owners
    }

    pub fn into_parts(self) -> (WorthQueryName<K>, Vec<O>) {
        (self.name, self.owners)
    }

    /// The same candidate with each owner mapped through `map`.
    pub fn map_owners<P>(self, map: impl FnMut(O) -> P) -> NameCandidate<K, P> {
        NameCandidate {
            name: self.name,
            owners: self.owners.into_iter().map(map).collect(),
        }
    }
}

impl<K, O: Clone> Clone for NameCandidate<K, O> {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            owners: self.owners.clone(),
        }
    }
}

impl<K, O: PartialEq> PartialEq for NameCandidate<K, O> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.owners == other.owners
    }
}

impl<K, O: Eq> Eq for NameCandidate<K, O> {}

impl<K, O: fmt::Debug> fmt::Debug for NameCandidate<K, O> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NameCandidate")
            .field("name", &self.name)
            .field("owners", &self.owners)
            .finish()
    }
}
