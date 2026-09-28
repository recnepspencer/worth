use worth_store_physical_format::{
    decode_extent_chunk, inspect_inline_page, ExtentChunkCoordinate, PageGenerationCell,
    PersistedPhysicalDataFrameSubject, PhysicalRecordFormatDeclaration, RecordArtifactFile,
    RecordFrameCoordinate,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum PhysicalDataFrameKind {
    InlinePage = 1,
    ExtentChunk = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PhysicalDataFrameSubject {
    InlinePage(PageGenerationCell),
    ExtentChunk(ExtentChunkCoordinate),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysicalDataFrameIdentity {
    subject: PhysicalDataFrameSubject,
    coordinate: RecordFrameCoordinate,
}

impl PhysicalDataFrameIdentity {
    pub(in crate::physical_runtime) fn inline_page(
        page: PageGenerationCell,
        artifact: RecordArtifactFile,
        offset: u64,
        length: u32,
    ) -> Option<Self> {
        let RecordArtifactFile::Segment { segment, .. } = artifact else {
            return None;
        };
        (segment == page.segment_id().get()).then_some(Self {
            subject: PhysicalDataFrameSubject::InlinePage(page),
            coordinate: RecordFrameCoordinate::new(artifact, offset, length)?,
        })
    }

    pub(in crate::physical_runtime) fn extent_chunk(
        chunk: ExtentChunkCoordinate,
        artifact: RecordArtifactFile,
        offset: u64,
        length: u32,
        range: worth_store_physical_format::ExtentArenaRange,
    ) -> Option<Self> {
        let RecordArtifactFile::ExtentArena { arena } = artifact else {
            return None;
        };
        (arena == range.arena().get()
            && offset >= range.offset()
            && offset.checked_add(u64::from(length))? <= range.end())
        .then_some(Self {
            subject: PhysicalDataFrameSubject::ExtentChunk(chunk),
            coordinate: RecordFrameCoordinate::new(artifact, offset, length)?,
        })
    }

    pub const fn kind(self) -> PhysicalDataFrameKind {
        match self.subject {
            PhysicalDataFrameSubject::InlinePage(_) => PhysicalDataFrameKind::InlinePage,
            PhysicalDataFrameSubject::ExtentChunk(_) => PhysicalDataFrameKind::ExtentChunk,
        }
    }

    pub const fn subject(self) -> PhysicalDataFrameSubject {
        self.subject
    }

    pub const fn coordinate(self) -> RecordFrameCoordinate {
        self.coordinate
    }

    pub(in crate::physical_runtime) fn admits_bytes(
        self,
        format: PhysicalRecordFormatDeclaration,
        bytes: &[u8],
    ) -> bool {
        if bytes.len() != self.coordinate.length() as usize {
            return false;
        }
        match self.subject {
            PhysicalDataFrameSubject::InlinePage(page) => inspect_inline_page(format, bytes)
                .is_ok_and(|geometry| geometry.page_cell() == page),
            PhysicalDataFrameSubject::ExtentChunk(chunk) => decode_extent_chunk(bytes, chunk)
                .is_ok_and(|(_, found_format)| found_format == format),
        }
    }

    pub(in crate::physical_runtime) fn is_exact_successor_of(self, source: Self) -> bool {
        match (source.subject, self.subject) {
            (
                PhysicalDataFrameSubject::InlinePage(source_page),
                PhysicalDataFrameSubject::InlinePage(target_page),
            ) => {
                source_page.segment_id() == target_page.segment_id()
                    && source_page.page_id() == target_page.page_id()
                    && source_page.generation().get().checked_add(1)
                        == Some(target_page.generation().get())
                    && self.coordinate.length() == source.coordinate.length()
                    && artifact_is_exact_successor(
                        source.coordinate.artifact(),
                        self.coordinate.artifact(),
                    )
            }
            _ => false,
        }
    }

    pub(in crate::physical_runtime) fn write_canonical(self, target: &mut Vec<u8>) {
        worth_store_physical_format::write_persisted_physical_data_frame_identity(
            self.persisted_subject(),
            self.coordinate,
            target,
        );
    }

    pub(in crate::physical_runtime) const fn persisted_subject(
        self,
    ) -> PersistedPhysicalDataFrameSubject {
        match self.subject {
            PhysicalDataFrameSubject::InlinePage(page) => {
                PersistedPhysicalDataFrameSubject::InlinePage(page)
            }
            PhysicalDataFrameSubject::ExtentChunk(chunk) => {
                PersistedPhysicalDataFrameSubject::ExtentChunk(chunk)
            }
        }
    }
}

fn artifact_is_exact_successor(source: RecordArtifactFile, target: RecordArtifactFile) -> bool {
    match (source, target) {
        (
            RecordArtifactFile::Segment {
                segment: source_segment,
                generation: source_generation,
            },
            RecordArtifactFile::Segment {
                segment: target_segment,
                generation: target_generation,
            },
        ) => {
            source_segment == target_segment
                && source_generation.checked_add(1) == Some(target_generation)
        }
        _ => false,
    }
}
