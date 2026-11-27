//! Lookup of a name the index recorded from a committed birth, by its raw
//! 32-byte digest, and of the canonical parts that birth carried.

use crate::contracts::lineage::data::naming::{
    PersistentNameBirthParts as Parts, PersistentNameIdentity,
    PersistentNameIdentityOwnerPort as _, PersistentNameSubjectKind,
};
use worth_foundational::facade::AggregateIdentity;

use crate::contracts::lineage::data::name_resolution::PersistentNameKey;

use super::PersistentNameBranchIndex;
use crate::lineage::tiered::NamingReadScope;

impl PersistentNameBranchIndex {
    /// The key this index recorded for `digest` under `subject`, when it
    /// ingested that name's birth.
    ///
    /// The probe built from `digest` only addresses the map and never leaves
    /// this function; callers receive the key the index itself recorded from
    /// a canonical committed birth, so bare bytes open nothing.
    ///
    /// Complexity: O(1) expected.
    pub(crate) fn recorded_name(
        &self,
        subject: PersistentNameSubjectKind,
        digest: &[u8; PersistentNameIdentity::BYTE_LEN],
        scope: &NamingReadScope,
    ) -> Option<PersistentNameKey> {
        let probe = match subject {
            PersistentNameSubjectKind::Entity => {
                PersistentNameKey::Entity(PersistentNameIdentity::from_minted_digest(*digest))
            }
            PersistentNameSubjectKind::Relation => {
                PersistentNameKey::Relation(PersistentNameIdentity::from_minted_digest(*digest))
            }
            PersistentNameSubjectKind::Group => {
                PersistentNameKey::Group(AggregateIdentity::from_bytes(*digest))
            }
        };
        self.entry_in(&probe, scope).map(|_| probe)
    }

    /// The persisted birth parts the index ingested for `name`, if any.
    pub fn recorded_birth<'s>(
        &'s self,
        name: PersistentNameKey,
        scope: &'s NamingReadScope,
    ) -> Option<&'s Parts> {
        self.entry_in(&name, scope)?.birth()
    }
}
