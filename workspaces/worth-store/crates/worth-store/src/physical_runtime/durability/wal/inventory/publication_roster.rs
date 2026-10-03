//! Authenticated, bounded publication-group facts retained beside WAL inventory.
//! Charge reconstruction belongs to the root owner, not to this decoder.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_canonical_redo_v3, PersistedPhysicalRecoveryOperation,
    PersistedPhysicalRecoveryProjection, PhysicalRecordFormatDeclaration,
    PhysicalRecoveryProjectionDecodeLimits, PhysicalRewriteRedo, CANONICAL_REDO_V3_DOMAIN,
    REWRITE_REDO_DOMAIN,
};
use worth_store_wal::{VerifiedWalFramePayload, WalLsnRange};

use super::PhysicalWalOpenFailure;
use crate::physical_runtime::durability::{
    reopened_membership_digest_fields, PersistedPhysicalMutationAttemptBinding,
    PhysicalBindingDecodingContext,
};

const COPY_PUBLICATION_DOMAIN: &[u8] = b"store.physical.extent-copy-publication.v1";

#[path = "publication_roster/metadata.rs"]
mod metadata;
#[cfg(test)]
#[path = "publication_roster/tests.rs"]
mod tests;
use metadata::decode_metadata;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct ReopenedPublicationMemberMetadata {
    source_root_generation: u64,
    successor_manifest_capacity: Option<u16>,
    inserted_records: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::physical_runtime) struct ReopenedWalPublicationGroup {
    segment: u64,
    generation: u64,
    lsn_range: WalLsnRange,
    encoded_wal_bytes: u64,
    members: Vec<ReopenedPublicationMemberMetadata>,
}

pub(super) struct PublicationRoster {
    members: Vec<ObservedPublicationMember>,
    admitted_bytes: u64,
}

struct ObservedPublicationMember {
    group: [u8; 32],
    membership: [u8; 32],
    ordinal: u32,
    count: u32,
    segment: u64,
    generation: u64,
    range: WalLsnRange,
    encoded_bytes: u64,
    metadata: ReopenedPublicationMemberMetadata,
    mutation_store: [u8; 16],
    mutation_runtime: u64,
    mutation_operation: u64,
    member_identity: [u8; 32],
    idempotency_identity: [u8; 32],
}

impl PublicationRoster {
    pub(super) const fn new(admitted_bytes: u64) -> Self {
        Self {
            members: Vec::new(),
            admitted_bytes,
        }
    }

    pub(super) fn admitted_frame_view_count(
        &self,
        active_segment_bytes: u64,
    ) -> Result<u64, PhysicalWalOpenFailure> {
        let required = retained_roster_ceiling(self.members.len())?
            .checked_add(active_segment_bytes)
            .ok_or(PhysicalWalOpenFailure::CounterOverflow)?;
        let available = self.admitted_bytes.checked_sub(required).ok_or(
            PhysicalWalOpenFailure::ReopenAllocationLimitExceeded {
                admitted: self.admitted_bytes,
                required,
            },
        )?;
        let slots =
            available / std::mem::size_of::<worth_store_wal::VerifiedWalFramePayload<'_>>() as u64;
        // The WAL verifier's Vec starts at four slots and grows by doubling.
        // Reallocation can hold old k and new 2k slots at once.
        Ok(if slots < 4 { 0 } else { slots / 3 })
    }

    pub(super) fn frame_view_capacity_ceiling(count: usize) -> Result<u64, PhysicalWalOpenFailure> {
        if count == 0 {
            return Ok(0);
        }
        (count as u64)
            .checked_mul(3)
            .map(|slots| slots.max(4))
            .and_then(|slots| {
                slots.checked_mul(
                    std::mem::size_of::<worth_store_wal::VerifiedWalFramePayload<'_>>() as u64,
                )
            })
            .ok_or(PhysicalWalOpenFailure::CounterOverflow)
    }

