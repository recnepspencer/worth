//! Paged dictionary admission. I/O occurs only in explicit demands, before
//! taking the symbol table's mutation lock.
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::sync::{Arc, Mutex};

use super::symbol_table::RuntimeSymbolTable;
use crate::durability::data::{RelationalCheckpointPage, RelationalCheckpointSymbolBucket};
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberLocator};
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};
use crate::storage::residency::RelationalResidencyDemandFailure as Failure;
use crate::symbols::data::Symbol;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub(super) struct RuntimeSymbolResidency {
    regions: Mutex<HashSet<crate::storage::residency::RelationalRegionSymbolAdmissionIdentity>>,
    by_id: Vec<RelationalCheckpointPage>,
    by_value: Vec<RelationalCheckpointSymbolBucket>,
    source: Arc<dyn RelationalDurableColdSource>,
    pages:
        Mutex<BTreeMap<RelationalDurableMemberLocator, Result<(), RelationalStorageUnavailable>>>,
}

impl RuntimeSymbolResidency {
    pub(super) fn all_id_pages_resident(&self) -> bool {
        let pages = self.pages.lock().unwrap_or_else(|error| error.into_inner());
        self.by_id
            .iter()
            .all(|page| matches!(pages.get(&page.member), Some(Ok(()))))
    }
    pub(super) fn new(
        by_id: Vec<RelationalCheckpointPage>,
        by_value: Vec<RelationalCheckpointSymbolBucket>,
        source: Arc<dyn RelationalDurableColdSource>,
    ) -> Self {
        Self {
            regions: Mutex::new(HashSet::new()),
            by_id,
            by_value,
            source,
            pages: Mutex::new(BTreeMap::new()),
        }
    }

    pub(super) fn detached_with_owner<T>(&self, snapshot: impl FnOnce() -> T) -> (Self, T) {
        let pages = self
            .pages
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let owner = snapshot();
        (
            Self {
                regions: Mutex::new(HashSet::new()),
                by_id: self.by_id.clone(),
                by_value: self.by_value.clone(),
                source: self.source.clone(),
                pages: Mutex::new(pages.clone()),
            },
            owner,
        )
    }

    pub(super) fn same_directory(&self, other: &Self) -> bool {
        self.by_id == other.by_id && self.by_value == other.by_value
    }

    pub(super) fn demand_region(
        &self,
        owner: &RuntimeSymbolTable,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        region: crate::storage::residency::RelationalRegionSymbolAdmissionIdentity,
        ids: impl FnOnce() -> BTreeSet<Symbol>,
    ) -> Result<(), Failure> {
        crate::execution::check_request(execution).map_err(Failure::Request)?;
        let mut admitted = self
            .regions
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if admitted.contains(&region) {
            return Ok(());
        }
        admitted.try_reserve(1).map_err(|_| {
            unavailable(
                RelationalStorageSubject::Symbols {
                    first: Symbol(0),
                    last: Symbol(u32::MAX - 1),
                },
                Cause::MemoryExhausted,
            )
        })?;
        self.demand_ids(owner, execution, ids())?;
        admitted.insert(region);
        Ok(())
    }

    pub(super) fn demand_ids(
        &self,
        owner: &RuntimeSymbolTable,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        ids: impl IntoIterator<Item = Symbol>,
    ) -> Result<(), Failure> {
        for id in ids {
            if owner.with_read(|symbols| symbols.retained_text(id).is_some()) {
                continue;
            }
            let position = self.by_id.partition_point(|page| page.last < id.0 as u64);
            let page = self
                .by_id
                .get(position)
                .filter(|page| page.first <= id.0 as u64)
                .ok_or_else(|| {
                    unavailable(
                        RelationalStorageSubject::Symbols {
                            first: id,
                            last: id,
                        },
                        Cause::SourceUnavailable,
                    )
                })?;
            self.demand_page(
                owner,
                execution,
                page.member,
                RelationalStorageSubject::Symbols {
                    first: Symbol(page.first as u32),
                    last: Symbol(page.last as u32),
                },
            )?;
            if owner.with_read(|symbols| symbols.retained_text(id).is_none()) {
                return Err(unavailable(
                    RelationalStorageSubject::Symbols {
                        first: id,
                        last: id,
                    },
                    Cause::Damaged,
                )
                .into());
            }
        }
        Ok(())
    }

    pub(super) fn demand_value(
        &self,
        owner: &RuntimeSymbolTable,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        value: &str,
    ) -> Result<(), Failure> {
        if owner.with_read(|symbols| symbols.contains(value)) {
            return Ok(());
        }
        use sha2::{Digest, Sha256};
        let bucket = Sha256::digest(value.as_bytes())[0];
        let Some(pages) = self.by_value.iter().find(|entry| entry.bucket == bucket) else {
            return Ok(());
        };
        for locator in &pages.pages {
            self.demand_page(
                owner,
                execution,
                *locator,
                RelationalStorageSubject::Symbols {
                    first: Symbol(0),
                    last: Symbol(u32::MAX - 1),
                },
            )?;
            if owner.with_read(|symbols| symbols.contains(value)) {
                break;
            }
        }
        Ok(())
    }

    fn demand_page(
        &self,
        owner: &RuntimeSymbolTable,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        locator: RelationalDurableMemberLocator,
        subject: RelationalStorageSubject,
    ) -> Result<(), Failure> {
        crate::execution::check_request(execution).map_err(Failure::Request)?;
        let mut pages = self
            .pages
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(result) = pages.get(&locator) {
            return (*result).map_err(Into::into);
        }
        let result = (|| {
            let bytes = self
                .source
                .read_member(locator, 1 << 30)
                .map_err(|cause| unavailable(subject, cause))?;
            let snapshot = crate::durability::checkpoints::symbol_pages::readmit(
                execution, &bytes, locator, subject,
            )?;
            owner
                .import_canonical_snapshot(&snapshot)
                .map_err(|_| unavailable(subject, Cause::Damaged))?;
            Ok(())
        })();
        match &result {
            Ok(()) => {
                pages.insert(locator, Ok(()));
            }
            Err(Failure::StorageUnavailable(failure)) if failure.cause.is_proven_damage() => {
                pages.insert(locator, Err(*failure));
            }
            Err(Failure::StorageUnavailable(_)) => {}
            Err(Failure::Request(_)) => {}
        }
        result
    }
}

fn unavailable(subject: RelationalStorageSubject, cause: Cause) -> RelationalStorageUnavailable {
    RelationalStorageUnavailable { subject, cause }
}
