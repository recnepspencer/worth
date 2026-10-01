use super::*;

impl PersistedPhysicalRecoveryFrame {
    pub fn new(
        subject: PersistedPhysicalDataFrameSubject,
        coordinate: RecordFrameCoordinate,
        bytes: &[u8],
    ) -> Option<Self> {
        (bytes.len() == coordinate.length() as usize && subject_matches(subject, coordinate))
            .then_some(Self {
                subject,
                coordinate,
                bytes: bytes.into(),
            })
    }
    pub const fn subject(&self) -> PersistedPhysicalDataFrameSubject {
        self.subject
    }
    pub const fn coordinate(&self) -> RecordFrameCoordinate {
        self.coordinate
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl PersistedPhysicalRecoveryManifest {
    pub fn new(coordinate: RecordFrameCoordinate, bytes: &[u8]) -> Option<Self> {
        (matches!(
            coordinate.artifact(),
            RecordArtifactFile::ExtentArena { .. }
        ) && coordinate.length() as usize == bytes.len())
        .then_some(Self {
            coordinate,
            bytes: bytes.into(),
        })
    }
    pub const fn artifact(&self) -> RecordArtifactFile {
        self.coordinate.artifact()
    }
    pub const fn coordinate(&self) -> RecordFrameCoordinate {
        self.coordinate
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

fn subject_matches(
    subject: PersistedPhysicalDataFrameSubject,
    coordinate: RecordFrameCoordinate,
) -> bool {
    match (subject, coordinate.artifact()) {
        (
            PersistedPhysicalDataFrameSubject::InlinePage(page),
            RecordArtifactFile::Segment {
                segment,
                generation: _,
            },
        ) => page.segment_id().get() == segment,
        (
            PersistedPhysicalDataFrameSubject::ExtentChunk(_),
            RecordArtifactFile::ExtentArena { arena },
        ) => arena != 0,
        _ => false,
    }
}