    /// Retains one publication member and returns whether it is a released
    /// drop.
    pub(super) fn observe(
        &mut self,
        frame: VerifiedWalFramePayload<'_>,
        segment: u64,
        generation: u64,
        format: PhysicalRecordFormatDeclaration,
        context: PhysicalBindingDecodingContext,
        active_segment_bytes: u64,
        active_frame_views_bytes: u64,
    ) -> Result<bool, PhysicalWalOpenFailure> {
        let mut payload = frame.payload();
        let binding_bytes = field(&mut payload)?;
        let redo = field(&mut payload)?;
        if !payload.is_empty() || binding_bytes.is_empty() || redo.is_empty() {
            return Err(PhysicalWalOpenFailure::MemberPayloadRejected);
        }
        let binding = PersistedPhysicalMutationAttemptBinding::decode_from_wal_member(
            binding_bytes,
            context,
            frame.lsn_range(),
            Sha256::digest(redo).into(),
        )
        .map_err(|_| PhysicalWalOpenFailure::MemberPayloadRejected)?;
        let next_count = self
            .members
            .len()
            .checked_add(1)
            .ok_or(PhysicalWalOpenFailure::CounterOverflow)?;
        let retained = retained_roster_ceiling(next_count)?;
        let required = retained
            .checked_add(active_segment_bytes)
            .and_then(|bytes| bytes.checked_add(active_frame_views_bytes))
            .ok_or(PhysicalWalOpenFailure::CounterOverflow)?;
        let available = self.admitted_bytes.checked_sub(required).ok_or(
            PhysicalWalOpenFailure::ReopenAllocationLimitExceeded {
                admitted: self.admitted_bytes,
                required,
            },
        )?;
        let (metadata, released_drop) =
            decode_metadata(redo, frame.lsn_range(), format, available)?;
        let group = binding.group();
        let mutation = binding.mutation();
        self.members
            .try_reserve_exact(1)
            .map_err(|_| PhysicalWalOpenFailure::ReopenAllocationRejected)?;
        self.members.push(ObservedPublicationMember {
            group: group.group_identity().bytes(),
            membership: group.membership_digest(),
            ordinal: group.ordinal().get(),
            count: group.member_count().get(),
            segment,
            generation,
            range: frame.lsn_range(),
            encoded_bytes: frame.encoded_bytes(),
            metadata,
            mutation_store: mutation.store_identity().bytes(),
            mutation_runtime: mutation.runtime_identity().get(),
            mutation_operation: mutation.operation_identity().get(),
            member_identity: binding.member().member_identity().bytes(),
            idempotency_identity: binding.idempotency_identity().bytes(),
        });
        Ok(released_drop)
    }

    pub(super) fn finish(
        mut self,
    ) -> Result<Vec<ReopenedWalPublicationGroup>, PhysicalWalOpenFailure> {
        self.members
            .sort_unstable_by_key(|member| (member.group, member.ordinal));
        let mut groups = Vec::new();
        let mut members = self.members.into_iter().peekable();
        while let Some(group) = members.peek().map(|member| member.group) {
            let mut group_members = Vec::new();
            while members.peek().is_some_and(|member| member.group == group) {
                group_members
                    .try_reserve_exact(1)
                    .map_err(|_| PhysicalWalOpenFailure::ReopenAllocationRejected)?;
                group_members.push(members.next().expect("peeked member exists"));
            }
            groups
                .try_reserve_exact(1)
                .map_err(|_| PhysicalWalOpenFailure::ReopenAllocationRejected)?;
            groups.push(finish_group(group_members)?);
        }
        Ok(groups)
    }
}

fn finish_group(
    mut members: Vec<ObservedPublicationMember>,
) -> Result<ReopenedWalPublicationGroup, PhysicalWalOpenFailure> {
    members.sort_unstable_by_key(|member| member.ordinal);
    let first = members
        .first()
        .ok_or(PhysicalWalOpenFailure::MemberPayloadRejected)?;
    let segment = first.segment;
    let generation = first.generation;
    let membership = first.membership;
    let count = first.count;
    if members.len() != count as usize {
        return Err(PhysicalWalOpenFailure::IncompletePublicationGroup);
    }
    let actual_membership = reopened_membership_digest_fields(
        members.len(),
        members.iter().map(|member| {
            (
                member.mutation_store,
                member.mutation_runtime,
                member.mutation_operation,
                member.member_identity,
                member.idempotency_identity,
            )
        }),
    )
    .ok_or(PhysicalWalOpenFailure::MemberPayloadRejected)?;
    if actual_membership != membership {
        return Err(PhysicalWalOpenFailure::MemberPayloadRejected);
    }
    let mut identities = Vec::new();
    identities
        .try_reserve_exact(members.len())
        .map_err(|_| PhysicalWalOpenFailure::ReopenAllocationRejected)?;
    identities.extend(
        members
            .iter()
            .map(|member| (member.member_identity, member.idempotency_identity)),
    );
    identities.sort_unstable_by_key(|pair| pair.0);
    if identities.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(PhysicalWalOpenFailure::MemberPayloadRejected);
    }
    identities.sort_unstable_by_key(|pair| pair.1);
    if identities.windows(2).any(|pair| pair[0].1 == pair[1].1) {
        return Err(PhysicalWalOpenFailure::MemberPayloadRejected);
    }
    let mut end = first.range.start();
    let mut encoded_wal_bytes = 0_u64;
    for (index, member) in members.iter().enumerate() {
        if member.ordinal as usize != index + 1
            || member.count != count
            || member.membership != membership
            || member.segment != segment
            || member.generation != generation
            || member.range.start() != end
        {
            return Err(PhysicalWalOpenFailure::MemberPayloadRejected);
        }
        end = member.range.end_exclusive();
        encoded_wal_bytes = encoded_wal_bytes
            .checked_add(member.encoded_bytes)
            .ok_or(PhysicalWalOpenFailure::CounterOverflow)?;
    }
    let lsn_range = WalLsnRange::new(first.range.start(), end)
        .map_err(|_| PhysicalWalOpenFailure::MemberPayloadRejected)?;
    let mut metadata = Vec::new();
    metadata
        .try_reserve_exact(members.len())
        .map_err(|_| PhysicalWalOpenFailure::ReopenAllocationRejected)?;
    metadata.extend(members.into_iter().map(|member| member.metadata));
    Ok(ReopenedWalPublicationGroup {
        segment,
        generation,
        lsn_range,
        encoded_wal_bytes,
        members: metadata,
    })
}

