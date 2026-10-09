use super::*;

impl ArtifactTreeMedia<'_> {
    /// Converges a caller-admitted immutable candidate only when the entire
    /// existing file is an exact prefix. A conflicting byte or exterior denies
    /// before any write. The Store caller separately proves it is unselected.
    pub fn write_candidate_prefix_scheduled_exact(
        &self,
        artifact: &ArtifactTreeFile,
        coordinate: RecordFrameCoordinate,
        bytes: &[u8],
        binding: BackendQueueExecutionPlanBinding,
        adaptation: BackendQueueExecutionAdaptation,
        durability: ArtifactRangeWriteDurabilityRequirement,
    ) -> ScheduledArtifactRangeWriteOutcome {
        self.write_scheduled(
            ArtifactRangeWriteRequest {
                artifact,
                coordinate,
                bytes,
                durability,
                posture: ArtifactRangeWritePosture::CandidatePrefix,
            },
            ScheduledArtifactRangeWriteContext {
                binding,
                adaptation,
                writeback_scope: None,
            },
        )
    }
}

pub(super) fn verify(
    owner: &crate::filesystem_media::FilesystemMediaOwner,
    file: &mut cap_std::fs::File,
    expected: &[u8],
    length: u64,
) -> Result<(), ArtifactTreeFailure> {
    let length = usize::try_from(length).map_err(|_| denial())?;
    let mut offset = 0;
    let mut buffer = [0u8; 16384];
    while offset < length {
        let take = (length - offset).min(buffer.len());
        match super::super::exact_read_effect::execute(owner, file, &mut buffer[..take]) {
            super::super::exact_read_effect::ExactReadEffect::Completed {
                completed_bytes, ..
            } if completed_bytes == take as u64
                && buffer[..take] == expected[offset..offset + take] => {}
            super::super::exact_read_effect::ExactReadEffect::DeniedBeforeEffect(failure) => {
                return Err(failure)
            }
            _ => return Err(denial()),
        }
        offset += take;
    }
    Ok(())
}

fn denial() -> ArtifactTreeFailure {
    ArtifactTreeFailure::structural(ArtifactTreeFailureKind::Damaged)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filesystem_media::*;
    use worth_proof::TransitionOutcome;

    #[test]
    fn candidate_prefix_converges_only_matching_bytes_without_exterior() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("store");
        let media =
            match FilesystemMediaOwner::qualify(FilesystemQualificationRequest::certification(
                &root,
                FilesystemAccessPosture::CoordinatedServiceAccount,
            ))
            .into_raw()
            {
                TransitionOutcome::Success(media) => media,
                _ => panic!("real qualified filesystem required"),
            };
        let tree = media.artifact_tree();
        let directory = ArtifactTreeDirectory::families()
            .child("candidate-test")
            .unwrap();
        tree.create_directory(&directory).unwrap();
        for (generation, initial, accepted) in [
            (1, &b"abc"[..], true),
            (2, &b"abX"[..], false),
            (3, &b"abcdefX"[..], false),
        ] {
            let logical =
                worth_store_physical_format::RecordArtifactFile::RootManifest { generation };
            let artifact = directory.file(&logical.file_name()).unwrap();
            assert!(matches!(
                tree.write_new_exact(
                    &artifact,
                    ArtifactNewWriteRange::new(initial.len() as u64).unwrap(),
                    initial
                ),
                ArtifactNewWriteOutcome::Completed(_)
            ));
            let coordinate = RecordFrameCoordinate::new(logical, 0, 6).unwrap();
            let outcome = tree.write_exact(ArtifactRangeWriteRequest {
                artifact: &artifact,
                coordinate,
                bytes: b"abcdef",
                durability: ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
                posture: ArtifactRangeWritePosture::CandidatePrefix,
            });
            assert_eq!(
                matches!(outcome, ArtifactRangeWriteOutcome::Completed(_)),
                accepted
            );
            assert_eq!(
                tree.read_bounded(&artifact, 16).unwrap(),
                if accepted { &b"abcdef"[..] } else { initial }
            );
        }
        media.close();
    }
}
