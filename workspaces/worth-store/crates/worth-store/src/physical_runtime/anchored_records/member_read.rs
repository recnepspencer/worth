use sha2::{Digest, Sha256};

use super::{
    AnchoredRecordMemberDamage, AnchoredRecordMemberLocator, AnchoredRecordMemberReadFailure,
    AnchoredRecordName, AnchoredRecords,
};
use crate::physical_runtime::{RecordByteLimit, RecordReadLimits, RecordReadSession};

type Failure = AnchoredRecordMemberReadFailure;

impl AnchoredRecords<'_> {
    /// The bytes of one member of the current value of `name`.
    ///
    /// The locator must be listed by the value the anchor selects in this
    /// read's protected root; a locator of a replaced value, a value never
    /// published or any other record refuses as `NotCurrentMember`. A member
    /// longer than `maximum_bytes` refuses before it is read. The bytes are
    /// verified against the locator's length and SHA-256; damage refuses.
    ///
    /// Cost: O(log members) for the membership check once the current value
    /// is verified (O(value bytes) the first time), plus O(member bytes).
    pub fn read_member(
        &self,
        name: AnchoredRecordName,
        locator: AnchoredRecordMemberLocator,
        maximum_bytes: u64,
    ) -> Result<Vec<u8>, Failure> {
        let reader = self.reader().map_err(Failure::Value)?;
        let Some(anchor) = self.current_anchor(&reader, name) else {
            return Err(Failure::NotCurrentMember);
        };
        let membership = self
            .membership(&reader, name, anchor)
            .map_err(Failure::Value)?;
        if !membership.lists(&locator) {
            return Err(Failure::NotCurrentMember);
        }
        let length = usize::try_from(locator.length())
            .ok()
            .filter(|_| locator.length() <= maximum_bytes)
            .ok_or(Failure::LimitExceeded {
                length: locator.length(),
            })?;
        let limit = RecordReadLimits::new(RecordByteLimit::new(u32::MAX).expect("nonzero limit"));
        let mut session = reader
            .open(locator.record(), limit)
            .map_err(Failure::Open)?;
        let mut bytes = vec![0; length];
        if fill(&mut session, &mut bytes)? < length {
            return Err(Failure::Damaged(AnchoredRecordMemberDamage::Truncated));
        }
        if fill(&mut session, &mut [0; 1])? != 0 {
            return Err(Failure::Damaged(AnchoredRecordMemberDamage::TrailingBytes));
        }
        if <[u8; 32]>::from(Sha256::digest(&bytes)) != locator.sha256() {
            return Err(Failure::Damaged(AnchoredRecordMemberDamage::DigestMismatch));
        }
        Ok(bytes)
    }
}

/// Reads until `buffer` is full or the record ends; returns the bytes read.
fn fill(session: &mut RecordReadSession, buffer: &mut [u8]) -> Result<usize, Failure> {
    let mut filled = 0;
    while filled < buffer.len() {
        let read = session
            .read_next(&mut buffer[filled..])
            .map_err(Failure::Stream)?;
        if read == 0 {
            break;
        }
        filled += read;
    }
    Ok(filled)
}