fn retained_roster_ceiling(count: usize) -> Result<u64, PhysicalWalOpenFailure> {
    // Raw observations remain allocated while finished groups are assembled.
    // Worst case every observed member is a separate group; the temporary
    // group vector and final member metadata coexist with the raw storage.
    let per_member = std::mem::size_of::<ObservedPublicationMember>()
        .checked_mul(2)
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<ReopenedWalPublicationGroup>()))
        .and_then(|bytes| {
            bytes.checked_add(std::mem::size_of::<ReopenedPublicationMemberMetadata>())
        })
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<([u8; 32], [u8; 32])>()))
        .ok_or(PhysicalWalOpenFailure::CounterOverflow)?;
    u64::try_from(count)
        .ok()
        .and_then(|count| count.checked_mul(per_member as u64))
        .ok_or(PhysicalWalOpenFailure::CounterOverflow)
}

fn take_u64(bytes: &mut &[u8]) -> Result<u64, PhysicalWalOpenFailure> {
    let (head, tail) = bytes
        .split_at_checked(8)
        .ok_or(PhysicalWalOpenFailure::MemberPayloadRejected)?;
    *bytes = tail;
    Ok(u64::from_le_bytes(head.try_into().unwrap()))
}

fn field<'a>(bytes: &mut &'a [u8]) -> Result<&'a [u8], PhysicalWalOpenFailure> {
    let count = usize::try_from(take_u64(bytes)?)
        .map_err(|_| PhysicalWalOpenFailure::MemberPayloadRejected)?;
    let (head, tail) = bytes
        .split_at_checked(count)
        .ok_or(PhysicalWalOpenFailure::MemberPayloadRejected)?;
    *bytes = tail;
    Ok(head)
}

impl ReopenedWalPublicationGroup {
    pub(in crate::physical_runtime) fn retained_metadata_capacity_bytes(&self) -> u64 {
        (std::mem::size_of::<Self>() as u64).saturating_add(
            (self.members.capacity() as u64)
                .saturating_mul(std::mem::size_of::<ReopenedPublicationMemberMetadata>() as u64),
        )
    }
    pub(in crate::physical_runtime) const fn segment(&self) -> (u64, u64) {
        (self.segment, self.generation)
    }
    pub(in crate::physical_runtime) const fn lsn_range(&self) -> WalLsnRange {
        self.lsn_range
    }
    pub(in crate::physical_runtime) const fn encoded_wal_bytes(&self) -> u64 {
        self.encoded_wal_bytes
    }
    pub(in crate::physical_runtime) fn members(&self) -> &[ReopenedPublicationMemberMetadata] {
        &self.members
    }
}

impl ReopenedPublicationMemberMetadata {
    pub(in crate::physical_runtime) const fn source_root_generation(self) -> u64 {
        self.source_root_generation
    }
    pub(in crate::physical_runtime) const fn successor_manifest_capacity(self) -> Option<u16> {
        self.successor_manifest_capacity
    }
    pub(in crate::physical_runtime) const fn inserted_records(self) -> u64 {
        self.inserted_records
    }
}
