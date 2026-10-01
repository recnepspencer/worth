//! Selected V3 predecessors form per-published-generation chains, not one
//! Store-wide chain. The checkpoint's global ratchet is joined separately.

use std::collections::{BTreeMap, BTreeSet};

use worth_store_physical_format::{PersistedRecordIdentity, ReleasedDropPredecessorV1};

use super::batch_join::SelectedReleasedGeneration;
use super::certificates::SelectedReleaseRoster;

#[derive(Clone, Copy)]
pub(super) struct SelectedDescriptorLink {
    pub(super) digest: [u8; 32],
    pub(super) predecessor: Option<ReleasedDropPredecessorV1>,
    pub(super) candidate_generation: u64,
    pub(super) cumulative_dropped: u64,
    pub(super) manifest_count: u16,
    pub(super) source: SelectedReleasedGeneration,
}

#[derive(Clone, Copy)]
pub(super) struct SelectedReleaseAnchor {
    pub(super) record: PersistedRecordIdentity,
    pub(super) digest: [u8; 32],
    pub(super) candidate_generation: u64,
}

pub(super) fn anchors(roster: &SelectedReleaseRoster) -> Vec<SelectedReleaseAnchor> {
    let tip = roster.accumulator().tip();
    let mut anchors = roster
        .batches()
        .iter()
        .map(|batch| SelectedReleaseAnchor {
            record: batch.descriptor_record(),
            digest: batch.descriptor_frame_sha256(),
            candidate_generation: batch.candidate_root_generation(),
        })
        .collect::<Vec<_>>();
    if !anchors
        .iter()
        .any(|anchor| anchor.record == tip.descriptor_record())
    {
        anchors.push(SelectedReleaseAnchor {
            record: tip.descriptor_record(),
            digest: tip.descriptor_frame_sha256(),
            candidate_generation: tip.candidate_root_generation(),
        });
    }
    anchors
}

pub(super) fn verifies(
    links: &BTreeMap<PersistedRecordIdentity, SelectedDescriptorLink>,
    anchors: &[SelectedReleaseAnchor],
) -> bool {
    if links.is_empty() || anchors.is_empty() {
        return false;
    }
    let mut anchored = BTreeSet::new();
    for anchor in anchors {
        let Some(link) = links.get(&anchor.record) else {
            return false;
        };
        if !anchored.insert(anchor.record)
            || link.digest != anchor.digest
            || link.candidate_generation != anchor.candidate_generation
        {
            return false;
        }
    }
    let mut predecessors = BTreeSet::new();
    for link in links.values() {
        if let Some(predecessor) = link.predecessor {
            let Some(prior) = links.get(&predecessor.descriptor_record()) else {
                return false;
            };
            if !predecessors.insert(predecessor.descriptor_record())
                || prior.digest != predecessor.descriptor_frame_sha256()
                || prior.source != link.source
                || prior.candidate_generation >= link.candidate_generation
                || prior
                    .cumulative_dropped
                    .checked_add(u64::from(link.manifest_count))
                    != Some(link.cumulative_dropped)
            {
                return false;
            }
        } else if link.cumulative_dropped != u64::from(link.manifest_count) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(ordinal: u64) -> PersistedRecordIdentity {
        PersistedRecordIdentity::new([1; 16], ordinal).unwrap()
    }

    fn source(object: u8) -> SelectedReleasedGeneration {
        SelectedReleasedGeneration {
            object: [object; 16],
            generation: 1,
            publication: record(u64::from(object)),
            publication_digest: [object; 32],
        }
    }

    #[test]
    fn independent_generations_and_exact_local_predecessors() {
        let first = SelectedDescriptorLink {
            digest: [2; 32],
            predecessor: None,
            candidate_generation: 5,
            cumulative_dropped: 2,
            manifest_count: 2,
            source: source(1),
        };
        let continuation = SelectedDescriptorLink {
            digest: [3; 32],
            predecessor: Some(ReleasedDropPredecessorV1::new(record(1), first.digest).unwrap()),
            candidate_generation: 8,
            cumulative_dropped: 3,
            manifest_count: 1,
            source: source(1),
        };
        let independent = SelectedDescriptorLink {
            digest: [4; 32],
            predecessor: None,
            candidate_generation: 11,
            cumulative_dropped: 1,
            manifest_count: 1,
            source: source(2),
        };
        let links = BTreeMap::from([
            (record(1), first),
            (record(2), continuation),
            (record(3), independent),
        ]);
        let anchor = SelectedReleaseAnchor {
            record: record(3),
            digest: independent.digest,
            candidate_generation: 11,
        };
        assert!(verifies(&links, &[anchor]));
        let mut forged = links.clone();
        forged.get_mut(&record(1)).unwrap().digest = [9; 32];
        assert!(!verifies(&forged, &[anchor]));
        let mut missing = links.clone();
        missing.remove(&record(1));
        assert!(!verifies(&missing, &[anchor]));
        let mut cross_generation = links.clone();
        cross_generation.get_mut(&record(2)).unwrap().source = source(2);
        assert!(!verifies(&cross_generation, &[anchor]));
        let mut wrong_count = links.clone();
        wrong_count.get_mut(&record(2)).unwrap().cumulative_dropped = 4;
        assert!(!verifies(&wrong_count, &[anchor]));
        let mut fork = links.clone();
        fork.insert(record(4), continuation);
        assert!(!verifies(&fork, &[anchor]));
        assert!(!verifies(
            &links,
            &[SelectedReleaseAnchor {
                digest: [9; 32],
                ..anchor
            }]
        ));
    }
}
