use super::RetiredArtifact;

/// Exact generation a held retirement claim may delete. Only the admission
/// owner can issue one, and only while that generation is claimed.
#[derive(Clone, Copy)]
pub(in crate::physical_runtime) struct RetirementRemovalPermit {
    artifact: RetiredArtifact,
}

impl RetirementRemovalPermit {
    pub(super) const fn issued(artifact: RetiredArtifact) -> Self {
        Self { artifact }
    }

    /// Whether this permit names `file` as one of its generation's files.
    pub(in crate::physical_runtime) fn admits(
        self,
        file: worth_store_physical_format::RecordArtifactFile,
    ) -> bool {
        self.artifact.admits_removal(file)
    }
}

pub(in crate::physical_runtime) const RETIREMENT_DOMAIN: &[u8] = b"store.physical.retirement.v2";

pub(in crate::physical_runtime) const RETIREMENT_INTENT: u8 = 1;
pub(in crate::physical_runtime) const RETIREMENT_COMPLETION: u8 = 2;
pub(in crate::physical_runtime) const RETIREMENT_EXTENT_INTENT: u8 = 3;
pub(in crate::physical_runtime) const RETIREMENT_EXTENT_COMPLETION: u8 = 4;

/// Why `retire_displaced_segment` did not complete a retirement.
///
/// Claim denials (`Absent` through `Retained`) precede every effect. Stage
/// denials name the first stage that did not finish; a durable intent survives
/// them and a later call or fresh recovery resumes the same retirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRetirementDenial {
    /// No displaced generation awaits retirement.
    Absent,
    /// A live reader still protects the source root or the displaced generation.
    Protected,
    /// A root-changing publication is still pending.
    Unresolved,
    /// The current root still reads the generation, or the successor checkpoint
    /// did not move past its source root.
    Retained,
    /// The retirement WAL frame could not be planned.
    WalPlan,
    /// Writing the retirement WAL frame failed.
    WalWrite,
    /// Synchronizing the retirement WAL frame failed.
    WalSync,
    /// Settling the retirement WAL frame failed.
    WalFinish,
    /// Another retirement is running, or the scheduler has not started the
    /// frame. The claim stays charged.
    Waiting,
    /// The admitted arena free-range index cannot represent another isolated
    /// release. The displaced extent remains charged and may be retried after
    /// allocator pressure changes; repeating without such a change cannot help.
    ArenaIndexCapacity,
    /// The arena has no capacity for this release independently of index size.
    ArenaCapacity,
    /// The range being released conflicts with the allocator's geometry or
    /// already admitted ranges; it is not a scheduler wait.
    ArenaReleaseInvalid,
    /// Deleting the generation or synchronizing its namespace failed.
    Delete,
    /// The successor-root checkpoint did not complete.
    Checkpoint,
}

