use std::cell::Cell;
use std::io::{self, Write};

use serde::Serialize;

use crate::durability::data::NativeCheckpointSectionBytes;

#[derive(Clone, Copy)]
pub(super) enum NativeSection {
    Envelopes,
    BranchRoots,
    BranchCells,
    PartitionMirror,
    DerivedIndexes,
}

#[derive(Default)]
pub(crate) struct CaptureSectionRecorder {
    written: Cell<usize>,
    envelopes: Cell<usize>,
    branch_roots: Cell<usize>,
    branch_cells: Cell<usize>,
    partition_mirror: Cell<usize>,
    derived_indexes: Cell<usize>,
}

impl CaptureSectionRecorder {
    pub(crate) fn writer<'a>(&'a self, bytes: &'a mut Vec<u8>) -> CountingWriter<'a> {
        CountingWriter {
            bytes,
            recorder: self,
        }
    }

    fn record(&self, section: NativeSection, bytes: usize) {
        let counter = match section {
            NativeSection::Envelopes => &self.envelopes,
            NativeSection::BranchRoots => &self.branch_roots,
            NativeSection::BranchCells => &self.branch_cells,
            NativeSection::PartitionMirror => &self.partition_mirror,
            NativeSection::DerivedIndexes => &self.derived_indexes,
        };
        counter.set(counter.get() + bytes);
    }

    pub(crate) fn finish(&self, total: usize) -> Option<NativeCheckpointSectionBytes> {
        if self.written.get() != total {
            return None;
        }
        let classified = [
            self.envelopes.get(),
            self.branch_roots.get(),
            self.branch_cells.get(),
            self.partition_mirror.get(),
            self.derived_indexes.get(),
        ]
        .into_iter()
        .try_fold(0usize, usize::checked_add)?;
        Some(NativeCheckpointSectionBytes {
            total,
            envelopes: self.envelopes.get(),
            branch_roots: self.branch_roots.get(),
            branch_cells: self.branch_cells.get(),
            partition_mirror: self.partition_mirror.get(),
            derived_indexes: self.derived_indexes.get(),
            framing_and_metadata: total.checked_sub(classified)?,
        })
    }
}

pub(super) struct MeasuredValue<'a, T: ?Sized> {
    pub(super) value: &'a T,
    pub(super) section: NativeSection,
    pub(super) recorder: Option<&'a CaptureSectionRecorder>,
}

impl<T: Serialize + ?Sized> Serialize for MeasuredValue<'_, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let before = self.recorder.map(|recorder| recorder.written.get());
        let result = self.value.serialize(serializer)?;
        if let (Some(recorder), Some(before)) = (self.recorder, before) {
            recorder.record(self.section, recorder.written.get() - before);
        }
        Ok(result)
    }
}

pub(crate) struct CountingWriter<'a> {
    bytes: &'a mut Vec<u8>,
    recorder: &'a CaptureSectionRecorder,
}

impl Write for CountingWriter<'_> {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        let written = self.bytes.write(input)?;
        self.recorder
            .written
            .set(self.recorder.written.get() + written);
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.bytes.flush()
    }
}
