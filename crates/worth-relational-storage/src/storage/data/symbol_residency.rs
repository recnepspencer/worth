//! Paged dictionary admission. I/O occurs only in explicit demands, before
//! taking the symbol table's mutation lock.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCauseOwnerPort as _;

use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberLocator};
use crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
use crate::storage::data::{
    checkpoint_page::RelationalCheckpointPage, checkpoint_pages::CheckpointPages,
    checkpoint_symbol_buckets::CheckpointSymbolBuckets,
    symbol_page_roster::RelationalCheckpointSymbolBucket,
};
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause, SymbolPageShape,
};
use crate::storage::residency::RelationalResidencyDemandFailure as Failure;
use crate::symbols::data::Symbol;

pub mod admission;
pub mod member_export;
pub mod selected_directory;
pub mod source_retirement;
use crate::storage::data::SymbolPageSelection;
use source_retirement::SymbolPageState;

#[derive(Debug)]
pub struct RuntimeSymbolResidency {
    pub by_id: CheckpointPages,
    pub by_value: CheckpointSymbolBuckets,
    pub pages: Mutex<SymbolPageState>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl RuntimeSymbolResidency {
    pub fn all_id_pages_resident(&self) -> bool {
        let pages = self.pages.lock().unwrap_or_else(|error| error.into_inner());
        self.by_id
            .iter()
            .all(|page| matches!(pages.get(&page.member), Some(Ok(_))))
    }
    pub fn new(
        by_id: Vec<RelationalCheckpointPage>,
        by_value: Vec<RelationalCheckpointSymbolBucket>,
        source: Arc<dyn RelationalDurableColdSource>,
    ) -> Result<Self, RelationalStorageUnavailable> {
        let members = by_id
            .iter()
            .map(|page| page.member)
            .chain(
                by_value
                    .iter()
                    .flat_map(|bucket| bucket.pages.iter().copied()),
            )
            .collect::<BTreeSet<_>>();
        let placement = source
            .retain_members(&members.into_iter().collect::<Vec<_>>())
            .map_err(|cause| {
                unavailable(
                    RelationalStorageSubject::Symbols {
                        first: Symbol(by_id.first().map_or(0, |page| page.first as u32)),
                        last: Symbol(by_id.last().map_or(0, |page| page.last as u32)),
                    },
                    cause,
                )
            })?;
        Ok(Self {
            by_id: CheckpointPages::from_ordinary(by_id),
            by_value: CheckpointSymbolBuckets::from_ordinary(by_value),
            pages: Mutex::new(SymbolPageState {
                data: Default::default(),
                physical: crate::durability::medium::NativeSourceMemberState::new(
                    source, placement,
                ),
            }),
            _memory: None,
        })
    }

    pub fn detached_with_owner<T>(&self, snapshot: impl FnOnce() -> T) -> (Self, T) {
        // Page admission and snapshot capture both take pages before the
        // dictionary. The same immutable state owns meanings and proof marks.
        let pages = self
            .pages
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let owner = snapshot();
        (
            Self {
                by_id: self.by_id.clone(),
                by_value: self.by_value.clone(),
                pages: Mutex::new(SymbolPageState {
                    data: pages.data.clone(),
                    physical: pages.physical.retain_serving(),
                }),
                _memory: None,
            },
            owner,
        )
    }

    pub fn same_directory(&self, other: &Self) -> bool {
        self.by_id == other.by_id && self.by_value == other.by_value
    }

    pub fn demand_ids(
        &self,
        owner: &RuntimeSymbolTable,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        ids: impl IntoIterator<Item = Symbol>,
    ) -> Result<(), Failure> {
        for id in ids {
            if owner.with_read(|symbols| symbols.retained_text(id).is_some()) {
                continue;
            }
            let page =
                RelationalCheckpointPage::find(&self.by_id, u64::from(id.0)).ok_or_else(|| {
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
                SymbolPageSelection::IdRange {
                    first: Symbol(page.first as u32),
                    last: Symbol(page.last as u32),
                    id_digest: SymbolPageShape::digest_ids(
                        (0..256)
                            .map(|offset| page.range() * 256 + offset + 1)
                            .filter(|id| page.contains(*id)),
                    ),
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

    pub fn demand_value(
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
                SymbolPageSelection::HashBucket(bucket),
            )?;
            if owner.with_read(|symbols| symbols.contains(value)) {
                break;
            }
        }
        Ok(())
    }

    /// A probe may reserve a new text only after every candidate bucket page
    /// has proved that no prior spelling owns the identity.
    pub fn value_bucket_is_resident(
        &self,
        value: &str,
    ) -> Result<bool, RelationalStorageUnavailable> {
        use sha2::{Digest, Sha256};
        let bucket = Sha256::digest(value.as_bytes())[0];
        let Some(bucket) = self.by_value.iter().find(|entry| entry.bucket == bucket) else {
            return Ok(true);
        };
        let mut pages = self.pages.lock().unwrap_or_else(|error| error.into_inner());
        for locator in &bucket.pages {
            match pages.get(locator).copied() {
                Some(Ok(shape)) => {
                    let selection = SymbolPageSelection::HashBucket(bucket.bucket);
                    if let Err(cause) = shape.validate(selection) {
                        let failure = unavailable(selection.subject(), cause);
                        pages.insert(*locator, Err(failure));
                        return Err(failure);
                    }
                }
                Some(Err(failure)) => return Err(failure),
                None => return Ok(false),
            }
        }
        Ok(true)
    }

    #[doc(hidden)]
    pub fn demand_page_for_test(
        &self,
        owner: &RuntimeSymbolTable,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        locator: RelationalDurableMemberLocator,
        selection: SymbolPageSelection,
    ) -> Result<(), Failure> {
        self.demand_page(owner, execution, locator, selection)
    }

    fn demand_page(
        &self,
        owner: &RuntimeSymbolTable,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        locator: RelationalDurableMemberLocator,
        selection: SymbolPageSelection,
    ) -> Result<(), Failure> {
        crate::execution::request_custody::check_request(execution).map_err(Failure::Request)?;
        let subject = selection.subject();
        let mut pages = self
            .pages
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(result) = pages.get(&locator).copied() {
            return match result {
                Ok(shape) => {
                    if let Err(cause) = shape.validate(selection) {
                        let failure = unavailable(subject, cause);
                        let mut refused = pages.data.clone();
                        refused
                            .insert_for_request(locator, Err(failure), execution)
                            .map_err(Failure::Request)?;
                        pages.data = refused;
                        return Err(failure.into());
                    }
                    Ok(())
                }
                Err(failure) => Err(failure.into()),
            };
        }
        let result = (|| {
            let bytes = pages
                .physical
                .source()
                .ok_or_else(|| unavailable(subject, Cause::SourceUnavailable))?
                .read_member(locator, 1 << 30)
                .map_err(|cause| unavailable(subject, cause))?;
            let snapshot = crate::storage::data::symbol_page_readmission::readmit(
                execution, &bytes, locator, subject,
            )?;
            let shape =
                SymbolPageShape::read(&snapshot).map_err(|cause| unavailable(subject, cause))?;
            shape
                .validate(selection)
                .map_err(|cause| unavailable(subject, cause))?;
            // Admit the complete map path before importing any dictionary text.
            // A refused update drops its partial path and leaves the cache intact.
            let mut admitted = pages.data.clone();
            admitted
                .insert_for_request(locator, Ok(shape), execution)
                .map_err(Failure::Request)?;
            crate::execution::request_custody::check_request(execution)
                .map_err(Failure::Request)?;
            owner
                .import_canonical_snapshot(&snapshot)
                .map_err(|_| unavailable(subject, Cause::Damaged))?;
            pages.data = admitted;
            Ok(())
        })();
        match &result {
            Ok(()) => {}
            Err(Failure::StorageUnavailable(failure)) if failure.cause.is_proven_damage() => {
                let mut refused = pages.data.clone();
                refused
                    .insert_for_request(locator, Err(*failure), execution)
                    .map_err(Failure::Request)?;
                pages.data = refused;
            }
            Err(Failure::StorageUnavailable(_)) => {}
            Err(Failure::Request(_) | Failure::Content(_)) => {}
        }
        result
    }
}

fn unavailable(subject: RelationalStorageSubject, cause: Cause) -> RelationalStorageUnavailable {
    RelationalStorageUnavailable { subject, cause }
}
