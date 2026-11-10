use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::runtime::{
    WorthQueryConcurrentHostileMatrixCounterSnapshot, WorthQueryConcurrentHostileMatrixTopology,
};

use super::posture::{
    classify_concurrent_hostile_matrix_posture, WorthQueryConcurrentHostileMatrixPosture,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryConcurrentHostileMatrixArtifact {
    topology: WorthQueryConcurrentHostileMatrixTopology,
    receipt_digests: Vec<String>,
    reader_result_digests: Vec<String>,
    published_artifact_digests: Vec<String>,
    preview_closeout_digests: Vec<String>,
    branch_basis_digests: Vec<String>,
    replay_digest: String,
    counters: WorthQueryConcurrentHostileMatrixCounterSnapshot,
    artifact_replay_equal: bool,
    repeated_run_equal: bool,
    sabotage_sensitive: bool,
    posture: WorthQueryConcurrentHostileMatrixPosture,
    digest: WorthQueryEvidenceIdentity,
}

impl WorthQueryConcurrentHostileMatrixArtifact {
    #[allow(clippy::too_many_arguments)]
    pub fn certify(
        topology: WorthQueryConcurrentHostileMatrixTopology,
        receipt_digests: Vec<String>,
        reader_result_digests: Vec<String>,
        published_artifact_digests: Vec<String>,
        preview_closeout_digests: Vec<String>,
        branch_basis_digests: Vec<String>,
        replay_digest: String,
        counters: WorthQueryConcurrentHostileMatrixCounterSnapshot,
        artifact_replay_equal: bool,
        repeated_run_equal: bool,
        sabotage_sensitive: bool,
    ) -> Result<Self, IdentityDerivationDenial> {
        let posture = classify_concurrent_hostile_matrix_posture(
            topology.satisfies_phase_sixteen_minimums(),
            artifact_replay_equal,
            repeated_run_equal,
            counters.exact_zero_residue_count(),
            counters.published_artifact_registry_lease_count(),
            sabotage_sensitive,
        );
        let digest = Self::compose_digest(
            topology,
            &receipt_digests,
            &reader_result_digests,
            &published_artifact_digests,
            &preview_closeout_digests,
            &branch_basis_digests,
            &replay_digest,
            &counters,
            posture,
        )?;
        Ok(Self {
            topology,
            receipt_digests,
            reader_result_digests,
            published_artifact_digests,
            preview_closeout_digests,
            branch_basis_digests,
            replay_digest,
            counters,
            artifact_replay_equal,
            repeated_run_equal,
            sabotage_sensitive,
            posture,
            digest,
        })
    }

    pub fn posture(&self) -> WorthQueryConcurrentHostileMatrixPosture {
        self.posture
    }

    pub fn counters(&self) -> &WorthQueryConcurrentHostileMatrixCounterSnapshot {
        &self.counters
    }

    pub fn topology(&self) -> WorthQueryConcurrentHostileMatrixTopology {
        self.topology
    }

    pub fn digest(&self) -> &WorthQueryEvidenceIdentity {
        &self.digest
    }

    pub fn replay_digest(&self) -> &str {
        &self.replay_digest
    }

    pub(crate) fn artifact_replay_equal(&self) -> bool {
        self.artifact_replay_equal
    }

    pub(crate) fn repeated_run_equal(&self) -> bool {
        self.repeated_run_equal
    }

    pub(crate) fn sabotage_sensitive(&self) -> bool {
        self.sabotage_sensitive
    }

    fn compose_digest(
        topology: WorthQueryConcurrentHostileMatrixTopology,
        receipt_digests: &[String],
        reader_result_digests: &[String],
        published_artifact_digests: &[String],
        preview_closeout_digests: &[String],
        branch_basis_digests: &[String],
        replay_digest: &str,
        counters: &WorthQueryConcurrentHostileMatrixCounterSnapshot,
        posture: WorthQueryConcurrentHostileMatrixPosture,
    ) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::RuntimeHostileCertificationArtifact,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("reader_thread_count"),
            topology.reader_thread_count(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("submitter_thread_count"),
            topology.submitter_thread_count(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("submission_round_count"),
            topology.submission_round_count(),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("receipt_digest"),
            receipt_digests.iter().map(String::as_str),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("reader_result_digest"),
            reader_result_digests.iter().map(String::as_str),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("published_artifact_digest"),
            published_artifact_digests.iter().map(String::as_str),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("preview_closeout_digest"),
            preview_closeout_digests.iter().map(String::as_str),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("branch_basis_digest"),
            branch_basis_digests.iter().map(String::as_str),
        )
        .field_value(WorthQueryEvidenceTag::new("replay_digest"), replay_digest)
        .field_usize(
            WorthQueryEvidenceTag::new("counter_residue_count"),
            counters.exact_zero_residue_count(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("registry_lease_count"),
            counters.published_artifact_registry_lease_count(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("phase_sixteen_posture"),
            posture.as_str(),
        )
        .seal()
    }
}
