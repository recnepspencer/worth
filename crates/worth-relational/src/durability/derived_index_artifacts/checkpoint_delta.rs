use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::durability::data::{DurabilityError, RecoveryFailureClass};
use crate::history::data::{BranchId, CommitId};
use crate::identity::data::VersionId;
use crate::indexes::data::{
    DerivedIndexApplicability, DerivedIndexEntries, DerivedIndexGeneration,
    DerivedIndexGenerationId, DerivedIndexId, DerivedIndexPublicationStatus,
};
use crate::schema::data::SchemaVersionId;

const FORMAT_VERSION: u16 = 1;
const DIGEST_DOMAIN: &[u8] = b"worth.relational.derived-index-checkpoint.v1\0";

/// Checkpoint-only deltas. Canonical envelopes remain the semantic authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DerivedIndexCheckpointArtifacts {
    format_version: u16,
    generation_ids: Vec<DerivedIndexGenerationId>,
    generations: Vec<CheckpointGeneration>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CheckpointGeneration {
    generation_id: DerivedIndexGenerationId,
    index_id: DerivedIndexId,
    source_commit_id: CommitId,
    source_branch_id: BranchId,
    applicability_branch_id: BranchId,
    version_id: VersionId,
    schema_version: SchemaVersionId,
    status: DerivedIndexPublicationStatus,
    base_generation_id: Option<DerivedIndexGenerationId>,
    shared_prefix_bytes: u64,
    shared_suffix_bytes: u64,
    changed_bytes: Vec<u8>,
    full_digest: [u8; 32],
}

impl DerivedIndexCheckpointArtifacts {
    pub(crate) fn capture(
        mut generations: Vec<DerivedIndexGeneration>,
    ) -> Result<Self, DurabilityError> {
        generations.sort_by_key(|generation| (generation.index_id, generation.generation_id));
        let mut previous = BTreeMap::<DerivedIndexId, (DerivedIndexGenerationId, Vec<u8>)>::new();
        let mut captured = Vec::with_capacity(generations.len());
        for generation in generations {
            let full = rmp_serde::to_vec_named(&generation)
                .map_err(|error| corrupt(format!("index checkpoint encoding failed: {error}")))?;
            let entries = rmp_serde::to_vec_named(&generation.entries)
                .map_err(|error| corrupt(format!("index entries encoding failed: {error}")))?;
            let (base_generation_id, prefix, suffix, changed_bytes) =
                if let Some((base_id, base)) = previous.get(&generation.index_id) {
                    let prefix = base
                        .iter()
                        .zip(&entries)
                        .take_while(|(a, b)| a == b)
                        .count();
                    let suffix = base[prefix..]
                        .iter()
                        .rev()
                        .zip(entries[prefix..].iter().rev())
                        .take_while(|(a, b)| a == b)
                        .count();
                    let changed = entries[prefix..entries.len() - suffix].to_vec();
                    if changed.len().saturating_add(16) >= entries.len() {
                        (None, 0, 0, entries.clone())
                    } else {
                        (Some(*base_id), prefix, suffix, changed)
                    }
                } else {
                    (None, 0, 0, entries.clone())
                };
            captured.push(CheckpointGeneration {
                generation_id: generation.generation_id,
                index_id: generation.index_id,
                source_commit_id: generation.source_commit_id,
                source_branch_id: generation.source_branch_id,
                applicability_branch_id: generation.applicability.branch_id,
                version_id: generation.applicability.version_id,
                schema_version: generation.applicability.schema_version,
                status: generation.status,
                base_generation_id,
                shared_prefix_bytes: u64::try_from(prefix)
                    .map_err(|_| corrupt("index checkpoint prefix exceeds wire range"))?,
                shared_suffix_bytes: u64::try_from(suffix)
                    .map_err(|_| corrupt("index checkpoint suffix exceeds wire range"))?,
                changed_bytes,
                full_digest: digest(&full),
            });
            previous.insert(generation.index_id, (generation.generation_id, entries));
        }
        Ok(Self {
            format_version: FORMAT_VERSION,
            generation_ids: captured.iter().map(|entry| entry.generation_id).collect(),
            generations: captured,
        })
    }

