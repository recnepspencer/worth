//! Lazy reader over a checkpoint's paged commit index.
//!
//! Opening reads nothing. Every page read goes through a source confined to
//! the index's own page roster and lands in a page cache enrolled in the
//! working set's eviction group, so lookups fault only the pages they touch
//! and resident pages evict under the historical pool's budget.
use std::sync::{Arc, OnceLock};

use worth_relational_contracts::internal_port::config::data::residency_budget::ResidencyBudget;
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use worth_relational_storage::internal_port::storage::residency::chunked::cold_member_port::{
    ColdMemberSource, ConfinedMemberSource,
};
use worth_relational_storage::internal_port::storage::residency::chunked::page_cache::{
    PageCacheObservation, PageEvictionGroup, ResidentPageCache,
};
use worth_relational_storage::internal_port::storage::residency::working_set::{
    ResidencyPoolKind, WorkingSetAccountant,
};

use super::page::ResidentCommitPage;
use super::{CheckpointCommitIndex, COMMIT_PAGE_BYTES};
use crate::durability::medium::RelationalDurableColdSource;
use crate::history::commit::index_entry::RelationalCommitIndexEntry as Entry;
use crate::history::data::CommitId;
use crate::identity::data::VersionId;
use crate::publication::patch::data::PatchStreamPosition;

pub struct ColdCommitIndex {
    index: Arc<CheckpointCommitIndex>,
    /// Prefix maxima and suffix minima of the page id ranges: the pages that
    /// can hold an id form one contiguous run found by two binary searches.
    commit_bounds: (Vec<u64>, Vec<u64>),
    version_bounds: (Vec<u64>, Vec<u64>),
    source: ConfinedMemberSource<ColdMemberSource>,
    group: Arc<PageEvictionGroup>,
    accountant: OnceLock<Arc<WorkingSetAccountant>>,
    cache: OnceLock<Arc<ResidentPageCache<ResidentCommitPage>>>,
}

impl std::fmt::Debug for ColdCommitIndex {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ColdCommitIndex")
            .field("count", &self.index.count)
            .field("pages", &self.index.pages.len())
            .finish_non_exhaustive()
    }
}

fn bounds(pages: impl Iterator<Item = [u64; 2]>) -> (Vec<u64>, Vec<u64>) {
    let pages = pages.collect::<Vec<_>>();
    let mut high = Vec::with_capacity(pages.len());
    let mut running = 0;
    for &[_, max] in &pages {
        running = running.max(max);
        high.push(running);
    }
    let mut low = vec![u64::MAX; pages.len()];
    let mut running = u64::MAX;
    for (slot, &[min, _]) in low.iter_mut().zip(&pages).rev() {
        running = running.min(min);
        *slot = running;
    }
    (high, low)
}

impl ColdCommitIndex {
    /// Opens `index` over `source` without reading any page.
    pub fn open(
        index: Arc<CheckpointCommitIndex>,
        source: Arc<dyn RelationalDurableColdSource>,
    ) -> Self {
        let roster = index.page_members().collect();
        Self::confined(
            index,
            ColdMemberSource::new(source, COMMIT_PAGE_BYTES as u64),
            roster,
        )
    }

    /// Opens `index` with reads confined to `roster`, the page members its
    /// checkpoint admitted; a descriptor naming any other member is refused.
    pub fn confined(
        index: Arc<CheckpointCommitIndex>,
        source: ColdMemberSource,
        roster: Vec<crate::durability::medium::RelationalDurableMemberLocator>,
    ) -> Self {
        Self {
            commit_bounds: bounds(index.pages.iter().map(|page| page.commits)),
            version_bounds: bounds(index.pages.iter().map(|page| page.versions)),
            source: ConfinedMemberSource::new(source, roster),
            group: PageEvictionGroup::new(),
            accountant: OnceLock::new(),
            cache: OnceLock::new(),
            index,
        }
    }

    /// Charges later faults to `accountant`; only effective before the first
    /// fault, after which pages already hold their charges.
    pub fn bind_working_set(&self, accountant: Arc<WorkingSetAccountant>) -> bool {
        self.cache.get().is_none() && self.accountant.set(accountant).is_ok()
    }

    pub fn descriptor(&self) -> &Arc<CheckpointCommitIndex> {
        &self.index
    }

    pub fn count(&self) -> u64 {
        self.index.count
    }

    pub fn observation(&self) -> PageCacheObservation {
        self.cache
            .get()
            .map(|cache| cache.observation())
            .unwrap_or_default()
    }

    fn cache(&self) -> &Arc<ResidentPageCache<ResidentCommitPage>> {
        self.cache.get_or_init(|| {
            let accountant = self.accountant.get().cloned().unwrap_or_else(|| {
                WorkingSetAccountant::new(ResidencyBudget {
                    head_working_set_bytes: u64::MAX,
                    historical_working_set_bytes: u64::MAX,
                    max_in_flight_fill_bytes: u64::MAX,
                    streaming_window_bytes: u32::MAX,
                })
            });
            self.group.cache(accountant, ResidencyPoolKind::Historical)
        })
    }

