use std::ffi::{OsStr, OsString};

use worth_store_physical_backend::ObservedWalArtifact;
use worth_store_physical_format::{store_namespace::NamespaceEntryType, WalSegmentIdentity};
use worth_store_physical_integrity::{
    IntegrityValidatedWalFrame, PhysicalArtifactScope, PhysicalByteRange,
    UntrustedPhysicalArtifact, WalPayloadProjectionDenial,
};

/// Store-owned binding between one C.9 validation and its exact C.4 WAL read.
///
/// The owned frame can cross recovery phases without retaining a raw media
/// borrow. Its bytes remain private to Store owner interpretation.
#[derive(Debug, Clone)]
pub struct IntegrityAdmittedRecoveryWalFrame {
    source_name: OsString,
    source_entry_type: NamespaceEntryType,
    source_incarnation: crate::physical_runtime::RecoveryWalObservationIdentity,
    scope: PhysicalArtifactScope,
    segment_identity: WalSegmentIdentity,
    lsn_start: u64,
    lsn_end: u64,
    identity_digest: [u8; 32],
    payload_digest: [u8; 32],
    encoded: Box<[u8]>,
    payload_range: std::ops::Range<usize>,
}

/// Store-owned exact-frame evidence for one complete C.9-admitted WAL segment.
///
/// Recovery may retain this opaque evidence for lawful cleanup, but only Store
/// modules can reach admitted redo payloads.
#[derive(Debug, Clone)]
pub struct IntegrityAdmittedRecoveryWalSegment {
    inspection: worth_store_wal::WalSegmentInspection,
    frames: Box<[IntegrityAdmittedRecoveryWalFrame]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryWalIntegrityAdmissionDenial {
    MissingBoundedArtifact,
    ScopeMismatch,
    SourceRangeOutsideObservation,
    SourceIncarnationMismatch,
}

impl IntegrityAdmittedRecoveryWalFrame {
    pub(in crate::physical_runtime) fn bind(
        observed: &ObservedWalArtifact,
        expected_scope: PhysicalArtifactScope,
        relative_range: PhysicalByteRange,
        validated: IntegrityValidatedWalFrame<'_>,
    ) -> Result<Self, RecoveryWalIntegrityAdmissionDenial> {
        if expected_scope != validated.scope()
            || relative_range != expected_scope.byte_range()
            || observed.store_identity() != expected_scope.store_identity()
        {
            return Err(RecoveryWalIntegrityAdmissionDenial::ScopeMismatch);
        }
        let bytes = observed
            .bytes()
            .ok_or(RecoveryWalIntegrityAdmissionDenial::MissingBoundedArtifact)?;
        let source_incarnation = observed.observation_identity();
        let start = usize::try_from(relative_range.offset())
            .map_err(|_| RecoveryWalIntegrityAdmissionDenial::SourceRangeOutsideObservation)?;
        let end = usize::try_from(relative_range.end_exclusive())
            .map_err(|_| RecoveryWalIntegrityAdmissionDenial::SourceRangeOutsideObservation)?;
        let encoded = bytes
            .get(start..end)
            .ok_or(RecoveryWalIntegrityAdmissionDenial::SourceRangeOutsideObservation)?;
        let input = UntrustedPhysicalArtifact::from_bounded_bytes(encoded);
        if !validated.matches_input(input) {
            return Err(RecoveryWalIntegrityAdmissionDenial::SourceIncarnationMismatch);
        }
        let payload = validated
            .project_payload(input, validated.segment_identity())
            .map_err(map_projection_denial)?;
        let payload_range = payload.payload_range();
        Ok(Self {
            source_name: observed.name().to_owned(),
            source_entry_type: observed.entry_type(),
            source_incarnation,
            scope: expected_scope,
            segment_identity: validated.segment_identity(),
            lsn_start: validated.lsn_start(),
            lsn_end: validated.lsn_end(),
            identity_digest: validated.identity_digest(),
            payload_digest: validated.payload_digest(),
            encoded: encoded.into(),
            payload_range,
        })
    }

