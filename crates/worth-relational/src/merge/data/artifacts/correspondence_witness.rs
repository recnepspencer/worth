use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::Arc;

use super::correspondence_witness_rows::{
    RelationalMergeCorrespondenceWitnessPosture, RelationalMergeCorrespondenceWitnessRow,
};
use crate::merge::data::{IdentityMatchCandidate, IdentityResolutionReason};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelationalMergeCorrespondenceWitness {
    request_digest: String,
    branch_basis_digest: String,
    rows: Arc<[RelationalMergeCorrespondenceWitnessRow]>,
    witness_digest: String,
}

impl RelationalMergeCorrespondenceWitness {
    pub(crate) fn retained(
        request_digest: String,
        branch_basis_digest: String,
        rows: Arc<[RelationalMergeCorrespondenceWitnessRow]>,
    ) -> Self {
        let witness_digest =
            merge_correspondence_witness_digest(&request_digest, &branch_basis_digest, &rows);
        Self {
            request_digest,
            branch_basis_digest,
            rows,
            witness_digest,
        }
    }

    pub fn request_digest(&self) -> &str {
        &self.request_digest
    }

    pub fn branch_basis_digest(&self) -> &str {
        &self.branch_basis_digest
    }

    pub fn rows(&self) -> &[RelationalMergeCorrespondenceWitnessRow] {
        &self.rows
    }

    pub fn admitted_rows(&self) -> impl Iterator<Item = &RelationalMergeCorrespondenceWitnessRow> {
        self.rows
            .iter()
            .filter(|row| row.posture() == RelationalMergeCorrespondenceWitnessPosture::Admitted)
    }

