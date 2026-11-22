//! Internal runtime-crossing admission authority for Relational-minted
//! persistent name digests.
//!
//! Not re-exported through [`crate::facade`] or `worth-query-decl`; ordinary
//! declaration consumers cannot construct a [`WorthQueryName`] from raw bytes.

use crate::application_naming::WorthQueryName;

/// Admits a fixed 32-byte digest minted by WORTH Relational into a kind-typed
/// [`WorthQueryName`].
pub const fn admit_minted_digest<K>(digest: [u8; 32]) -> WorthQueryName<K> {
    WorthQueryName::from_admitted_digest(digest)
}
