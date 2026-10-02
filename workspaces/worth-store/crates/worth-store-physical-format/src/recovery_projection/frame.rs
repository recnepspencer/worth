use super::*;

impl PersistedPhysicalRecoveryFrame {
    pub fn new(
        subject: PersistedPhysicalDataFrameSubject,
        coordinate: RecordFrameCoordinate,
        bytes: &[u8],
    ) -> Option<Self> {
        Self::new_with_storage(
            subject,
            coordinate,
            bytes,
            &mut decode_storage::UnrestrictedDecodeStorage,
        )
        .ok()
    }
    pub(super) fn new_with_storage<S: PhysicalRecoveryDecodeStorage>(
        subject: PersistedPhysicalDataFrameSubject,
        coordinate: RecordFrameCoordinate,
        bytes: &[u8],
        storage: &mut S,
    ) -> Result<Self, PhysicalRecoveryDecodeFailure<S::Denial>> {
        if bytes.len() != coordinate.length() as usize || !subject_matches(subject, coordinate) {
            return Err(PhysicalRecoveryProjectionDenial::InvalidFrame.into());
        }
        Ok(Self {
            subject,
            coordinate,
            bytes: decode_storage::copy_box(bytes, storage)?,
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
        Self::new_with_storage(
            coordinate,
            bytes,
            &mut decode_storage::UnrestrictedDecodeStorage,
        )
        .ok()
    }
    pub(super) fn new_with_storage<S: PhysicalRecoveryDecodeStorage>(
        coordinate: RecordFrameCoordinate,
        bytes: &[u8],
        storage: &mut S,
    ) -> Result<Self, PhysicalRecoveryDecodeFailure<S::Denial>> {
        if !matches!(
            coordinate.artifact(),
            RecordArtifactFile::ExtentArena { .. }
        ) || coordinate.length() as usize != bytes.len()
        {
            return Err(PhysicalRecoveryProjectionDenial::InvalidManifest.into());
        }
        Ok(Self {
            coordinate,
            bytes: decode_storage::copy_box(bytes, storage)?,
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
