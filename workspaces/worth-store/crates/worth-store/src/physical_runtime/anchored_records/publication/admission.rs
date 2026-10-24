//! Admission of a requested member set, before any read or effect: bounds,
//! keys, and per-member size; then, against the current value, carried
//! membership and the content identity a retry is compared by.

use sha2::{Digest, Sha256};

use super::super::membership::ValueMembership;
use super::super::{
    AnchoredRecordMember, AnchoredRecordName, AnchoredRecordPublicationFailure,
    MAXIMUM_ANCHORED_RECORD_MEMBERS,
};
use super::batch_capacity::BatchCapacity;

type Failure = AnchoredRecordPublicationFailure;

/// A member set whose new members hash to their keys and each fit one batch.
pub(super) struct AdmittedMembers {
    members: Vec<AnchoredRecordMember>,
}

impl AdmittedMembers {
    /// Cost: O(new member bytes) to verify keys.
    pub(super) fn admit(
        members: Vec<AnchoredRecordMember>,
        capacity: BatchCapacity,
    ) -> Result<Self, Failure> {
        if members.len() > MAXIMUM_ANCHORED_RECORD_MEMBERS {
            return Err(Failure::TooManyMembers {
                members: members.len(),
            });
        }
        for (index, member) in members.iter().enumerate() {
            let AnchoredRecordMember::New { key, bytes } = member else {
                continue;
            };
            if bytes.len() as u64 > capacity.bytes {
                return Err(Failure::MemberTooLarge { index });
            }
            if <[u8; 32]>::from(Sha256::digest(bytes)) != *key {
                return Err(Failure::MemberKeyMismatch { index });
            }
        }
        Ok(Self { members })
    }

    pub(super) fn len(&self) -> usize {
        self.members.len()
    }

    pub(super) fn carries_any(&self) -> bool {
        self.members
            .iter()
            .any(|member| matches!(member, AnchoredRecordMember::Carried { .. }))
    }

    /// Each member's content key and length, in order.
    pub(super) fn contents(&self) -> impl ExactSizeIterator<Item = ([u8; 32], u64)> + '_ {
        self.members.iter().map(AnchoredRecordMember::content)
    }

    /// Refuses the first carried member the current value does not list.
    /// With no current value, no member can be carried.
    pub(super) fn require_carried_current(
        &self,
        current: Option<&ValueMembership>,
    ) -> Result<(), Failure> {
        for (index, member) in self.members.iter().enumerate() {
            let AnchoredRecordMember::Carried { locator } = member else {
                continue;
            };
            if !current.is_some_and(|value| value.lists(locator)) {
                return Err(Failure::CarriedMemberNotCurrent { index });
            }
        }
        Ok(())
    }

    /// The identity of the requested value at `generation`: payload digest
    /// and every member's kind, key and length, in order. Two requests with
    /// equal digests write equal member batches.
    pub(super) fn content_digest(
        &self,
        name: AnchoredRecordName,
        generation: u64,
        payload_sha256: [u8; 32],
    ) -> [u8; 32] {
        let mut sha = Sha256::new();
        sha.update(b"worth.store.anchored-record.content.v1");
        sha.update([name.as_str().len() as u8]);
        sha.update(name.as_str().as_bytes());
        sha.update(generation.to_le_bytes());
        sha.update(payload_sha256);
        sha.update((self.members.len() as u64).to_le_bytes());
        for member in &self.members {
            let (key, length) = member.content();
            let carried = matches!(member, AnchoredRecordMember::Carried { .. });
            sha.update([u8::from(carried)]);
            sha.update(key);
            sha.update(length.to_le_bytes());
        }
        sha.finalize().into()
    }

    pub(super) fn into_members(self) -> Vec<AnchoredRecordMember> {
        self.members
    }
}
