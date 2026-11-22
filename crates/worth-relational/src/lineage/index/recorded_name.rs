//! Lookup of a name the index recorded from a committed birth, by its raw
//! 32-byte digest.

use worth_foundational::facade::AggregateIdentity;

use crate::lineage::data::{PersistentNameIdentity, PersistentNameKey, PersistentNameSubjectKind};

use super::PersistentNameBranchIndex;

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
        self.names.get_key_value(&probe).map(|(key, _)| *key)
    }
}
