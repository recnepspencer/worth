use super::MergeWitnessRows;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::sync::Arc;

use super::correspondence_witness_rows::{
    RelationalMergeCorrespondenceWitnessPosture, RelationalMergeCorrespondenceWitnessRow,
};
use crate::merge::data::{IdentityMatchCandidate, IdentityResolutionReason};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelationalMergeCorrespondenceWitness {
    request_digest: String,
    branch_basis_digest: String,
    rows: MergeWitnessRows<RelationalMergeCorrespondenceWitnessRow>,
    witness_digest: String,
}

impl RelationalMergeCorrespondenceWitness {
    pub(crate) fn retain_rows_memory(
        &self,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Result<(), Option<worth_execution::ExecutionMemoryReservation>> {
        self.rows.retain_decoded_memory(memory)
    }

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
            rows: rows.into(),
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
    rows: MergeWitnessRows<RelationalMergeCorrespondenceWitnessRow>,
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
            let expected_posture =
                if row.reason() == &IdentityResolutionReason::SchemaDeclaredCorrespondence {
                    let source_count =
                        record_occurrences(&schema_counts.source_records, row.source_record());
                    let target_count = row.target_record().map_or(0, |target| {
                        record_occurrences(&schema_counts.target_records, target)
                    });
                    schema_declared_correspondence_posture(source_count, target_count)
                } else {
                    correspondence_posture_for_match_class(row.match_class())
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
    correspondence_posture_for_match_class(&candidate.match_class)
}

fn correspondence_posture_for_match_class(
    class: &crate::merge::data::IdentityMatchClass,
) -> RelationalMergeCorrespondenceWitnessPosture {
    match class {
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
    let mut digest = Sha256::new();
    digest.update(b"worth.relational.merge.correspondence_witness.v1");
    digest.update(request_digest.as_bytes());
    digest.update(branch_basis_digest.as_bytes());
    {
        let mut writer = CorrespondenceHashWriter(&mut digest);
        write!(&mut writer, "{}", rows.len()).expect("hash writing cannot fail");
        for row in rows {
            rmp_serde::encode::write_named(&mut writer, row)
                .expect("closed native correspondence row must encode");
        }
    }
    lowercase_sha256(digest.finalize().into())
}

pub(super) fn lowercase_sha256(digest: [u8; 32]) -> String {
    let mut text = String::with_capacity(64);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in digest {
        text.push(HEX[usize::from(byte >> 4)] as char);
        text.push(HEX[usize::from(byte & 15)] as char);
    }
    text
}

pub(super) struct CorrespondenceHashWriter<'a>(pub(super) &'a mut Sha256);
impl Write for CorrespondenceHashWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

struct SchemaDeclaredCorrespondenceCounts {
    source_records: Vec<crate::transactions::data::RecordRef>,
    target_records: Vec<crate::transactions::data::RecordRef>,
}

fn schema_declared_correspondence_counts(
    rows: &[RelationalMergeCorrespondenceWitnessRow],
) -> SchemaDeclaredCorrespondenceCounts {
    let mut source_records = Vec::with_capacity(rows.len());
    let mut target_records = Vec::with_capacity(rows.len());
    for row in rows
        .iter()
        .filter(|row| row.reason() == &IdentityResolutionReason::SchemaDeclaredCorrespondence)
    {
        source_records.push(row.source_record().clone());
        if let Some(target) = row.target_record() {
            target_records.push(target.clone());
        }
    }
    source_records.sort_unstable();
    target_records.sort_unstable();
    SchemaDeclaredCorrespondenceCounts {
        source_records,
        target_records,
    }
}
fn record_occurrences(
    records: &[crate::transactions::data::RecordRef],
    target: &crate::transactions::data::RecordRef,
) -> usize {
    records.partition_point(|record| record <= target)
        - records.partition_point(|record| record < target)
}

fn digest_is_lowercase_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::data::{EntityId, PartitionId};
    use crate::merge::data::{IdentityBasisKind, IdentityMatchClass, MergeRecordIdentity};
    use crate::transactions::data::RecordRef;
    #[test]
    fn owned_decode_correspondence_digest_and_frequency_readmission_keep_original_codec() {
        let source = RecordRef::Entity(EntityId::new(PartitionId(1), 2, 1));
        let rows = [3, 4]
            .into_iter()
            .map(|slot| {
                let target = RecordRef::Entity(EntityId::new(PartitionId(1), slot, 1));
                let candidate = IdentityMatchCandidate {
                    scope: None,
                    source_record: source.clone(),
                    target_record: Some(target.clone()),
                    source: MergeRecordIdentity::StorageRecord(source.clone()),
                    target: Some(MergeRecordIdentity::StorageRecord(target)),
                    match_class: IdentityMatchClass::Exact,
                    reason: IdentityResolutionReason::SchemaDeclaredCorrespondence,
                    basis: IdentityBasisKind::StorageIdentity,
                };
                let mut original_candidate =
                    b"worth.relational.merge.identity_match_candidate.v1".to_vec();
                original_candidate.extend(rmp_serde::to_vec_named(&candidate).unwrap());
                let expected_candidate = format!("{:x}", Sha256::digest(original_candidate));
                let row =
                    row_for_candidate(&candidate, schema_declared_correspondence_posture(2, 1));
                assert_eq!(row.candidate_digest(), expected_candidate);
                row
            })
            .collect::<Vec<_>>();
        let request = "a".repeat(64);
        let branch = "b".repeat(64);
        let mut original = b"worth.relational.merge.correspondence_witness.v1".to_vec();
        original.extend_from_slice(request.as_bytes());
        original.extend_from_slice(branch.as_bytes());
        original.extend_from_slice(rows.len().to_string().as_bytes());
        for row in &rows {
            original.extend(rmp_serde::to_vec_named(row).unwrap());
        }
        let expected = format!("{:x}", Sha256::digest(original));
        let witness =
            RelationalMergeCorrespondenceWitness::retained(request, branch, Arc::from(rows));
        assert_eq!(witness.witness_digest(), expected);
        let bytes = rmp_serde::to_vec_named(&witness).unwrap();
        assert_eq!(
            rmp_serde::from_slice::<RelationalMergeCorrespondenceWitness>(&bytes).unwrap(),
            witness
        );
    }
}
