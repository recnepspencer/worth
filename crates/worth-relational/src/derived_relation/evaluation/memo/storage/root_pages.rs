//! Extent-local node coordinates use sparse pages and bounded direct slots.
use super::*;
pub(super) const PAGE_NODES: usize = usize::BITS as usize;
#[derive(Default)]
pub(super) struct RootPages {
    pub(super) pages: BTreeMap<usize, Page>,
}
#[derive(Default)]
pub(super) struct Page {
    pub(super) slots: Vec<Option<Box<Bucket>>>,
}
pub(super) fn address(key: &Key) -> (usize, usize) {
    (
        key.node / PAGE_NODES,
        (key.node % PAGE_NODES) * 2 + usize::from(key.reverse),
    )
}
impl RootPages {
    pub(super) fn get<'a>(
        &'a self,
        key: &Key,
        read: &mut impl SourceRead,
    ) -> Result<Option<&'a Endpoints>, Denial> {
        let (page, slot) = address(key);
        charge(read, size_of::<usize>(), self.pages.len())?;
        let Some(page) = self.pages.get(&page) else {
            return Ok(None);
        };
        read.checkpoint(1)?;
        match page.slots.get(slot).and_then(Option::as_deref) {
            Some(bucket) => bucket.get(key, read),
            None => Ok(None),
        }
    }
    pub(super) fn remove(
        &mut self,
        key: &Key,
        read: &mut impl SourceRead,
    ) -> Result<Option<Endpoints>, Denial> {
        let (page, slot) = address(key);
        charge(read, size_of::<usize>(), self.pages.len())?;
        let Some(page) = self.pages.get_mut(&page) else {
            return Ok(None);
        };
        read.checkpoint(1)?;
        match page.slots.get_mut(slot).and_then(Option::as_deref_mut) {
            Some(bucket) => bucket.remove(key, read),
            None => Ok(None),
        }
    }
    pub(super) fn insert(
        &mut self,
        key: Key,
        values: Endpoints,
        read: &mut impl SourceRead,
    ) -> Result<Option<Endpoints>, Denial> {
        let (page, slot) = address(&key);
        charge(read, size_of::<usize>(), self.pages.len())?;
        let page = match self.pages.entry(page) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                read.retain(size_of::<(usize, Page)>() + 4 * size_of::<usize>())?;
                entry.insert(Page::default())
            }
        };
        read.checkpoint(1)?;
        let next = slot + 1; // remainder/direction bound this by2*PAGE_NODES.
        if next > page.slots.len() {
            let growth = next - page.slots.len();
            if next > page.slots.capacity() {
                let bytes = next
                    .checked_mul(size_of::<Option<Box<Bucket>>>())
                    .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
                read.retain(bytes)?;
                read.checkpoint(
                    page.slots
                        .len()
                        .checked_mul(size_of::<Option<Box<Bucket>>>())
                        .ok_or(DefinitionDenial::WorkBudgetExceeded)?,
                )?;
                page.slots.try_reserve_exact(growth).map_err(|_| {
                    Denial::Request(
                        crate::execution::RelationalExecutionDenialCause::AllocationFailed,
                    )
                })?;
            }
            read.checkpoint(growth)?;
            page.slots.resize_with(next, || None);
        }
        if page.slots[slot].is_none() {
            read.retain(size_of::<Bucket>())?;
            page.slots[slot] = Some(Box::new(Bucket::default()));
        }
        page.slots[slot]
            .as_mut()
            .expect("actual funded slot")
            .insert(key, values, read)
    }
}
