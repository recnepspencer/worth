use crate::diagnostics::state::DiagnosticHistory;

use serde::{Deserialize, Serialize};

use super::record::LineageRecord;

#[derive(Debug, Clone, Copy, Default)]
pub struct RetainedLineageView<'a> {
    records: Option<&'a DiagnosticHistory<LineageRecord>>,
    offset: usize,
    len: usize,
}

impl<'a> RetainedLineageView<'a> {
    pub(crate) fn new(
        records: &'a DiagnosticHistory<LineageRecord>,
        offset: usize,
        len: usize,
    ) -> Self {
        Self {
            records: Some(records),
            offset,
            len,
        }
    }

    pub fn empty() -> Self {
        Self {
            records: None,
            offset: 0,
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &'a LineageRecord> + ExactSizeIterator {
        self.records
            .map(DiagnosticHistory::iter)
            .unwrap_or_default()
            .skip(self.offset)
            .take(self.len)
    }

    pub fn first(&self) -> Option<&'a LineageRecord> {
        self.iter().next()
    }

    pub fn last(&self) -> Option<&'a LineageRecord> {
        self.iter().next_back()
    }

    pub fn to_owned_records(&self) -> Vec<LineageRecord> {
        self.iter().cloned().collect()
    }
}

impl<'a> PartialEq for RetainedLineageView<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<'a> Eq for RetainedLineageView<'a> {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SynthesizedLineageChain {
    records: Vec<LineageRecord>,
}

impl SynthesizedLineageChain {
    pub fn new(records: Vec<LineageRecord>) -> Self {
        Self { records }
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &LineageRecord> {
        self.records.iter()
    }

    pub fn first(&self) -> Option<&LineageRecord> {
        self.records.first()
    }

    pub fn last(&self) -> Option<&LineageRecord> {
        self.records.last()
    }

    pub fn to_owned_records(&self) -> Vec<LineageRecord> {
        self.records.clone()
    }
}
