//! Encode resident earlier suffixes and declare their cold member dependencies.
use super::region_image::{RegionHistoryBase, RegionHistoryRow};
use crate::durability::data::{
    RelationalCheckpointHistoryDependency, RelationalHistoryMemberDirectory,
};
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::overlay::PartitionState;
use crate::storage::substrate::{RecordArena, RecordKind};
use std::collections::BTreeMap;
use std::sync::Arc;

pub(super) fn split_rows<K: RecordKind, T>(
    arena: &RecordArena<K>,
    rows: &mut [Vec<T>],
) -> Result<Vec<RegionHistoryRow<T>>, Cause> {
    let mut history = Vec::new();
    history
        .try_reserve_exact(rows.len())
        .map_err(|_| Cause::MemoryExhausted)?;
    for (physical, rows) in rows.iter_mut().enumerate() {
        let prefix = arena.history_prefixes[physical]
            .as_ref()
            .map(|prefix| {
                if prefix.partition_id != arena.partition_ids[physical] || rows.is_empty() {
                    return Err(Cause::Damaged);
                }
                Ok(RegionHistoryBase {
                    member: prefix.member,
                    partition_id: prefix.partition_id,
                    schema_carrier_digest: prefix.schema_carrier_digest,
                    row_index: u64::try_from(prefix.row_index).map_err(|_| Cause::Damaged)?,
                    kind_commitment: prefix.history_opening.kind_commitment,
                })
            })
            .transpose()?;
        let current = rows.pop();
        let earlier = std::mem::take(rows);
        rows.extend(current);
        history.push(RegionHistoryRow { prefix, earlier });
    }
    Ok(history)
}

pub(super) fn directory(
    partition: &PartitionState,
    member: RelationalDurableMemberLocator,
    bases: Vec<RelationalDurableMemberLocator>,
) -> Result<Arc<RelationalHistoryMemberDirectory>, Cause> {
    let mut declarations = BTreeMap::new();
    let mut visited = BTreeMap::<_, Vec<&Arc<RelationalHistoryMemberDirectory>>>::new();
    for prefix in partition
        .entity_arena
        .history_prefixes
        .iter()
        .chain(partition.relation_arena.history_prefixes.iter())
        .flatten()
    {
        let directories = visited.entry(prefix.member).or_default();
        if directories
            .iter()
            .any(|directory| Arc::ptr_eq(directory, &prefix.history_members))
        {
            continue;
        }
        directories.push(&prefix.history_members);
        let members = prefix.history_members.members(prefix.member);
        for declaration in prefix.history_members.declarations_for(&members) {
            merge(&mut declarations, declaration)?;
        }
    }
    if !bases.is_empty() {
        merge(
            &mut declarations,
            RelationalCheckpointHistoryDependency { member, bases },
        )?;
    }
    let declarations = declarations
        .into_iter()
        .map(|(member, bases)| RelationalCheckpointHistoryDependency { member, bases })
        .collect::<Vec<_>>();
    RelationalHistoryMemberDirectory::readmit(&declarations, [member]).map(Arc::new)
}

fn merge(
    declarations: &mut BTreeMap<
        RelationalDurableMemberLocator,
        Vec<RelationalDurableMemberLocator>,
    >,
    declaration: RelationalCheckpointHistoryDependency,
) -> Result<(), Cause> {
    if let Some(prior) = declarations.get(&declaration.member) {
        if prior != &declaration.bases {
            return Err(Cause::IntegrityMismatch);
        }
    } else {
        declarations.insert(declaration.member, declaration.bases);
    }
    Ok(())
}
