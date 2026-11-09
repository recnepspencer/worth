//! Direct grants for changed member groups; path nodes use ordered storage.
use super::*;
use crate::ExecutionMemoryReservation;
pub(super) struct Members {
    pub(super) values: PartitionMembers,
    _memory: Option<ExecutionMemoryReservation>,
}
impl Members {
    fn edited(
        old: Option<&Self>,
        item: PartitionItemId,
        insert: bool,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Arc<Self>, ExecutionOrderedMapDenial> {
        let memory = if policy.uses_accounting() {
            policy.reserve_memory((size_of::<Self>() + 2 * size_of::<usize>()) as u64)?
        } else {
            None
        };
        let mut values = old.map_or_else(PartitionMembers::default, |m| m.values.clone());
        if insert {
            values.0.insert_cloned_in(&item, &(), policy)?;
        } else {
            values.0.remove_in(&item, policy)?;
        }
        Ok(Arc::new(Self {
            values,
            _memory: memory,
        }))
    }
}
impl<K: Ord + Clone + ChargedBytes> KeyedPartitioner<K> {
    /// Full reconstruction is explicit; ordinary rekeys clone shared roots.
    pub fn kept_in(
        &self,
        mut keep: impl FnMut(PartitionItemId) -> bool,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, KeyedEditDenial<ExecutionOrderedMapDenial>> {
        let mut next = Self::new();
        for entry in self.items.values().filter(|entry| keep(entry.item)) {
            next.upsert_in(entry, policy)?;
        }
        Ok(next)
    }
    /// O(log N + log affected-group size), with direct retained grants. The
    /// complete candidate root is published only after all allocations succeed.
    pub fn upsert_in(
        &mut self,
        entry: &KeyedItem<K>,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<PartitionWork, KeyedEditDenial<ExecutionOrderedMapDenial>> {
        policy
            .check_live()
            .map_err(ExecutionOrderedMapDenial::from)
            .map_err(KeyedEditDenial::Admission)?;
        self.check(entry).map_err(KeyedEditDenial::Keyed)?;
        if self.items.get(&entry.item) == Some(entry) {
            return Ok(PartitionWork::default());
        }
        let mut next = self.clone();
        next.key_bytes = if policy.uses_accounting() {
            Some(
                self.key_bytes
                    .or_else(|| self.census_key_bytes())
                    .ok_or(KeyedEditDenial::BoundOverflow)?,
            )
        } else {
            None
        };
        next.remove_in(entry.item, policy)
            .map_err(KeyedEditDenial::Admission)?;
        let fresh = !next.keys.contains_key(&entry.key);
        let old = next.keys.get(&entry.key);
        let members = Members::edited(old.map(|g| &*g.members), entry.item, true, policy)
            .map_err(KeyedEditDenial::Admission)?;
        let group = Group {
            partition: entry.partition,
            members,
        };
        if fresh {
            next.identities
                .insert_cloned_in(&entry.partition, &entry.key, policy)
                .map_err(KeyedEditDenial::Admission)?;
            if let Some(bytes) = next.key_bytes {
                next.key_bytes = Some(
                    bytes
                        .checked_add(
                            entry
                                .key
                                .additional_charged_bytes()
                                .checked_mul(2)
                                .ok_or(KeyedEditDenial::BoundOverflow)?,
                        )
                        .ok_or(KeyedEditDenial::BoundOverflow)?,
                );
            }
        }
        next.keys
            .insert_cloned_in(&entry.key, &group, policy)
            .map_err(KeyedEditDenial::Admission)?;
        next.items
            .insert_cloned_in(&entry.item, entry, policy)
            .map_err(KeyedEditDenial::Admission)?;
        if let Some(bytes) = next.key_bytes {
            next.key_bytes = Some(
                bytes
                    .checked_add(entry.key.additional_charged_bytes())
                    .ok_or(KeyedEditDenial::BoundOverflow)?,
            );
        }
        let rerouted = self.route(entry.item) != Some(entry.partition);
        *self = next;
        Ok(PartitionWork {
            items_rerouted: u64::from(rerouted),
            members_visited: 1,
            ..Default::default()
        })
    }
    pub fn remove_in(
        &mut self,
        item: PartitionItemId,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<PartitionWork, ExecutionOrderedMapDenial> {
        policy.check_live()?;
        let Some(entry) = self.items.get(&item) else {
            return Ok(PartitionWork::default());
        };
        let group = self.keys.get(&entry.key).expect("routed item owns a group");
        let mut next = self.clone();
        next.key_bytes = if policy.uses_accounting() {
            Some(
                self.key_bytes
                    .or_else(|| self.census_key_bytes())
                    .ok_or(ExecutionOrderedMapDenial::CapacityOverflow)?,
            )
        } else {
            None
        };
        let mut released = next.key_bytes.map(|_| entry.key.additional_charged_bytes());
        if group.members.values.len() == 1 {
            next.keys.remove_in(&entry.key, policy)?;
            next.identities.remove_in(&entry.partition, policy)?;
            released = released
                .map(|bytes| {
                    bytes
                        .checked_mul(3)
                        .ok_or(ExecutionOrderedMapDenial::CapacityOverflow)
                })
                .transpose()?;
        } else {
            let members = Members::edited(Some(&group.members), item, false, policy)?;
            next.keys.insert_cloned_in(
                &entry.key,
                &Group {
                    partition: group.partition,
                    members,
                },
                policy,
            )?;
        }
        next.items.remove_in(&item, policy)?;
        if let Some((bytes, released)) = next.key_bytes.zip(released) {
            next.key_bytes = Some(
                bytes
                    .checked_sub(released)
                    .ok_or(ExecutionOrderedMapDenial::CapacityOverflow)?,
            );
        }
        *self = next;
        Ok(PartitionWork {
            items_rerouted: 1,
            members_visited: 1,
            ..Default::default()
        })
    }
}
