//! Replace actual fresh-call reader sets without changing the original key universe.
use super::super::super::retained_decision_facts::{StorageControl, StoreDenial};
use super::*;
use std::sync::Arc;
use worth_execution::{ChargedBytes, ExecutionOrderedMapDenial};

pub(super) fn storage_denial(cause: ExecutionOrderedMapDenial) -> StoreDenial {
    match cause {
        ExecutionOrderedMapDenial::Allocation(cause) => StoreDenial::Allocation(cause),
        ExecutionOrderedMapDenial::CapacityOverflow => StoreDenial::Representability,
    }
}
impl SealedComputationFacts {
    /// The caller already established real completed-run/native-pin closure.
    /// Every changed call is replaced, not unioned with its old readers. Keys
    /// cannot enter/leave the completed computation through this fixed-key door.
    pub(super) fn reattribute(
        &self,
        calls: &BTreeSet<ComputationRead>,
        fresh: &BTreeMap<ComputationRead, Vec<&WorthQueryApplicationFactKey>>,
        control: StorageControl<'_, '_>,
    ) -> Result<Option<Self>, StoreDenial> {
        control.check_live()?;
        let funding = if control.policy().uses_accounting() {
            let count = calls
                .iter()
                .try_fold(0usize, |sum, read| {
                    sum.checked_add(self.index.keys(*read).count())?
                        .checked_add(fresh.get(read).map_or(0, Vec::len))
                })
                .ok_or(StoreDenial::Representability)?;
            let bytes = count
                .checked_mul(64 * size_of::<usize>() + 16 * size_of::<ComputationRead>())
                .and_then(|n| u64::try_from(n).ok())
                .ok_or(StoreDenial::Representability)?;
            control.policy().reserve_memory(bytes)
        } else {
            Ok(None)
        };
        control.check_live()?;
        let _funding = funding?;
        let mut changes: BTreeMap<
            &WorthQueryApplicationFactKey,
            (BTreeSet<ComputationRead>, BTreeSet<ComputationRead>),
        > = BTreeMap::new();
        for read in calls {
            control.check_live()?;
            let mut old = self.index.keys(*read).peekable();
            let mut new = fresh.get(read).into_iter().flatten().copied().peekable();
            while old.peek().is_some() || new.peek().is_some() {
                match (old.peek(), new.peek()) {
                    (Some(a), Some(b)) if a == b => {
                        old.next();
                        new.next();
                    }
                    (Some(a), Some(b)) if a < b => {
                        changes
                            .entry(old.next().unwrap())
                            .or_default()
                            .0
                            .insert(*read);
                    }
                    (Some(_), Some(_)) | (None, Some(_)) => {
                        let key = new.next().unwrap();
                        if !self.metadata.contains_key(key) {
                            return Ok(None);
                        }
                        changes.entry(key).or_default().1.insert(*read);
                    }
                    (Some(_), None) => {
                        changes
                            .entry(old.next().unwrap())
                            .or_default()
                            .0
                            .insert(*read);
                    }
                    (None, None) => break,
                }
            }
        }
        let mut metadata = (*self.metadata).clone();
        for (key, (removed, added)) in changes {
            control.check_live()?;
            super::super::super::super::fact_verification_observation::completion_fresh_key();
            let old = self
                .metadata
                .get(key)
                .expect("old or checked fresh dependency");
            let memory = if control.policy().uses_accounting() {
                let count = old
                    .readers
                    .reads()
                    .count()
                    .checked_add(added.len())
                    .ok_or(StoreDenial::Representability)?;
                let quote = count
                    .checked_mul(32 * size_of::<usize>() + size_of::<ComputationRead>())
                    .and_then(|n| u64::try_from(n).ok())
                    .ok_or(StoreDenial::Representability)?;
                control.policy().reserve_memory(quote)
            } else {
                Ok(None)
            };
            control.check_live()?;
            let _memory = memory?;
            let mut reads = ComputationFactReads::default();
            for read in old
                .readers
                .reads()
                .filter(|read| !removed.contains(read))
                .chain(added)
            {
                reads.record(read);
            }
            if reads.reads().next().is_none() {
                return Ok(None);
            }
            let value = SealedComputationFact {
                ordinal: old.ordinal,
                readers: reads.in_order(),
            };
            metadata
                .insert_cloned_in(key, &value, control.policy())
                .map_err(storage_denial)?;
        }
        let mut index = (*self.index).clone();
        for read in calls {
            control.check_live()?;
            let keys = fresh.get(read);
            if keys.is_none_or(|keys| keys.is_empty()) {
                index
                    .calls
                    .remove_in(read, control.policy())
                    .map_err(storage_denial)?;
                continue;
            }
            let funding = if control.policy().uses_accounting() {
                let mut quote = keys
                    .map_or(0, |v| v.len())
                    .checked_mul(size_of::<WorthQueryApplicationFactKey>())
                    .and_then(|n| u64::try_from(n).ok())
                    .ok_or(StoreDenial::Representability)?;
                for key in keys.into_iter().flatten() {
                    quote = quote
                        .checked_add(key.additional_charged_bytes())
                        .ok_or(StoreDenial::Representability)?;
                }
                control.policy().reserve_memory(quote)
            } else {
                Ok(None)
            };
            control.check_live()?;
            let _funding = funding?;
            let mut copied = Vec::with_capacity(keys.map_or(0, Vec::len));
            copied.extend(keys.into_iter().flatten().map(|key| (*key).clone()));
            index
                .calls
                .insert_cloned_in(read, &copied, control.policy())
                .map_err(storage_denial)?;
        }
        let mut result = self.clone();
        result.metadata = Arc::new(metadata);
        result.index = Arc::new(index);
        Ok(Some(result))
    }
}
