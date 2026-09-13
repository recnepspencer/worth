//! Historical skips require a WAL image chain ending at the selected, intact page.
use super::*;
use crate::RecoveryPageSource;
use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

struct PageClaim<'a> {
    member: usize,
    target: &'a PhysicalRedoTarget,
    last_lsn: u64,
    lsns: Vec<u64>,
}

pub(super) fn admit_scratch_bytes(
    retained: u64,
    records: &[PhysicalRedoRecord],
    projection: &PersistedPhysicalRecoveryProjection,
    limit: u64,
) -> Result<u64, PhysicalRedoPlanningDenial> {
    // 4 KiB per bounded item conservatively covers Vec spare capacity, sparse
    // BTree nodes, retained descriptor/range facts, and overlapping history,
    // predecessor, cursor, and chain containers. No page payload is copied.
    let targets = records
        .iter()
        .try_fold(0_u64, |sum, record| {
            sum.checked_add(record.targets().len() as u64)
        })
        .ok_or(PhysicalRedoPlanningDenial::CounterOverflow)?;
    let observed = targets
        .checked_add(projection.frames().len() as u64)
        .and_then(|count| count.checked_add(projection.placements().len() as u64))
        .and_then(|count| count.checked_mul(4096))
        .and_then(|bytes| retained.checked_add(bytes))
        .ok_or(PhysicalRedoPlanningDenial::CounterOverflow)?;
    if observed > limit {
        return Err(PhysicalRedoPlanningDenial::RecoveryMemoryLimit {
            observed,
            admitted: limit,
        });
    }
    Ok(observed)
}

pub(super) fn observed_predecessors(
    members: &[AdmittedPhysicalRedoMember],
    observations: &[RecoveryPageObservation],
) -> Result<BTreeSet<(u64, PhysicalRedoTargetIdentity)>, PhysicalRedoPlanningDenial> {
    let mut pages = BTreeMap::<(u64, u64), BTreeMap<u64, PageClaim<'_>>>::new();
    for (index, member) in members.iter().enumerate() {
        for record in &member.records {
            for target in record.targets() {
                let PhysicalRedoTargetIdentity::InlinePage {
                    segment,
                    page,
                    generation,
                } = target.identity()
                else {
                    continue;
                };
                let history = pages.entry((segment, page)).or_default();
                if let Some(claim) = history.get_mut(&generation) {
                    if claim.member != index || claim.target != target {
                        return Err(PhysicalRedoPlanningDenial::GenerationMismatch);
                    }
                    claim.last_lsn = claim.last_lsn.max(record.lsn().get());
                    claim.lsns.push(record.lsn().get());
                } else {
                    history.insert(
                        generation,
                        PageClaim {
                            member: index,
                            target,
                            last_lsn: record.lsn().get(),
                            lsns: vec![record.lsn().get()],
                        },
                    );
                }
            }
        }
    }
    let mut predecessors = BTreeSet::new();
    for observed in observations {
        let PhysicalRedoTargetIdentity::InlinePage {
            segment,
            page,
            generation,
        } = observed.target()
        else {
            continue;
        };
        let Some(history) = pages.get(&(segment, page)) else {
            continue;
        };
        let Some((&first, _)) = history.first_key_value() else {
            continue;
        };
        if first >= generation {
            continue;
        }
        let Some(anchor) = history.get(&generation) else {
            continue;
        };
        if !matches_observed(anchor, observed) {
            continue;
        }
        let chain = history
            .range(..=generation)
            .map(|(_, claim)| claim)
            .collect::<Vec<_>>();
        for adjacent in chain.windows(2) {
            require_successor(members, adjacent[0], adjacent[1])?;
        }
        for claim in chain
            .into_iter()
            .filter(|claim| claim.target.identity() != observed.target())
        {
            for lsn in &claim.lsns {
                predecessors.insert((*lsn, claim.target.identity()));
            }
        }
    }
    Ok(predecessors)
}

