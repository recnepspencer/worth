//! Producer-side spilling of one slot's sequence into continuation pages.
//!
//! The spill buffers at most one page of entries. When the next entry would
//! overflow a page, the buffer is written as the next page. At the end of the
//! sequence a remainder within the inline budget stays inline; a larger
//! remainder becomes the final page. Sealing memory therefore stays bounded
//! by one page per open sequence, however long the slot's history is.
use super::super::layout::ChunkLayout;
use super::super::member_frame::{encode_member, ChunkMemberKind};
use super::super::member_io::ChunkMemberSink;
use super::super::paged_directory::{
    DirectoryBuildError, PagedDirectoryBuilder, PagedDirectoryRoot,
};
use super::super::slot_projection::{AdjacencyEdge, SlotVersion};
use super::{
    version_bytes, ContinuationEntries, ContinuationKey, ContinuationPage, ContinuationPageRef,
    ContinuationStream, EDGE_BYTES, MAX_CONTINUATION_DIRECTORY_ENTRY_BYTES, PAGE_FRAME_SLACK,
};

/// An entry kind that a continuation page can carry.
pub trait SpillEntry: Sized {
    const STREAM: ContinuationStream;
    fn encoded_bytes(&self) -> usize;
    fn into_entries(entries: Vec<Self>) -> ContinuationEntries;
}

impl SpillEntry for SlotVersion {
    const STREAM: ContinuationStream = ContinuationStream::History;

    fn encoded_bytes(&self) -> usize {
        version_bytes(self)
    }

    fn into_entries(entries: Vec<Self>) -> ContinuationEntries {
        ContinuationEntries::History(entries)
    }
}

impl SpillEntry for AdjacencyEdge {
    const STREAM: ContinuationStream = ContinuationStream::Adjacency;

    fn encoded_bytes(&self) -> usize {
        EDGE_BYTES
    }

    fn into_entries(entries: Vec<Self>) -> ContinuationEntries {
        ContinuationEntries::Adjacency(entries)
    }
}

/// Writes one chunk's continuation pages and indexes them, in key order, in
/// the chunk's continuation directory.
pub struct ContinuationWriter {
    layout: ChunkLayout,
    directory: PagedDirectoryBuilder<ContinuationKey, ContinuationPageRef>,
    pages_written: u64,
}

impl ContinuationWriter {
    pub fn new(layout: ChunkLayout) -> Result<Self, DirectoryBuildError> {
        let directory_page_limit = layout
            .directory_page_entries
            .saturating_mul(MAX_CONTINUATION_DIRECTORY_ENTRY_BYTES)
            .max(layout.continuation_page_bytes)
            .saturating_add(PAGE_FRAME_SLACK);
        Ok(Self {
            layout,
            directory: PagedDirectoryBuilder::new(
                layout.directory_page_entries,
                directory_page_limit,
            )?,
            pages_written: 0,
        })
    }

    pub fn pages_written(&self) -> u64 {
        self.pages_written
    }

    /// The continuation directory root, or `None` when nothing spilled.
    pub fn finish(
        self,
        sink: &mut dyn ChunkMemberSink,
    ) -> Result<Option<PagedDirectoryRoot>, DirectoryBuildError> {
        self.directory.finish(sink)
    }

    fn write_page(
        &mut self,
        sink: &mut dyn ChunkMemberSink,
        key: ContinuationKey,
        entries: ContinuationEntries,
    ) -> Result<(), DirectoryBuildError> {
        let count = u32::try_from(entries.len()).map_err(|_| {
            DirectoryBuildError::Member(
                crate::storage::data::RelationalStorageUnavailableCause::Damaged,
            )
        })?;
        let page = ContinuationPage { key, entries };
        let member = encode_member(
            ChunkMemberKind::Continuation,
            &page,
            self.layout.continuation_page_bytes + PAGE_FRAME_SLACK,
        )?;
        let reference = ContinuationPageRef {
            page: member.locator,
            entries: count,
        };
        sink.put(member.locator, member.bytes)?;
        self.directory.push(sink, key, reference)?;
        self.pages_written += 1;
        Ok(())
    }
}

/// The open spill of one slot's sequence.
pub struct SlotSpill<T: SpillEntry> {
    slot: u64,
    pending: Vec<T>,
    pending_bytes: usize,
    next_sequence: u32,
}

impl<T: SpillEntry> SlotSpill<T> {
    pub fn new(slot: u64) -> Self {
        Self {
            slot,
            pending: Vec::new(),
            pending_bytes: 0,
            next_sequence: 0,
        }
    }

    pub fn push(
        &mut self,
        writer: &mut ContinuationWriter,
        sink: &mut dyn ChunkMemberSink,
        entry: T,
    ) -> Result<(), DirectoryBuildError> {
        let bytes = entry.encoded_bytes();
        if !self.pending.is_empty()
            && self.pending_bytes + bytes > writer.layout.continuation_page_bytes
        {
            self.emit(writer, sink)?;
        }
        self.pending.push(entry);
        self.pending_bytes += bytes;
        Ok(())
    }

    /// Closes the sequence and returns the entries that stay inline.
    pub fn finish(
        mut self,
        writer: &mut ContinuationWriter,
        sink: &mut dyn ChunkMemberSink,
    ) -> Result<Vec<T>, DirectoryBuildError> {
        if self.pending_bytes > writer.layout.slot_inline_budget {
            self.emit(writer, sink)?;
        }
        Ok(self.pending)
    }

    fn emit(
        &mut self,
        writer: &mut ContinuationWriter,
        sink: &mut dyn ChunkMemberSink,
    ) -> Result<(), DirectoryBuildError> {
        let key = ContinuationKey {
            slot: self.slot,
            stream: T::STREAM,
            sequence: self.next_sequence,
        };
        let entries = T::into_entries(std::mem::take(&mut self.pending));
        self.pending_bytes = 0;
        self.next_sequence += 1;
        writer.write_page(sink, key, entries)
    }
}