    pub fn source_name(&self) -> &OsStr {
        &self.source_name
    }
    pub const fn source_entry_type(&self) -> NamespaceEntryType {
        self.source_entry_type
    }
    pub const fn scope(&self) -> PhysicalArtifactScope {
        self.scope
    }
    pub const fn segment_identity(&self) -> WalSegmentIdentity {
        self.segment_identity
    }
    pub const fn lsn_start(&self) -> u64 {
        self.lsn_start
    }
    pub const fn lsn_end(&self) -> u64 {
        self.lsn_end
    }
    pub fn lsn_range(&self) -> worth_store_wal::WalLsnRange {
        worth_store_wal::WalLsnRange::new(
            worth_store_wal::LogSequenceNumber::new(self.lsn_start),
            worth_store_wal::LogSequenceNumber::new(self.lsn_end),
        )
        .expect("C.9 admitted WAL frame carries an ordered LSN range")
    }
    pub const fn identity_digest(&self) -> [u8; 32] {
        self.identity_digest
    }
    pub const fn payload_digest(&self) -> [u8; 32] {
        self.payload_digest
    }
    pub fn encoded_byte_count(&self) -> u64 {
        self.encoded.len() as u64
    }
    pub fn payload_byte_count(&self) -> u64 {
        self.payload_range.len() as u64
    }
    pub(in crate::physical_runtime) fn payload(&self) -> &[u8] {
        &self.encoded[self.payload_range.clone()]
    }
}

impl IntegrityAdmittedRecoveryWalSegment {
    pub(in crate::physical_runtime) fn from_complete_frames(
        observed: &ObservedWalArtifact,
        identity: worth_store_wal::WalSegmentArtifactIdentity,
        frames: Vec<IntegrityAdmittedRecoveryWalFrame>,
    ) -> Option<Self> {
        use sha2::{Digest, Sha256};

        let first = frames.first()?;
        let last = frames.last()?;
        let observed_bytes = observed.bytes()?;
        let mut expected_offset = 0_u64;
        if frames.iter().any(|frame| {
            let range = frame.scope().byte_range();
            let start = usize::try_from(range.offset()).ok();
            let end = usize::try_from(range.end_exclusive()).ok();
            let source_matches = frame.source_name() == observed.name()
                && frame.source_entry_type() == observed.entry_type()
                && frame.scope().store_identity() == observed.store_identity()
                && frame.source_incarnation == observed.observation_identity()
                && start
                    .zip(end)
                    .and_then(|(start, end)| observed_bytes.get(start..end))
                    .is_some_and(|bytes| bytes == &*frame.encoded);
            let contiguous = range.offset() == expected_offset;
            expected_offset = range.end_exclusive();
            frame.segment_identity() != identity.format_identity() || !source_matches || !contiguous
        }) || frames
            .windows(2)
            .any(|pair| pair[0].lsn_end() != pair[1].lsn_start())
        {
            return None;
        }
        let byte_count = frames.iter().try_fold(0_u64, |total, frame| {
            total.checked_add(frame.encoded_byte_count())
        })?;
        let mut artifact_digest = Sha256::new();
        for frame in &frames {
            artifact_digest.update(&frame.encoded);
        }
        let lsn_range = worth_store_wal::WalLsnRange::new(
            worth_store_wal::LogSequenceNumber::new(first.lsn_start()),
            worth_store_wal::LogSequenceNumber::new(last.lsn_end()),
        )
        .ok()?;
        let inspection = worth_store_wal::WalSegmentInspection::from_admitted_frames(
            identity,
            lsn_range,
            frames.len() as u64,
            byte_count,
            artifact_digest.finalize().into(),
        )?;
        Some(Self {
            inspection,
            frames: frames.into_boxed_slice(),
        })
    }

    pub const fn inspection(&self) -> worth_store_wal::WalSegmentInspection {
        self.inspection
    }

    pub fn frames(&self) -> &[IntegrityAdmittedRecoveryWalFrame] {
        &self.frames
    }
}

fn map_projection_denial(
    denial: WalPayloadProjectionDenial,
) -> RecoveryWalIntegrityAdmissionDenial {
    match denial {
        WalPayloadProjectionDenial::InputIncarnationMismatch => {
            RecoveryWalIntegrityAdmissionDenial::SourceIncarnationMismatch
        }
        WalPayloadProjectionDenial::SegmentIdentityMismatch => {
            RecoveryWalIntegrityAdmissionDenial::ScopeMismatch
        }
    }
}

#[cfg(test)]
mod tests {
    use worth_proof::TransitionOutcome;
    use worth_store_physical_format::wal_frame::{encode_wal_frame_v1, WalFrameV1EncodeRequest};
    use worth_store_physical_integrity::{validate_wal_frame, UntrustedPhysicalArtifact};

