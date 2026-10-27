use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use worth_foundational::{FoundationalBranchTargetBasis, FoundationalBranchTargetEncoding};

/// Relational's immutable descriptive target for one exact branch reference.
/// Runtime identity is part of the target so equal commit ordinals from
/// different runtimes cannot compare as the same basis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalBranchTarget {
    runtime_instance_id: u64,
    /// A basis target is always a committed source. The shared `Empty`
    /// variant represents the absence of a committed target; `None` is not a
    /// second empty-like state inside a `Basis` descriptor.
    #[serde(rename = "commit_id")]
    selected_commit_id: u64,
    version_id: u64,
    parent_commit_ids: Vec<u64>,
    roots: RelationalBranchRootDescriptor,
}

/// The immutable truth and schema roots selected by one relational commit.
///
/// These roots are descriptive identity, not currentness or authority. The
/// owner supplies them when it lowers a production commit into the shared
/// branch-reference grammar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalBranchRootDescriptor {
    truth_root: [u8; 32],
    schema_root: [u8; 32],
}

impl RelationalBranchRootDescriptor {
    pub const fn new(truth_root: [u8; 32], schema_root: [u8; 32]) -> Self {
        Self {
            truth_root,
            schema_root,
        }
    }

    pub const fn truth_root(&self) -> &[u8; 32] {
        &self.truth_root
    }

    pub const fn schema_root(&self) -> &[u8; 32] {
        &self.schema_root
    }
}

impl RelationalBranchTarget {
    pub(crate) fn new(
        runtime_instance_id: u64,
        selected_commit_id: u64,
        version_id: u64,
        parent_commit_ids: Vec<u64>,
        roots: RelationalBranchRootDescriptor,
    ) -> Self {
        Self {
            runtime_instance_id,
            selected_commit_id,
            version_id,
            parent_commit_ids,
            roots,
        }
    }

    pub const fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }

    pub const fn selected_commit_id(&self) -> u64 {
        self.selected_commit_id
    }

    pub const fn version_id(&self) -> u64 {
        self.version_id
    }

    pub fn parent_commit_ids(&self) -> &[u64] {
        &self.parent_commit_ids
    }

    pub const fn roots(&self) -> &RelationalBranchRootDescriptor {
        &self.roots
    }

    pub(crate) fn rebind_runtime_instance_id(&self, runtime_instance_id: u64) -> Self {
        Self {
            runtime_instance_id,
            selected_commit_id: self.selected_commit_id,
            version_id: self.version_id,
            parent_commit_ids: self.parent_commit_ids.clone(),
            roots: self.roots.clone(),
        }
    }
}

impl RelationalBranchTarget {
    pub(crate) fn roots_for_commit(
        reference: &crate::history::data::RelationalCommitReceipt,
    ) -> RelationalBranchRootDescriptor {
        let mut bytes = Vec::with_capacity(24 + reference.parents.len() * 8);
        bytes.extend_from_slice(&reference.commit_id.0.to_be_bytes());
        bytes.extend_from_slice(&reference.version_id.0.to_be_bytes());
        bytes.extend_from_slice(&(reference.parents.len() as u64).to_be_bytes());
        for parent in &reference.parents {
            bytes.extend_from_slice(&parent.0.to_be_bytes());
        }
        let mut truth_input = b"worth.relational.phase4.truth-root\0".to_vec();
        truth_input.extend_from_slice(&bytes);
        let mut schema_input = b"worth.relational.phase4.schema-root\0".to_vec();
        schema_input.extend_from_slice(&bytes);
        RelationalBranchRootDescriptor::new(
            Sha256::digest(truth_input).into(),
            Sha256::digest(schema_input).into(),
        )
    }

    pub(crate) fn from_commit_receipt(
        runtime_instance_id: u64,
        reference: &crate::history::data::RelationalCommitReceipt,
        roots: RelationalBranchRootDescriptor,
    ) -> Self {
        Self::new(
            runtime_instance_id,
            reference.commit_id.0,
            reference.version_id.0,
            reference.parents.iter().map(|parent| parent.0).collect(),
            roots,
        )
    }
}

impl FoundationalBranchTargetBasis for RelationalBranchTarget {
    fn canonical_encoding(&self) -> FoundationalBranchTargetEncoding {
        let mut bytes = Vec::new();
        write_u64(&mut bytes, self.runtime_instance_id);
        write_u64(&mut bytes, self.selected_commit_id);
        write_u64(&mut bytes, self.version_id);
        write_u64(&mut bytes, self.parent_commit_ids.len() as u64);
        for parent in &self.parent_commit_ids {
            write_u64(&mut bytes, *parent);
        }
        bytes.extend_from_slice(self.roots.truth_root());
        bytes.extend_from_slice(self.roots.schema_root());
        FoundationalBranchTargetEncoding::new("worth.relational.branch-target", 2, bytes)
            .expect("static relational target encoding is valid")
    }
}

