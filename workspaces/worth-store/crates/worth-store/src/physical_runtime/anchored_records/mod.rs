//! Anchored records: named "current value" records that a Store home replaces
//! whole, one generation at a time, each value with an optional set of
//! content-addressed member records.
//!
//! Each name is a root anchor selecting exactly one record. A publication
//! writes one framed record and advances the anchor from the generation the
//! caller observed to the next one in the same root generation, so the value
//! is current exactly when that root is durably selected. Store treats the
//! payload and member bytes as opaque owner bytes; the frame binds the payload
//! and the member table to the anchor name and generation.
//!
//! Members. A value lists its members' locators in its frame. New members are
//! written as ordinary unanchored records in root generations before the
//! anchored record's, in as many batches as the batch capacity needs; carried
//! members are members of the current value and are not rewritten. The
//! anchor advance is the single commit point: a member batch is acknowledged
//! only once its root generation is durably selected, so every new member is
//! durable before the anchored record's batch starts, and until that batch's
//! root is selected no anchor lists the new members. A crash therefore leaves
//! the old value with its members, or the new value with its members.
//!
//! Residue and reachability. New member records of a publication that never
//! committed, and members a replacement dropped, stay routed and
//! unreferenced: Store never reads them as current, because a member is read
//! or carried only through the table of the value its anchor selects. Before
//! Milestone 4 reclamation nothing removes such a record: manifest routing
//! carries every published record into every successor root; maintenance
//! rewrites and displaced-artifact retirement move placements without
//! changing record identities; the only record removal, blob reclamation
//! (which recovery may complete), names blob records only; and recovery rolls
//! back only an unselected pending append. A member of a current value was
//! selected in a generation before its anchor's, so it stays reachable.
//!
//! The canonical commit journal is also anchored, but it keeps its own door:
//! its name is reserved and cannot be named here.

use std::collections::BTreeSet;

use worth_store_physical_format::RecordAnchorName;

use crate::physical_runtime::{ServingPhysicalRuntime, RELATIONAL_CANONICAL_COMMITS};

mod failure;
mod frame;
mod member;
mod member_read;
mod membership;
mod publication;
mod read;
mod value;
mod verified_value;

pub use failure::{
    AnchoredRecordDamage, AnchoredRecordMemberDamage, AnchoredRecordMemberReadFailure,
    AnchoredRecordMemberWriteFailure, AnchoredRecordPublicationFailure, AnchoredRecordReadFailure,
    AnchoredRecordsDenial,
};
pub use member::{
    AnchoredRecordMember, AnchoredRecordMemberLocator, ANCHORED_RECORD_MEMBER_LOCATOR_BYTES,
    MAXIMUM_ANCHORED_RECORD_MEMBERS,
};
pub(in crate::physical_runtime) use membership::AnchoredMembershipCache;
pub use publication::{AnchoredRecordPublication, AnchoredRecordPublished};
pub use value::{AnchoredRecord, AnchoredRecordMembership, AnchoredRecordVersion};

/// The name of one anchored record: a valid anchor name that no other Store
/// door owns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnchoredRecordName(RecordAnchorName);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AnchoredRecordNameDenial {
    /// Not 1 to 32 bytes of `[a-z0-9.-]`.
    Malformed,
    /// The name belongs to the canonical commit journal.
    ReservedForCanonicalJournal,
}

impl AnchoredRecordName {
    pub fn new(name: &str) -> Result<Self, AnchoredRecordNameDenial> {
        let anchor = RecordAnchorName::new(name).ok_or(AnchoredRecordNameDenial::Malformed)?;
        if anchor.as_str() == RELATIONAL_CANONICAL_COMMITS {
            return Err(AnchoredRecordNameDenial::ReservedForCanonicalJournal);
        }
        Ok(Self(anchor))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub(in crate::physical_runtime) const fn anchor(self) -> RecordAnchorName {
        self.0
    }
}

/// Serializes anchored-record publications and remembers, per name, those
/// whose fate is indeterminate until the runtime is reopened. Other names
/// stay publishable: an unresolved value never orders another name's value.
#[derive(Debug, Default)]
pub(in crate::physical_runtime) struct AnchoredRecordState {
    unresolved: BTreeSet<RecordAnchorName>,
}

/// Borrowed access to the home's anchored records.
pub struct AnchoredRecords<'runtime> {
    runtime: &'runtime ServingPhysicalRuntime,
}

impl<'runtime> AnchoredRecords<'runtime> {
    pub(in crate::physical_runtime) const fn new(
        runtime: &'runtime ServingPhysicalRuntime,
    ) -> Self {
        Self { runtime }
    }
}
