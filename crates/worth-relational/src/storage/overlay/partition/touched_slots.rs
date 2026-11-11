//! The numeric touched-slot journal, sharing only its actual AVL owners.
use crate::execution::RelationalExecutionDenialCause;
use crate::storage::substrate::SharedMap;
use worth_execution::ExecutionRequest;

#[derive(Debug, Clone, Default)]
pub(crate) struct TouchedSlots {
    slots: SharedMap<usize, ()>,
}

impl TouchedSlots {
    pub(crate) fn len(&self) -> usize {
        self.slots.len()
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }
    pub(crate) fn contains(&self, slot: &usize) -> bool {
        self.slots.contains_key(slot)
    }
    pub(crate) fn iter(&self) -> TouchedSlotsIter<'_> {
        TouchedSlotsIter(self.slots.iter())
    }
    /// Existing ordinary fixture/image construction; no request claim.
    pub(crate) fn insert(&mut self, slot: usize) -> bool {
        if self.contains(&slot) {
            return false;
        }
        self.slots.insert(slot, ());
        true
    }
    pub(crate) fn insert_for_request(
        &mut self,
        slot: usize,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<bool, RelationalExecutionDenialCause> {
        request.consult()?;
        if self.contains(&slot) {
            return Ok(false);
        }
        self.slots.insert_for_request(slot, (), request)?;
        Ok(true)
    }
}
impl FromIterator<usize> for TouchedSlots {
    fn from_iter<T: IntoIterator<Item = usize>>(values: T) -> Self {
        let mut result = Self::default();
        result.extend(values);
        result
    }
}
impl Extend<usize> for TouchedSlots {
    fn extend<T: IntoIterator<Item = usize>>(&mut self, values: T) {
        for slot in values {
            self.insert(slot);
        }
    }
}

pub(crate) struct TouchedSlotsIter<'a>(crate::storage::substrate::SharedMapIter<'a, usize, ()>);
impl<'a> Iterator for TouchedSlotsIter<'a> {
    type Item = &'a usize;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(slot, ())| slot)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
impl ExactSizeIterator for TouchedSlotsIter<'_> {}
impl<'a> IntoIterator for &'a TouchedSlots {
    type Item = &'a usize;
    type IntoIter = TouchedSlotsIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<const N: usize> From<[usize; N]> for TouchedSlots {
    fn from(values: [usize; N]) -> Self {
        values.into_iter().collect()
    }
}