fn write_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::branch::relational_branch_observation;
    use crate::history::data::{BranchId, CommitId, RelationalCommitReceipt};
    use crate::identity::data::VersionId;
    use worth_foundational::{
        FoundationalBranchReferenceGeneration, FoundationalBranchReferenceMismatchAxis,
        FoundationalBranchTarget, FoundationalBranchTargetBasis,
    };

    fn commit() -> RelationalCommitReceipt {
        RelationalCommitReceipt {
            commit_id: CommitId(7),
            version_id: VersionId(3),
            branch_id: BranchId("source".to_owned()),
            parents: vec![CommitId(2)],
        }
    }

    #[test]
    fn commit_reference_lowers_to_runtime_affine_foundational_target() {
        let commit = commit();
        let roots = RelationalBranchRootDescriptor::new([1; 32], [2; 32]);
        let first = RelationalBranchTarget::from_commit_receipt(11, &commit, roots.clone());
        let second = RelationalBranchTarget::from_commit_receipt(12, &commit, roots);
        assert_ne!(first, second);
        let observation = relational_branch_observation(
            11,
            "storm",
            FoundationalBranchTarget::basis(first),
            FoundationalBranchReferenceGeneration::initial(),
        )
        .expect("owner branch lowers to the shared grammar");
        assert_eq!(observation.branch_id().as_str(), "relational/11/storm");
        assert_eq!(
            RelationalBranchTarget::from_commit_receipt(
                11,
                &commit,
                RelationalBranchRootDescriptor::new([1; 32], [2; 32]),
            )
            .canonical_encoding()
            .bytes(),
            hex_bytes(concat!(
                "000000000000000b00000000000000070000000000000003",
                "00000000000000010000000000000002",
                "0101010101010101010101010101010101010101010101010101010101010101",
                "0202020202020202020202020202020202020202020202020202020202020202",
            ))
        );
        let foreign_observation = relational_branch_observation(
            12,
            "storm",
            FoundationalBranchTarget::basis(second),
            FoundationalBranchReferenceGeneration::initial(),
        )
        .expect("foreign runtime lowers to the shared grammar");
        let mismatch = observation
            .compare(&foreign_observation)
            .expect_err("foreign runtime twins must not compare equal");
        assert_eq!(
            mismatch.axes(),
            &[
                FoundationalBranchReferenceMismatchAxis::BranchIdentity,
                FoundationalBranchReferenceMismatchAxis::TargetBasis,
            ]
        );
    }

    #[test]
    fn relational_target_roots_are_exact_observation_axes() {
        let commit = commit();
        let expected = relational_branch_observation(
            11,
            "storm",
            FoundationalBranchTarget::basis(RelationalBranchTarget::from_commit_receipt(
                11,
                &commit,
                RelationalBranchRootDescriptor::new([1; 32], [2; 32]),
            )),
            FoundationalBranchReferenceGeneration::initial(),
        )
        .expect("valid expected reference");
        let observed = relational_branch_observation(
            11,
            "storm",
            FoundationalBranchTarget::basis(RelationalBranchTarget::from_commit_receipt(
                11,
                &commit,
                RelationalBranchRootDescriptor::new([3; 32], [2; 32]),
            )),
            FoundationalBranchReferenceGeneration::initial(),
        )
        .expect("valid observed reference");
        let mismatch = expected
            .compare(&observed)
            .expect_err("truth roots are part of exact target identity");
        assert_eq!(
            mismatch.axes(),
            &[FoundationalBranchReferenceMismatchAxis::TargetBasis]
        );
    }

    #[test]
    fn relational_observation_rejects_foreign_target_runtime() {
        let denial = relational_branch_observation(
            11,
            "storm",
            FoundationalBranchTarget::basis(RelationalBranchTarget::from_commit_receipt(
                12,
                &commit(),
                RelationalBranchRootDescriptor::new([1; 32], [2; 32]),
            )),
            FoundationalBranchReferenceGeneration::initial(),
        )
        .expect_err("foreign runtime target must not cross the observation adapter");
        assert!(matches!(
            denial,
            crate::branch::RelationalBranchObservationConstructionDenial::RuntimeInstanceMismatch {
                observation_runtime_instance_id: 11,
                target_runtime_instance_id: 12,
            }
        ));
    }

    fn hex_bytes(value: &str) -> Vec<u8> {
        value
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| {
                u8::from_str_radix(std::str::from_utf8(pair).expect("hex is utf8"), 16)
                    .expect("valid hex")
            })
            .collect()
    }
}
