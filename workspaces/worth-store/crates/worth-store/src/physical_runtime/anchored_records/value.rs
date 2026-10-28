//! What the door reports about a durable anchored value.

use super::{AnchoredRecordMemberLocator, AnchoredRecordName};
use crate::physical_runtime::PhysicalRecordId;

/// The identity of one durable value of an anchored record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnchoredRecordVersion {
    pub(super) name: AnchoredRecordName,
    pub(super) generation: u64,
    pub(super) record: PhysicalRecordId,
    pub(super) payload_sha256: [u8; 32],
    pub(super) payload_bytes: u64,
}

impl AnchoredRecordVersion {
    pub const fn name(&self) -> AnchoredRecordName {
        self.name
    }
    /// One for the first value; each replacement adds one.
    pub const fn generation(&self) -> u64 {
        self.generation
    }
    pub const fn record(&self) -> PhysicalRecordId {
        self.record
    }
    pub const fn payload_sha256(&self) -> [u8; 32] {
        self.payload_sha256
    }
    pub const fn payload_bytes(&self) -> u64 {
        self.payload_bytes
    }
}

/// A durable value's version and its members, in the publisher's order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchoredRecordMembership {
    pub(super) version: AnchoredRecordVersion,
    pub(super) members: Vec<AnchoredRecordMemberLocator>,
}

impl AnchoredRecordMembership {
    pub const fn version(&self) -> AnchoredRecordVersion {
        self.version
    }
    pub fn members(&self) -> &[AnchoredRecordMemberLocator] {
        &self.members
    }
    pub fn into_members(self) -> Vec<AnchoredRecordMemberLocator> {
        self.members
    }
}

/// The current value of an anchored record, verified against its anchor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchoredRecord {
    pub(super) membership: AnchoredRecordMembership,
    pub(super) bytes: Vec<u8>,
}

impl AnchoredRecord {
    pub const fn version(&self) -> AnchoredRecordVersion {
        self.membership.version
    }
    /// The value's members, in the publisher's order.
    pub fn members(&self) -> &[AnchoredRecordMemberLocator] {
        &self.membership.members
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
    pub fn into_parts(self) -> (AnchoredRecordMembership, Vec<u8>) {
        (self.membership, self.bytes)
    }
}