    use super::*;
    use crate::physical_runtime::{
        FilesystemAccessPosture, FilesystemMediaAdmission, PhysicalRuntimeAdmission, PhysicalStore,
        QualifiedRecoveryFilesystemMedia,
    };

    #[test]
    fn c4_observation_cannot_be_substituted_during_segment_assembly() {
        let parent = tempfile::tempdir().unwrap();
        let root_a = parent.path().join("store-a");
        let root_b = parent.path().join("store-b");
        let store_a = initialize(&root_a);
        let _store_b = initialize(&root_b);
        let identity = WalSegmentIdentity::new(1, 1).unwrap();
        let frame = encode_wal_frame_v1(
            WalFrameV1EncodeRequest::from_segment_identity(
                identity,
                2,
                3,
                b"store-substitution",
                b"payload",
            )
            .unwrap(),
        );
        let file_name = "segment-1-generation-1.wal";
        for root in [&root_a, &root_b] {
            let wal = root.join("families").join("wal");
            std::fs::create_dir_all(&wal).unwrap();
            std::fs::write(wal.join(file_name), &frame).unwrap();
        }
        let media_a = QualifiedRecoveryFilesystemMedia::qualify_existing(&root_a)
            .unwrap()
            .admit_persisted_store()
            .unwrap();
        let media_b = QualifiedRecoveryFilesystemMedia::qualify_existing(&root_b)
            .unwrap()
            .admit_persisted_store()
            .unwrap();
        let mut discovery_a = media_a.bounded_discovery(2, 4096).unwrap();
        let mut discovery_b = media_b.bounded_discovery(2, 4096).unwrap();
        let observed_a = discovery_a.read_wal_artifacts(1, 4096).unwrap();
        let repeated_a = discovery_a.read_wal_artifacts(1, 4096).unwrap();
        let observed_b = discovery_b.read_wal_artifacts(1, 4096).unwrap();
        let media_a = discovery_a.finish();
        let mut rediscovery_a = media_a.bounded_discovery(1, 4096).unwrap();
        let rediscovered_a = rediscovery_a.read_wal_artifacts(1, 4096).unwrap();
        let range = PhysicalByteRange::new(0, frame.len() as u64).unwrap();
        let scope = PhysicalArtifactScope::wal_frame(store_a, identity, range);
        let validation = validate_wal_frame(
            UntrustedPhysicalArtifact::from_bounded_bytes(observed_a[0].bytes().unwrap()),
            scope,
        )
        .0;
        let worth_store_physical_integrity::WalFrameIntegrityValidation::Intact(validated) =
            validation
        else {
            panic!("fixture frame must be intact")
        };
        let admitted =
            IntegrityAdmittedRecoveryWalFrame::bind(&observed_a[0], scope, range, validated)
                .unwrap();
        let artifact_identity = worth_store_wal::WalSegmentArtifactIdentity::parse(file_name)
            .expect("canonical fixture identity");
        assert!(IntegrityAdmittedRecoveryWalSegment::from_complete_frames(
            &observed_b[0],
            artifact_identity,
            vec![admitted.clone()],
        )
        .is_none());
        assert!(IntegrityAdmittedRecoveryWalSegment::from_complete_frames(
            &repeated_a[0],
            artifact_identity,
            vec![admitted.clone()],
        )
        .is_none());
        assert!(IntegrityAdmittedRecoveryWalSegment::from_complete_frames(
            &rediscovered_a[0],
            artifact_identity,
            vec![admitted.clone()],
        )
        .is_none());
        assert!(IntegrityAdmittedRecoveryWalSegment::from_complete_frames(
            &observed_a[0],
            artifact_identity,
            vec![admitted],
        )
        .is_some());
        drop(rediscovery_a.finish());
        drop(discovery_b.finish());
    }

    fn initialize(
        root: &std::path::Path,
    ) -> worth_store_physical_format::store_namespace::StableStoreIdentity {
        let runtime =
            PhysicalStore::admit(PhysicalRuntimeAdmission::new(root.to_owned()).unwrap()).unwrap();
        let admission = FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        );
        let media = match runtime.try_admit_filesystem_media(admission).into_raw() {
            TransitionOutcome::Success(media) => media,
            _ => panic!("store initialization failed"),
        };
        let store = media.store_identity();
        let _ = media.close();
        store
    }
}

#[cfg(test)]
mod media_generation_tests;
