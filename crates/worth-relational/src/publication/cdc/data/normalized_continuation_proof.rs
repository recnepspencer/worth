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
pub struct NormalizedContinuationProof {
    boundary_fingerprints: Vec<SchemaBoundaryFingerprint>,
    descriptor_semantics_version: DescriptorSemanticsVersion,
    normalized_boundary_count: usize,
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