fn matches_observed(claim: &PageClaim<'_>, observed: &RecoveryPageObservation) -> bool {
    let RecoveryPageSource::Materialized { coordinate, .. } = observed.source() else {
        return false;
    };
    observed.target() == claim.target.identity()
        && observed.page_lsn() == claim.last_lsn
        && observed.frame_digest() == claim.target.resulting_digest()
        && Some(coordinate)
            == RecordFrameCoordinate::new(
                claim.target.artifact(),
                claim.target.artifact_offset(),
                claim.target.artifact_length(),
            )
}

fn require_successor(
    members: &[AdmittedPhysicalRedoMember],
    prior: &PageClaim<'_>,
    next: &PageClaim<'_>,
) -> Result<(), PhysicalRedoPlanningDenial> {
    let PhysicalRedoTargetIdentity::InlinePage {
        generation: prior_generation,
        ..
    } = prior.target.identity()
    else {
        unreachable!()
    };
    let PhysicalRedoTargetIdentity::InlinePage {
        generation: next_generation,
        ..
    } = next.target.identity()
    else {
        unreachable!()
    };
    if prior_generation.checked_add(1) != Some(next_generation)
        || prior.last_lsn >= next.lsns[0]
        || prior.member >= next.member
        || !successor_artifact(prior.target.artifact(), next.target.artifact())
    {
        return Err(PhysicalRedoPlanningDenial::GenerationMismatch);
    }
    // Every intervening admitted group must continue the same root lineage.
    for pair in members[prior.member..=next.member].windows(2) {
        let before = pair[0].projection.source_root_generation();
        let after = pair[1].projection.source_root_generation();
        let same_group = pair[0].group.group_identity() == pair[1].group.group_identity();
        if (same_group && before != after) || (!same_group && before.checked_add(1) != Some(after))
        {
            return Err(PhysicalRedoPlanningDenial::GenerationMismatch);
        }
    }
    require_preserved_records(
        &members[prior.member],
        prior.target,
        &members[next.member],
        next.target,
    )
}

fn successor_artifact(prior: RecordArtifactFile, next: RecordArtifactFile) -> bool {
    matches!((prior, next), (
        RecordArtifactFile::Segment { segment: before, generation },
        RecordArtifactFile::Segment { segment: after, generation: successor },
    ) if before == after && generation.checked_add(1) == Some(successor))
}

fn require_preserved_records(
    prior: &AdmittedPhysicalRedoMember,
    prior_target: &PhysicalRedoTarget,
    next: &AdmittedPhysicalRedoMember,
    next_target: &PhysicalRedoTarget,
) -> Result<(), PhysicalRedoPlanningDenial> {
    let before = inline_image(prior, prior_target)?;
    let after = inline_image(next, next_target)?;
    let before_bytes = prior.projection.frames()[before.frame_index].bytes();
    let after_bytes = next.projection.frames()[after.frame_index].bytes();
    for (placement, range) in &before.records {
        let Some((_, after_range)) = after.records.iter().find(|(candidate, _)| {
            candidate.record() == placement.record()
                && candidate.slot() == placement.slot()
                && candidate.slot_generation() == placement.slot_generation()
                && candidate.payload_bytes() == placement.payload_bytes()
        }) else {
            return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
        };
        if before_bytes[range.clone()] != after_bytes[after_range.clone()] {
            return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
        }
    }
    Ok(())
}

fn inline_image<'a>(
    member: &'a AdmittedPhysicalRedoMember,
    target: &PhysicalRedoTarget,
) -> Result<&'a projection_admission::AdmittedInlineFrame, PhysicalRedoPlanningDenial> {
    member
        .inline_frames
        .iter()
        .find(|image| {
            let coordinate = member.projection.frames()[image.frame_index].coordinate();
            coordinate.artifact() == target.artifact()
                && coordinate.offset() == target.artifact_offset()
                && coordinate.length() == target.artifact_length()
        })
        .ok_or(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)
}
