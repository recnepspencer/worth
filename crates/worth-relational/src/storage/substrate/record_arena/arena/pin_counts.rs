//! Runtime retention counters are sparse metadata, not record truth columns.
use crate::storage::substrate::SharedMap;

#[derive(Debug, Clone)]
pub(crate) struct RecordPinCounts {
    len: usize,
    values: SharedMap<usize, u32>,
}

impl RecordPinCounts {
    pub(super) fn new() -> Self {
        Self {
            len: 0,
            values: SharedMap::new(),
        }
    }
    pub(super) fn push(&mut self, value: u32) {
        if value != 0 {
            self.values.insert(self.len, value);
        }
        self.len += 1;
    }
    pub(crate) fn get(&self, index: usize) -> Option<&u32> {
        (index < self.len).then(|| self.values.get(&index).unwrap_or(&0))
    }
    pub(crate) fn set(&mut self, index: usize, value: u32) {
        assert!(index < self.len, "pin row is absent");
        if value == 0 {
            self.values.remove(&index);
        } else {
            self.values.insert(index, value);
        }
    }
    pub(super) fn adjust(&mut self, index: usize, delta: i32) -> Option<()> {
        let value = self.get(index)?.saturating_add_signed(delta);
        self.set(index, value);
        Some(())
    }
    pub(super) fn remap_from(&mut self, source: &Self, map: impl Fn(usize) -> Option<usize>) {
        self.fill(0);
        for (&physical, &count) in &source.values {
            if let Some(destination) = map(physical) {
                self.set(destination, count);
            }
        }
    }
    pub(super) fn fill(&mut self, value: u32) {
        assert_eq!(value, 0, "retention counters only bulk-reset to zero");
        self.values = SharedMap::new();
    }
    pub(crate) fn into_vec(self) -> Vec<u32> {
        (0..self.len)
            .map(|index| *self.get(index).unwrap())
            .collect()
    }
    pub(super) fn allocation_bytes(&self) -> u64 {
        self.values.allocation_bytes()
    }

    pub(super) fn visit_allocations(
        &self,
        unique: bool,
        visitor: &mut dyn crate::storage::substrate::StorageAllocationVisitor,
    ) {
        self.values.visit_allocations(
            unique,
            visitor,
            &mut crate::storage::substrate::visit_inline_value,
        );
    }
}

impl From<Vec<u32>> for RecordPinCounts {
    fn from(values: Vec<u32>) -> Self {
        let mut counts = Self::new();
        for value in values {
            counts.push(value);
        }
        counts
    }
}

impl std::ops::Index<usize> for RecordPinCounts {
    type Output = u32;
    fn index(&self, index: usize) -> &u32 {
        self.get(index).expect("pin row is absent")
    }
}
