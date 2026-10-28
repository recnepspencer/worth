//! Bounded, deduplicated continuation evidence is admitted before publication.
use super::subscriber_contract::MAX_NORMALIZED_CONTINUATION_BOUNDARIES;
use crate::schema::data::{DescriptorSemanticsVersion, SchemaBoundaryFingerprint};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NormalizedContinuationProofLimitExceeded {
    pub(crate) actual: usize,
    pub(crate) maximum: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "NormalizedContinuationProofInput")]
pub struct NormalizedContinuationProof {
    boundary_fingerprints: Vec<SchemaBoundaryFingerprint>,
    descriptor_semantics_version: DescriptorSemanticsVersion,
    normalized_boundary_count: usize,
}

#[derive(Deserialize)]
struct NormalizedContinuationProofInput {
    boundary_fingerprints: Vec<SchemaBoundaryFingerprint>,
    descriptor_semantics_version: DescriptorSemanticsVersion,
    normalized_boundary_count: usize,
}

impl TryFrom<NormalizedContinuationProofInput> for NormalizedContinuationProof {
    type Error = &'static str;

    fn try_from(input: NormalizedContinuationProofInput) -> Result<Self, Self::Error> {
        if input.boundary_fingerprints.len() > MAX_NORMALIZED_CONTINUATION_BOUNDARIES {
            return Err("continuation proof exceeds the normalized boundary limit");
        }
        if input.normalized_boundary_count != input.boundary_fingerprints.len() {
            return Err("continuation proof count disagrees with its boundaries");
        }
        let proof = Self::new(
            input.boundary_fingerprints,
            input.descriptor_semantics_version,
        )
        .map_err(|_| "continuation proof exceeds the normalized boundary limit")?;
        if proof.normalized_boundary_count != input.normalized_boundary_count {
            return Err("continuation proof boundaries are not deduplicated");
        }
        Ok(proof)
    }
}

impl NormalizedContinuationProof {
    pub(crate) fn new(
        boundary_fingerprints: Vec<SchemaBoundaryFingerprint>,
        descriptor_semantics_version: DescriptorSemanticsVersion,
    ) -> Result<Self, NormalizedContinuationProofLimitExceeded> {
        let mut deduped = BTreeSet::new();
        let mut normalized = Vec::with_capacity(boundary_fingerprints.len());
        for fingerprint in boundary_fingerprints {
            if deduped.insert(fingerprint) {
                normalized.push(fingerprint);
            }
        }
        if normalized.len() > MAX_NORMALIZED_CONTINUATION_BOUNDARIES {
            return Err(NormalizedContinuationProofLimitExceeded {
                actual: normalized.len(),
                maximum: MAX_NORMALIZED_CONTINUATION_BOUNDARIES,
            });
        }
        Ok(Self {
            normalized_boundary_count: normalized.len(),
            boundary_fingerprints: normalized,
            descriptor_semantics_version,
        })
    }

    pub(crate) fn empty(descriptor_semantics_version: DescriptorSemanticsVersion) -> Self {
        Self {
            boundary_fingerprints: Vec::new(),
            descriptor_semantics_version,
            normalized_boundary_count: 0,
        }
    }

    pub fn boundary_fingerprints(&self) -> &[SchemaBoundaryFingerprint] {
        &self.boundary_fingerprints
    }

    pub fn descriptor_semantics_version(&self) -> DescriptorSemanticsVersion {
        self.descriptor_semantics_version
    }

    pub fn normalized_boundary_count(&self) -> usize {
        self.normalized_boundary_count
    }

    #[cfg(test)]
    pub(crate) fn from_raw_parts_for_test(
        boundary_fingerprints: Vec<SchemaBoundaryFingerprint>,
        descriptor_semantics_version: DescriptorSemanticsVersion,
        normalized_boundary_count: usize,
    ) -> Self {
        Self {
            boundary_fingerprints,
            descriptor_semantics_version,
            normalized_boundary_count,
        }
    }
}

impl Default for NormalizedContinuationProof {
    fn default() -> Self {
        Self::empty(DescriptorSemanticsVersion::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn continuation_deserialization_checks_limit_count_and_normalization() {
        let fingerprints = (0..65)
            .map(|n| {
                let mut bytes = [0; 32];
                bytes[..8].copy_from_slice(&(n as u64).to_le_bytes());
                SchemaBoundaryFingerprint::new(bytes)
            })
            .collect::<Vec<_>>();
        let version = DescriptorSemanticsVersion::default();
        for (boundaries, count) in [
            (fingerprints.clone(), 65),
            (fingerprints.clone(), 0),
            (fingerprints[..1].to_vec(), 65),
            (vec![fingerprints[0]; 2], 2),
        ] {
            let bytes = rmp_serde::to_vec(&(boundaries, version, count)).unwrap();
            assert!(rmp_serde::from_slice::<NormalizedContinuationProof>(&bytes).is_err());
        }
        for boundaries in [Vec::new(), fingerprints[..64].to_vec()] {
            let admitted = NormalizedContinuationProof::new(boundaries, version).unwrap();
            let bytes = rmp_serde::to_vec(&admitted).unwrap();
            assert_eq!(
                rmp_serde::from_slice::<NormalizedContinuationProof>(&bytes).unwrap(),
                admitted
            );
        }
    }

    #[test]
    fn continuation_constructor_refuses_only_the_normalized_boundary_excess() {
        let fingerprints = (0..=MAX_NORMALIZED_CONTINUATION_BOUNDARIES)
            .map(|n| {
                let mut bytes = [0; 32];
                bytes[..8].copy_from_slice(&(n as u64).to_le_bytes());
                SchemaBoundaryFingerprint::new(bytes)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            NormalizedContinuationProof::new(
                fingerprints.clone(),
                DescriptorSemanticsVersion::default()
            ),
            Err(NormalizedContinuationProofLimitExceeded {
                actual: 65,
                maximum: 64
            })
        );
        let repeated = vec![fingerprints[0]; MAX_NORMALIZED_CONTINUATION_BOUNDARIES + 1];
        assert_eq!(
            NormalizedContinuationProof::new(repeated, DescriptorSemanticsVersion::default())
                .unwrap()
                .normalized_boundary_count(),
            1
        );
        assert_eq!(
            NormalizedContinuationProof::new(
                fingerprints[..64].to_vec(),
                DescriptorSemanticsVersion::default()
            )
            .unwrap()
            .normalized_boundary_count(),
            64
        );
    }
}