    pub fn witness_digest(&self) -> &str {
        &self.witness_digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct RelationalMergeCorrespondenceWitnessWire {
    request_digest: String,
    branch_basis_digest: String,
    rows: Arc<[RelationalMergeCorrespondenceWitnessRow]>,
    witness_digest: String,
}

impl<'de> Deserialize<'de> for RelationalMergeCorrespondenceWitness {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RelationalMergeCorrespondenceWitnessWire::deserialize(deserializer)?;
        if !digest_is_lowercase_sha256_hex(&wire.request_digest) {
            return Err(D::Error::custom(
                "merge correspondence witness request digest is not valid lowercase sha256 hex",
            ));
        }
        if !digest_is_lowercase_sha256_hex(&wire.branch_basis_digest) {
            return Err(D::Error::custom(
                "merge correspondence witness branch basis digest is not valid lowercase sha256 hex",
            ));
        }
        for row in wire.rows.iter() {
            if !digest_is_lowercase_sha256_hex(row.candidate_digest()) {
                return Err(D::Error::custom(
                    "merge correspondence witness row candidate digest is not valid lowercase sha256 hex",
                ));
            }
        }
        let schema_counts = schema_declared_correspondence_counts(&wire.rows);
        for row in wire.rows.iter() {
            let expected_posture = if row.candidate().reason
                == IdentityResolutionReason::SchemaDeclaredCorrespondence
            {
                let source_count = schema_counts
                    .source_counts
                    .get(&row.candidate().source_record)
                    .copied()
                    .unwrap_or(0);
                let target_count = row
                    .candidate()
                    .target_record
                    .as_ref()
                    .and_then(|target| schema_counts.target_counts.get(target).copied())
                    .unwrap_or(0);
                schema_declared_correspondence_posture(source_count, target_count)
            } else {
                correspondence_posture_for_candidate(&row.candidate())
            };
            if row.posture() != expected_posture {
                return Err(D::Error::custom(
                    "merge correspondence witness row posture does not match retained correspondence admission truth",
                ));
            }
        }
        let witness_digest = merge_correspondence_witness_digest(
            &wire.request_digest,
            &wire.branch_basis_digest,
            &wire.rows,
        );
        if witness_digest != wire.witness_digest {
            return Err(D::Error::custom(
                "merge correspondence witness digest does not match retained correspondence truth",
            ));
        }
        Ok(Self {
            request_digest: wire.request_digest,
            branch_basis_digest: wire.branch_basis_digest,
            rows: wire.rows,
            witness_digest: wire.witness_digest,
        })
    }
}

pub(crate) fn schema_declared_correspondence_posture(
    source_count: usize,
    target_count: usize,
) -> RelationalMergeCorrespondenceWitnessPosture {
    match (source_count > 1, target_count > 1) {
        (true, true) => {
            RelationalMergeCorrespondenceWitnessPosture::DeniedSchemaNonUniqueSourceAndTarget
        }
        (true, false) => RelationalMergeCorrespondenceWitnessPosture::DeniedSchemaNonUniqueSource,
        (false, true) => RelationalMergeCorrespondenceWitnessPosture::DeniedSchemaNonUniqueTarget,
        (false, false) => RelationalMergeCorrespondenceWitnessPosture::Admitted,
    }
}

pub(crate) fn correspondence_posture_for_candidate(
    candidate: &IdentityMatchCandidate,
) -> RelationalMergeCorrespondenceWitnessPosture {
    match candidate.match_class {
        crate::merge::data::IdentityMatchClass::Exact
        | crate::merge::data::IdentityMatchClass::Reconciliable => {
            RelationalMergeCorrespondenceWitnessPosture::Admitted
        }
        crate::merge::data::IdentityMatchClass::Ambiguous => {
            RelationalMergeCorrespondenceWitnessPosture::DeniedAmbiguous
        }
        crate::merge::data::IdentityMatchClass::MissingTarget => {
            RelationalMergeCorrespondenceWitnessPosture::UnavailableMissingTarget
        }
    }
}

pub(crate) fn row_for_candidate(
    candidate: &IdentityMatchCandidate,
    posture: RelationalMergeCorrespondenceWitnessPosture,
) -> RelationalMergeCorrespondenceWitnessRow {
    RelationalMergeCorrespondenceWitnessRow::from_candidate(candidate, posture)
}

fn merge_correspondence_witness_digest(
    request_digest: &str,
    branch_basis_digest: &str,
    rows: &[RelationalMergeCorrespondenceWitnessRow],
) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"worth.relational.merge.correspondence_witness.v1");
    bytes.extend_from_slice(request_digest.as_bytes());
    bytes.extend_from_slice(branch_basis_digest.as_bytes());
    bytes.extend_from_slice(rows.len().to_string().as_bytes());
    for row in rows {
        bytes.extend_from_slice(
            &rmp_serde::to_vec_named(row).expect(/* Proof: The serialized value contains only closed native scalar, sequence, map and derived-Serialize record fields; encoding uses an in-memory Vec/hash writer, with no user serializer or fallible I/O. */ "merge correspondence witness row must encode"),
        );
    }
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

struct SchemaDeclaredCorrespondenceCounts {
    source_counts: BTreeMap<crate::transactions::data::RecordRef, usize>,
    target_counts: BTreeMap<crate::transactions::data::RecordRef, usize>,
}

fn schema_declared_correspondence_counts(
    rows: &[RelationalMergeCorrespondenceWitnessRow],
) -> SchemaDeclaredCorrespondenceCounts {
    let mut source_counts = BTreeMap::new();
    let mut target_counts = BTreeMap::new();
    for row in rows.iter().filter(|row| {
        row.candidate().reason == IdentityResolutionReason::SchemaDeclaredCorrespondence
    }) {
        *source_counts
            .entry(row.candidate().source_record)
            .or_insert(0) += 1;
        if let Some(target_record) = row.candidate().target_record {
            *target_counts.entry(target_record).or_insert(0) += 1;
        }
    }
    SchemaDeclaredCorrespondenceCounts {
        source_counts,
        target_counts,
    }
}

fn digest_is_lowercase_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
