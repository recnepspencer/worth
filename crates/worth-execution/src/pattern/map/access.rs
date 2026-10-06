use std::mem::size_of;

use worth_foundational::PartitionIdentity;
use worth_proof::DisjointKeySetFamily;

use crate::report::ChargedBytes;

/// The bytes a map retains for its declared access sets; `None` when the
/// count does not fit.
pub(super) fn access_memory_bytes<K: ChargedBytes>(
    read_sets: &[Vec<K>],
    read_set_capacity: usize,
    write_sets: &DisjointKeySetFamily<PartitionIdentity, K>,
    write_member_capacity: usize,
) -> Option<u64> {
    let keys = read_sets
        .iter()
        .chain(write_sets.members().iter().map(|(_, keys)| keys));
    let mut total = 0_u64;
    for set in keys {
        let inline = set.capacity().checked_mul(size_of::<K>())?;
        total = total.checked_add(u64::try_from(inline).ok()?)?;
        for key in set {
            total = total.checked_add(key.additional_charged_bytes())?;
        }
    }
    let read_members = read_set_capacity.checked_mul(size_of::<Vec<K>>())?;
    let write_members =
        write_member_capacity.checked_mul(size_of::<(PartitionIdentity, Vec<K>)>())?;
    total
        .checked_add(u64::try_from(read_members).ok()?)?
        .checked_add(u64::try_from(write_members).ok()?)
}