pub(in crate::physical_runtime) fn payload_is_retirement(payload: &[u8]) -> bool {
    let Some(encoded_len) = payload.get(..8) else {
        return false;
    };
    let Ok(encoded_len) = <[u8; 8]>::try_from(encoded_len) else {
        return false;
    };
    let length = u64::from_le_bytes(encoded_len);
    length == RETIREMENT_DOMAIN.len() as u64
        && payload.get(8..8 + RETIREMENT_DOMAIN.len()) == Some(RETIREMENT_DOMAIN)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct RetirementRecord {
    pub(in crate::physical_runtime) artifact: RetiredArtifact,
    pub(in crate::physical_runtime) completion: bool,
    pub(in crate::physical_runtime) source_root: u64,
    pub(in crate::physical_runtime) bytes: u64,
    pub(in crate::physical_runtime) release: Option<super::RetirementReleaseProjection>,
}

pub(in crate::physical_runtime) fn decode_retirement(payload: &[u8]) -> Option<RetirementRecord> {
    if !payload_is_retirement(payload) {
        return None;
    }
    let body = payload.get(8 + RETIREMENT_DOMAIN.len()..)?;
    let (action, body) = body.split_first()?;
    if body.len() != 120 {
        return None;
    }
    let mut numbers = [0u64; 7];
    for (index, number) in numbers.iter_mut().enumerate() {
        let start = index * 8;
        *number = u64::from_le_bytes(body[start..start + 8].try_into().ok()?);
    }
    let range = if numbers[4..] == [0, 0, 0] {
        None
    } else {
        Some(worth_store_physical_format::ExtentArenaRange::new(
            worth_store_physical_format::ExtentArenaId::new(numbers[4])?,
            numbers[5],
            numbers[6],
        )?)
    };
    if numbers[0] == 0 || range.is_some_and(|range| range.length() != numbers[3]) {
        return None;
    }
    let (artifact, completion) =
        RetiredArtifact::from_action(*action, numbers[1], numbers[2], range)?;
    let release = if body[56..] == [0; 64] {
        None
    } else {
        Some(super::RetirementReleaseProjection::new(
            u64::from_le_bytes(body[56..64].try_into().ok()?),
            u64::from_le_bytes(body[64..72].try_into().ok()?),
            body[72..104].try_into().ok()?,
            u64::from_le_bytes(body[104..112].try_into().ok()?),
            u64::from_le_bytes(body[112..120].try_into().ok()?),
        )?)
    };
    let valid_release = match (artifact, release) {
        (RetiredArtifact::Segment { .. }, None) => true,
        (RetiredArtifact::Extent { .. }, Some(release)) => release.source_generation() > numbers[0],
        (RetiredArtifact::Arena { generation, .. }, Some(release)) => {
            generation == numbers[0] && release.source_generation() >= numbers[0]
        }
        _ => false,
    };
    if !valid_release {
        return None;
    }
    Some(RetirementRecord {
        artifact,
        completion,
        source_root: numbers[0],
        bytes: numbers[3],
        release,
    })
}

/// The newest record for each retired generation that has not reached completion.
pub(in crate::physical_runtime) fn unresolved_retirements(
    records: Vec<RetirementRecord>,
) -> Vec<RetirementRecord> {
    let mut latest = std::collections::BTreeMap::new();
    for record in records {
        if !record.completion {
            latest.entry(record.artifact).or_insert(record);
        } else if latest
            .get(&record.artifact)
            .is_some_and(|intent| same_retirement(*intent, record))
        {
            latest.remove(&record.artifact);
        }
    }
    latest.into_values().collect()
}

fn same_retirement(intent: RetirementRecord, completion: RetirementRecord) -> bool {
    intent.artifact == completion.artifact
        && intent.source_root == completion.source_root
        && intent.bytes == completion.bytes
        && intent.release == completion.release
}

/// Live reclamation keeps these spans until a completion is the newest record.
pub(in crate::physical_runtime) fn unresolved_retirement_holds(
    records: Vec<(RetirementRecord, u64, u64)>,
) -> Vec<(RetiredArtifact, u64, u64)> {
    let mut latest = std::collections::BTreeMap::new();
    for (record, start, end) in records {
        if !record.completion {
            latest
                .entry(record.artifact)
                .or_insert((record, start, end));
        } else if latest
            .get(&record.artifact)
            .is_some_and(|(intent, _, _)| same_retirement(*intent, record))
        {
            latest.remove(&record.artifact);
        }
    }
    latest
        .into_iter()
        .map(|(artifact, (_, start, end))| (artifact, start, end))
        .collect()
}

pub(in crate::physical_runtime::durability) fn note_retirement_hold(
    holds: &mut Vec<(RetiredArtifact, u64, u64)>,
    retirement: Option<(RetiredArtifact, bool)>,
    start: u64,
    end: u64,
) {
    let Some((artifact, completion)) = retirement else {
        return;
    };
    holds.retain(|hold| hold.0 != artifact);
    if !completion {
        holds.push((artifact, start, end));
    }
}

pub(in crate::physical_runtime) fn encode_retirement(
    artifact: RetiredArtifact,
    completion: bool,
    source_root: u64,
    bytes: u64,
    release: Option<super::RetirementReleaseProjection>,
) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(8 + RETIREMENT_DOMAIN.len() + 1 + 120);
    encoded.extend_from_slice(&(RETIREMENT_DOMAIN.len() as u64).to_le_bytes());
    encoded.extend_from_slice(RETIREMENT_DOMAIN);
    encoded.push(artifact.action_code(completion));
    encoded.extend_from_slice(&source_root.to_le_bytes());
    encoded.extend_from_slice(&artifact.id().to_le_bytes());
    encoded.extend_from_slice(&artifact.generation().to_le_bytes());
    encoded.extend_from_slice(&bytes.to_le_bytes());
    let coordinates = artifact.arena_range().map_or([0, 0, 0], |range| {
        [range.arena().get(), range.offset(), range.length()]
    });
    for coordinate in coordinates {
        encoded.extend_from_slice(&coordinate.to_le_bytes());
    }
    if let Some(release) = release {
        encoded.extend_from_slice(&release.source_generation().to_le_bytes());
        encoded.extend_from_slice(&release.candidate_generation().to_le_bytes());
        encoded.extend_from_slice(&release.candidate_digest());
        encoded.extend_from_slice(&release.metadata_bytes().to_le_bytes());
        encoded.extend_from_slice(&release.publication().to_le_bytes());
    } else {
        encoded.extend_from_slice(&[0; 64]);
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arena_retirement_binds_empty_root_and_requires_forget_publication() {
        let artifact = RetiredArtifact::Arena {
            arena: 3,
            generation: 8,
        };
        let release =
            super::super::RetirementReleaseProjection::new(8, 9, [0x44; 32], 2048, 72).unwrap();
        let encoded = encode_retirement(artifact, false, 8, 64 << 20, Some(release));
        let record = decode_retirement(&encoded).unwrap();
        assert_eq!(record.artifact, artifact);
        assert_eq!(record.release, Some(release));
        assert!(artifact.admits_removal(
            worth_store_physical_format::RecordArtifactFile::ExtentArena { arena: 3 }
        ));
        assert!(!artifact.admits_removal(
            worth_store_physical_format::RecordArtifactFile::ExtentArena { arena: 4 }
        ));
        assert!(
            decode_retirement(&encode_retirement(artifact, false, 8, 64 << 20, None)).is_none()
        );
        assert!(decode_retirement(&encode_retirement(
            artifact,
            false,
            7,
            64 << 20,
            Some(release)
        ))
        .is_none());
        let wrong_nonce =
            super::super::RetirementReleaseProjection::new(8, 9, [0x44; 32], 2048, 73).unwrap();
        let completion = decode_retirement(&encode_retirement(
            artifact,
            true,
            8,
            64 << 20,
            Some(wrong_nonce),
        ))
        .unwrap();
        assert_eq!(
            unresolved_retirements(vec![record, completion]),
            vec![record]
        );
    }

    #[test]
    fn extent_release_round_trip_binds_range_and_exact_publication() {
        let range = worth_store_physical_format::ExtentArenaRange::new(
            worth_store_physical_format::ExtentArenaId::new(3).unwrap(),
            4096,
            8192,
        )
        .unwrap();
        let artifact = RetiredArtifact::Extent {
            extent: 7,
            generation: 2,
            range,
        };
        let release =
            super::super::RetirementReleaseProjection::new(8, 9, [0x41; 32], 4096, 71).unwrap();
        let encoded = encode_retirement(artifact, false, 4, 8192, Some(release));
        let intent = decode_retirement(&encoded).unwrap();
        assert_eq!(intent.artifact, artifact);
        assert_eq!(intent.release, Some(release));
        assert!(decode_retirement(&encode_retirement(artifact, false, 4, 8192, None)).is_none());
        assert!(
            decode_retirement(&encode_retirement(artifact, false, 4, 4096, Some(release)))
                .is_none()
        );
        let wrong_completion = RetirementRecord {
            completion: true,
            release: super::super::RetirementReleaseProjection::new(8, 9, [0x42; 32], 4096, 71),
            ..intent
        };
        assert_eq!(
            unresolved_retirements(vec![intent, wrong_completion]),
            vec![intent]
        );
        assert_eq!(
            unresolved_retirement_holds(vec![(intent, 11, 12), (wrong_completion, 12, 13)]),
            vec![(artifact, 11, 12)]
        );
        assert!(unresolved_retirements(vec![
            intent,
            RetirementRecord {
                completion: true,
                ..intent
            }
        ])
        .is_empty());
    }
}