    pub(crate) fn readmit(&self) -> Result<Vec<DerivedIndexGeneration>, DurabilityError> {
        if self.format_version != FORMAT_VERSION {
            return Err(corrupt("unsupported derived index checkpoint format"));
        }
        if self.generation_ids
            != self
                .generations
                .iter()
                .map(|entry| entry.generation_id)
                .collect::<Vec<_>>()
        {
            return Err(corrupt("derived index checkpoint manifest mismatch"));
        }
        let mut previous = BTreeMap::<DerivedIndexId, (DerivedIndexGenerationId, Vec<u8>)>::new();
        let mut seen = BTreeSet::new();
        let mut generations = Vec::with_capacity(self.generations.len());
        for entry in &self.generations {
            if !seen.insert(entry.generation_id) {
                return Err(corrupt("duplicate derived index checkpoint generation"));
            }
            let entries_bytes = match entry.base_generation_id {
                Some(base_id) => {
                    let (previous_id, base) = previous
                        .get(&entry.index_id)
                        .ok_or_else(|| corrupt("missing derived index checkpoint base"))?;
                    if *previous_id != base_id {
                        return Err(corrupt("stale derived index checkpoint base"));
                    }
                    let prefix = usize::try_from(entry.shared_prefix_bytes)
                        .map_err(|_| corrupt("invalid derived index checkpoint prefix"))?;
                    let suffix = usize::try_from(entry.shared_suffix_bytes)
                        .map_err(|_| corrupt("invalid derived index checkpoint suffix"))?;
                    if prefix > base.len() || suffix > base.len() - prefix {
                        return Err(corrupt("derived index checkpoint delta exceeds base"));
                    }
                    let mut full = Vec::with_capacity(
                        prefix
                            .saturating_add(entry.changed_bytes.len())
                            .saturating_add(suffix),
                    );
                    full.extend_from_slice(&base[..prefix]);
                    full.extend_from_slice(&entry.changed_bytes);
                    full.extend_from_slice(&base[base.len() - suffix..]);
                    full
                }
                None if entry.shared_prefix_bytes == 0 && entry.shared_suffix_bytes == 0 => {
                    entry.changed_bytes.clone()
                }
                None => return Err(corrupt("unbased derived index checkpoint delta")),
            };
            let entries: DerivedIndexEntries = rmp_serde::from_slice(&entries_bytes)
                .map_err(|error| corrupt(format!("index entries decoding failed: {error}")))?;
            let generation = DerivedIndexGeneration {
                generation_id: entry.generation_id,
                index_id: entry.index_id,
                source_commit_id: entry.source_commit_id,
                source_branch_id: entry.source_branch_id.clone(),
                applicability: DerivedIndexApplicability {
                    branch_id: entry.applicability_branch_id.clone(),
                    version_id: entry.version_id,
                    schema_version: entry.schema_version,
                },
                status: entry.status,
                entries,
            };
            let full = rmp_serde::to_vec_named(&generation)
                .map_err(|error| corrupt(format!("index readmission encoding failed: {error}")))?;
            if digest(&full) != entry.full_digest {
                return Err(corrupt("derived index checkpoint digest mismatch"));
            }
            previous.insert(entry.index_id, (entry.generation_id, entries_bytes));
            generations.push(generation);
        }
        Ok(generations)
    }
}

fn digest(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(DIGEST_DOMAIN);
    hasher.update(bytes);
    hasher.finalize().into()
}

fn corrupt(detail: impl Into<String>) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::data::{BranchId, CommitId};
    use crate::identity::data::{EntityId, PartitionId};
    use crate::indexes::data::{
        DerivedIndexApplicability, DerivedIndexEntries, DerivedIndexPublicationStatus,
    };
    use crate::storage::data::AuthoritativeFieldComparisonKey;
    use worth_foundational::facade::AspectValue;

    fn generation(id: u64, changed_key: u64) -> DerivedIndexGeneration {
        let entries = (0..100)
            .map(|key| {
                let value = if key == changed_key { id } else { key };
                (
                    AuthoritativeFieldComparisonKey::from_aspect_value(&AspectValue::UInt64(key)),
                    vec![EntityId::new(PartitionId::main(), value, 1)],
                )
            })
            .collect::<BTreeMap<_, _>>()
            .into();
        DerivedIndexGeneration {
            generation_id: DerivedIndexGenerationId(id),
            index_id: DerivedIndexId(1),
            source_commit_id: CommitId(id),
            source_branch_id: BranchId("main".into()),
            applicability: DerivedIndexApplicability {
                branch_id: BranchId("main".into()),
                version_id: VersionId(id),
                schema_version: SchemaVersionId(1),
            },
            status: DerivedIndexPublicationStatus::Published,
            entries: DerivedIndexEntries::EntityField(entries),
        }
    }

    #[test]
    fn delta_checkpoint_roundtrips_exact_payload_and_shares_unchanged_bytes() {
        let generations = vec![generation(1, 90), generation(2, 90)];
        let checkpoint = DerivedIndexCheckpointArtifacts::capture(generations.clone()).unwrap();
        let wire = rmp_serde::to_vec_named(&checkpoint).unwrap();
        let decoded: DerivedIndexCheckpointArtifacts = rmp_serde::from_slice(&wire).unwrap();
        assert_eq!(decoded.readmit().unwrap(), generations);
        assert!(decoded.generations[1].shared_suffix_bytes > 0);
        assert!(wire.len() < rmp_serde::to_vec_named(&generations).unwrap().len());
    }

    #[test]
    fn delta_checkpoint_denies_corrupt_missing_and_incompatible_payloads() {
        let generations = vec![generation(1, 10), generation(2, 90)];
        let mut checkpoint = DerivedIndexCheckpointArtifacts::capture(generations).unwrap();
        checkpoint.generations[1].changed_bytes[0] ^= 1;
        assert!(checkpoint.readmit().is_err());
        let mut checkpoint =
            DerivedIndexCheckpointArtifacts::capture(vec![generation(1, 10)]).unwrap();
        checkpoint.generations.clear();
        assert!(checkpoint.readmit().is_err());
        checkpoint.format_version = 2;
        assert!(checkpoint.readmit().is_err());
    }
}
