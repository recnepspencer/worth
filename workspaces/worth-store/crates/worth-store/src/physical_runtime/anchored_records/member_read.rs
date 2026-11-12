use sha2::{Digest, Sha256};

use super::{
    AnchoredRecordMemberDamage, AnchoredRecordMemberLocator, AnchoredRecordMemberReadFailure,
    AnchoredRecordName, AnchoredRecords,
};
use crate::physical_runtime::{RecordByteLimit, RecordReadLimits, RecordReadSession};

type Failure = AnchoredRecordMemberReadFailure;

impl AnchoredRecords<'_> {
    /// Open an actual TABLE INDEX from this original verified read route.
    /// This performs O(1) table selection after nonwaiting owner/protection
    /// validation, never a raw-locator promotion or latest-anchor recapture.
    /// The returned lower-level stream requires caller-owned original request
    /// polling, bounded scratch and consumption through verified EOF.
    pub fn open_read_member(
        &self,
        read: &super::AnchoredRecordReadMembership,
        table_index: usize,
        maximum_bytes: u64,
    ) -> Result<super::AnchoredRecordMemberStream, Failure> {
        let membership = self
            .validate_read_membership(read)
            .map_err(|cause| match cause {
                super::AnchoredRecordReadFailure::ForeignReader => Failure::ForeignReader,
                cause => Failure::Value(cause),
            })?;
        let locator = membership
            .members()
            .get(table_index)
            .copied()
            .ok_or(Failure::NotCurrentMember)?;
        require_limit(locator, maximum_bytes)?;
        open_listed_member(read.original_reader(), locator)
    }

    /// Open one selected member without allocating its payload. The returned
    /// protected stream returns EOF only after checking length and SHA. This
    /// lower-level primitive retains no request or refusal latch; its owning
    /// caller polls the original request and bounds scratch before every read.
    pub fn open_member(
        &self,
        name: AnchoredRecordName,
        locator: AnchoredRecordMemberLocator,
        maximum_bytes: u64,
    ) -> Result<super::AnchoredRecordMemberStream, Failure> {
        let reader = self.reader().map_err(Failure::Value)?;
        self.open_member_from(&reader, name, locator, maximum_bytes)
    }

    pub fn open_member_from(
        &self,
        reader: &crate::physical_runtime::PhysicalRecordReader,
        name: AnchoredRecordName,
        locator: AnchoredRecordMemberLocator,
        maximum_bytes: u64,
    ) -> Result<super::AnchoredRecordMemberStream, Failure> {
        if !reader.belongs_to(self.runtime) {
            return Err(Failure::ForeignReader);
        }
        require_limit(locator, maximum_bytes)?;
        let anchor = self
            .current_anchor(reader, name)
            .ok_or(Failure::NotCurrentMember)?;
        if !self
            .membership(reader, name, anchor)
            .map_err(Failure::Value)?
            .lists(&locator)
        {
            return Err(Failure::NotCurrentMember);
        }
        open_listed_member(reader, locator)
    }

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

fn require_limit(locator: AnchoredRecordMemberLocator, maximum: u64) -> Result<(), Failure> {
    if locator.length() > maximum {
        return Err(Failure::LimitExceeded {
            length: locator.length(),
        });
    }
    Ok(())
}
fn open_listed_member(
    reader: &crate::physical_runtime::PhysicalRecordReader,
    locator: AnchoredRecordMemberLocator,
) -> Result<super::AnchoredRecordMemberStream, Failure> {
    let limit = RecordReadLimits::new(RecordByteLimit::new(u32::MAX).expect("nonzero limit"));
    let session = reader
        .open(locator.record(), limit)
        .map_err(Failure::Open)?;
    if session.observation().bytes_requested() != locator.length() {
        return Err(Failure::Damaged(AnchoredRecordMemberDamage::Truncated));
    }
    Ok(super::AnchoredRecordMemberStream::new(session, locator))
}
