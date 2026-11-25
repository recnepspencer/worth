//! Internal runtime-crossing admission authority for Relational-minted
//! persistent name digests.
//!
//! Not re-exported through [`crate::facade`] or `worth-query-decl`; ordinary
//! declaration consumers cannot construct a [`WorthQueryName`] from raw bytes.

use std::collections::BTreeSet;
use std::sync::Arc;

use crate::application_naming::{
    recorded_name_references, ApplicationNameDiscriminatorDecodeDenial, RecordedNameBirth,
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

/// Every name digest a recorded canonical discriminator references, so the
/// admission boundary can admit each one at its read's basis.
///
/// Complexity: O(len).
pub fn recorded_discriminator_references(
    canonical_discriminator: &[u8],
) -> Result<Vec<[u8; 32]>, ApplicationNameDiscriminatorDecodeDenial> {
    recorded_name_references(canonical_discriminator)
}

/// Admits the birth Relational recorded for `name`, read at a basis where
/// `name`, every parent and every reference in `admitted_references` are
/// recorded and visible.
pub fn admit_recorded_birth<K>(
    name: [u8; 32],
    occurrence_key: String,
    role_path: String,
    parents: impl IntoIterator<Item = [u8; 32]>,
    canonical_discriminator: Vec<u8>,
    admitted_references: BTreeSet<[u8; 32]>,
) -> RecordedNameBirth<K> {
    RecordedNameBirth::from_admitted_parts(
        WorthQueryName::from_admitted_digest(name),
        occurrence_key,
        role_path,
        parents
            .into_iter()
            .map(WorthQueryName::from_admitted_digest)
            .collect(),
        canonical_discriminator,
        admitted_references,
    )
}