    /// Page `ordinal`, faulted on a miss and checked against its descriptor.
    pub fn page(&self, ordinal: usize) -> Result<Arc<ResidentCommitPage>, Cause> {
        let reference = self.index.pages.get(ordinal).ok_or(Cause::Damaged)?;
        self.cache()
            .page_checked(&self.source, reference.member, |page| {
                page.admit(reference, &self.index)
            })
    }

    /// Reads, decodes, and admits page `ordinal` without inserting it into the
    /// resident page cache, for single-pass streaming readers.
    pub fn read_page_uncached(&self, ordinal: usize) -> Result<ResidentCommitPage, Cause> {
        use worth_relational_storage::internal_port::storage::residency::chunked::{
            member_io::ChunkMemberSource as _, page_cache::ResidentPage as _,
        };
        let reference = self.index.pages.get(ordinal).ok_or(Cause::Damaged)?;
        let length =
            usize::try_from(reference.member.length).map_err(|_| Cause::MemoryExhausted)?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(length)
            .map_err(|_| Cause::MemoryExhausted)?;
        bytes.resize(length, 0);
        let mut filled = 0;
        while filled < length {
            match self
                .source
                .read_range(reference.member, filled as u64, &mut bytes[filled..])?
            {
                0 => return Err(Cause::MissingMember),
                read => filled += read,
            }
        }
        let page = ResidentCommitPage::decode(&bytes, reference.member)?;
        page.admit(reference, &self.index)?;
        Ok(page)
    }

    pub fn at_position(&self, position: PatchStreamPosition) -> Result<Option<Entry>, Cause> {
        if let Some(inline) = self.index.inline_at(position.0) {
            return Ok(Some(inline.clone()));
        }
        let Some(ordinal) = self.index.page_for(position) else {
            return Ok(None);
        };
        Ok(self.page(ordinal)?.row(position.0).cloned())
    }

    pub fn by_commit(&self, commit: CommitId) -> Result<Option<Entry>, Cause> {
        if let Some(inline) = self.index.head(commit).or(self
            .index
            .tail
            .as_ref()
            .filter(|tail| tail.commit.commit_id == commit))
        {
            return Ok(Some(inline.clone()));
        }
        self.find(
            commit.0,
            &self.commit_bounds,
            |page| page.commits,
            |row| row.commit.commit_id == commit,
        )
    }

    pub fn by_version(&self, version: VersionId) -> Result<Option<Entry>, Cause> {
        if let Some(inline) = self
            .index
            .heads
            .iter()
            .find(|entry| entry.commit.version_id == version)
            .or(self
                .index
                .tail
                .as_ref()
                .filter(|tail| tail.commit.version_id == version))
        {
            return Ok(Some(inline.clone()));
        }
        self.find(
            version.0,
            &self.version_bounds,
            |page| page.versions,
            |row| row.commit.version_id == version,
        )
    }

    fn find(
        &self,
        key: u64,
        (high, low): &(Vec<u64>, Vec<u64>),
        range: fn(&super::CommitIndexPageRef) -> [u64; 2],
        matches: impl Fn(&Entry) -> bool,
    ) -> Result<Option<Entry>, Cause> {
        let start = high.partition_point(|max| *max < key);
        let end = low.partition_point(|min| *min <= key);
        for ordinal in start..end.max(start) {
            let [min, max] = range(&self.index.pages[ordinal]);
            if key < min || key > max {
                continue;
            }
            if let Some(row) = self
                .page(ordinal)?
                .page
                .rows
                .iter()
                .find(|row| matches(row))
            {
                return Ok(Some(row.clone()));
            }
        }
        Ok(None)
    }

    /// The largest commit id in the index, read from descriptors alone.
    pub fn max_commit(&self) -> Option<CommitId> {
        self.commit_bounds.0.last().copied().map(CommitId)
    }

    pub fn max_version(&self) -> Option<VersionId> {
        self.version_bounds.0.last().copied().map(VersionId)
    }

    /// Up to `limit` entries after `after`, faulting only the pages they span.
    pub fn after(&self, after: u64, limit: usize) -> Result<Vec<Entry>, Cause> {
        let mut rows = Vec::new();
        let mut position = after.saturating_add(1);
        while rows.len() < limit && position <= self.index.count {
            let ordinal = self
                .index
                .page_for(PatchStreamPosition(position))
                .ok_or(Cause::Damaged)?;
            let page = self.page(ordinal)?;
            while rows.len() < limit
                && let Some(row) = page.row(position)
            {
                rows.push(row.clone());
                position += 1;
            }
        }
        Ok(rows)
    }

    /// Visits every entry in position order, one page resident at a time.
    pub fn try_for_each<E: From<Cause>>(
        &self,
        mut visit: impl FnMut(&Entry) -> Result<(), E>,
    ) -> Result<(), E> {
        for ordinal in 0..self.index.pages.len() {
            let page = self.page(ordinal)?;
            for row in &page.page.rows {
                visit(row)?;
            }
        }
        Ok(())
    }
}
