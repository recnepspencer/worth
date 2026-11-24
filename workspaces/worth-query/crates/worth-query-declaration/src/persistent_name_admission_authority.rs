//! Internal runtime-crossing admission authority for Relational-minted
//! persistent name digests.
//!
//! Not re-exported through [`crate::facade`] or `worth-query-decl`; ordinary
//! declaration consumers cannot construct a [`WorthQueryName`] from raw bytes.

use std::sync::Arc;

use crate::application_naming::{
    SameCommitNameReference, WorthQueryName, WorthQuerySameCommitName,
};

/// Issues the kind-typed reference to the name the `output`-th named output
/// of `candidate` will be born with. Only the candidate's own writer, through
/// the naming admission boundary, may issue one for an output it named.
pub fn issue_same_commit_name<K>(
    candidate: &Arc<()>,
    output: usize,
) -> WorthQuerySameCommitName<K> {
    WorthQuerySameCommitName::issued(SameCommitNameReference::issued(
        Arc::clone(candidate),
        output,
    ))
}

/// Admits a fixed 32-byte digest minted by WORTH Relational into a kind-typed
/// [`WorthQueryName`].
pub const fn admit_minted_digest<K>(digest: [u8; 32]) -> WorthQueryName<K> {
    WorthQueryName::from_admitted_digest(digest)
}
