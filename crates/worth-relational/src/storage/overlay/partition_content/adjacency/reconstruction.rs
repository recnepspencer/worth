//! Full adjacency commitment construction; no implicit empty predecessor.
use super::*;
use crate::storage::substrate::maintenance::{ReconstructionLane, ReconstructionWork};
impl AdjacencyContentIndex {
    pub(in crate::storage::overlay::partition_content) fn rebuild(
        &mut self,
        lane: &ReconstructionLane<'_>,
        work: &mut ReconstructionWork<'_, '_>,
        request: super::super::request_admission::Request<'_, '_, '_>,
        current: &SparseAdjacencyTable,
    ) -> Result<u64, PartitionContentDigestError> {
        work.prepare(
            current.iter().len() as u64,
            (current.iter().len()
                * (size_of::<(usize, DigestIndex)>() + size_of::<(u128, [u8; 32])>()))
                as u64,
        )?;
        let mut slots = Vec::new();
        let mut memberships = Vec::new();
        let mut hashed = 0;
        for (&slot, set) in current.iter() {
            work.prepare(1, 0)?;
            let count = set.current_ids().len();
            work.prepare(count as u64, (count * size_of::<(u128, [u8; 32])>()) as u64)?;
            let mut entries = Vec::with_capacity(count);
            for id in set.current_ids().iter() {
                let key = ((id.partition_id.as_u32() as u128) << 96)
                    | ((id.slot_index() as u128) << 32)
                    | id.generation_value() as u128;
                entries.push((
                    key,
                    super::super::admission::hash_controlled(b"relation-membership", id, work)?,
                ));
                hashed += 1;
            }
            let (units, _) = DigestIndex::construction_quote(entries.len())
                .ok_or(PartitionContentDigestError::AllocationAccountingOverflow)?;
            work.prepare(units, 0)?;
            let membership = DigestIndex::from_sorted_entries_for_request(lane, &entries, request)?;
            self.membership_bytes = self
                .membership_bytes
                .checked_add(membership.allocation_bytes())
                .ok_or(PartitionContentDigestError::AllocationAccountingOverflow)?;
            slots.push((slot as u128, membership.digest()));
            memberships.push((slot, membership));
        }
        // SparseAdjacencyTable's native map already yields ascending unique slots.
        let (units, _) = DigestIndex::construction_quote(slots.len())
            .ok_or(PartitionContentDigestError::AllocationAccountingOverflow)?;
        work.prepare(units, 0)?;
        self.slots = DigestIndex::from_sorted_entries_for_request(lane, &slots, request)?;
        work.prepare(memberships.len() as u64, 0)?;
        for (slot, membership) in memberships {
            match request {
                Some(request) => {
                    self.memberships
                        .insert_for_request(slot, membership, request)
                        .map_err(PartitionContentDigestError::Request)?;
                }
                None => {
                    self.memberships.insert(slot, membership);
                }
            }
        }
        Ok(hashed)
    }
}
