//! Exact published membership within a fixed 256-identity page range.
use crate::durability::medium::RelationalDurableMemberLocator;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RelationalCheckpointPage {
    pub(crate) first: u64,
    pub(crate) last: u64,
    pub(crate) membership: [u64; 4],
    pub(crate) member: RelationalDurableMemberLocator,
}
impl RelationalCheckpointPage {
    pub(crate) fn range_of(id: u64) -> u64 {
        id.saturating_sub(1) / 256
    }
    pub(crate) fn range(&self) -> u64 {
        Self::range_of(self.first)
    }
    pub(crate) fn from_ids(
        ids: impl IntoIterator<Item = u64>,
        member: RelationalDurableMemberLocator,
    ) -> Option<Self> {
        let mut ids = ids.into_iter();
        let first = ids.next()?;
        if first == 0 || first >= u64::MAX - 1 {
            return None;
        }
        let mut page = Self {
            first,
            last: first,
            membership: [0; 4],
            member,
        };
        for id in std::iter::once(first).chain(ids) {
            if id < page.last || id == 0 || id >= u64::MAX - 1 || Self::range_of(id) != page.range()
            {
                return None;
            }
            let offset = (id - 1) % 256;
            let mask = 1 << (offset % 64);
            let word = &mut page.membership[(offset / 64) as usize];
            if *word & mask != 0 {
                return None;
            }
            *word |= mask;
            page.last = id;
        }
        Some(page)
    }
    pub(crate) fn contains(&self, id: u64) -> bool {
        id != 0
            && Self::range_of(id) == self.range()
            && self.membership[(((id - 1) % 256) / 64) as usize] & (1 << ((id - 1) % 64)) != 0
    }
    pub(crate) fn find(pages: &[Self], id: u64) -> Option<&Self> {
        let range = Self::range_of(id);
        let start = pages.partition_point(|page| page.range() < range);
        pages[start..]
            .iter()
            .take_while(|page| page.range() == range)
            .find(|page| page.contains(id))
    }
    pub(crate) fn valid(&self) -> bool {
        if self.first == 0
            || self.last < self.first
            || self.last >= u64::MAX - 1
            || Self::range_of(self.last) != self.range()
        {
            return false;
        }
        let base = u128::from(self.range()) * 256;
        let first = self
            .membership
            .iter()
            .enumerate()
            .find(|(_, word)| **word != 0)
            .map(|(i, word)| base + i as u128 * 64 + u128::from(word.trailing_zeros()) + 1);
        let last = self
            .membership
            .iter()
            .enumerate()
            .rfind(|(_, word)| **word != 0)
            .map(|(i, word)| base + i as u128 * 64 + 64 - u128::from(word.leading_zeros()));
        first == Some(u128::from(self.first)) && last == Some(u128::from(self.last))
    }
}
