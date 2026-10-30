//! Record and equivalence evidence retained by a structural fingerprint.
use crate::snapshot::validated_value_basis::validated_snapshot_read_value_canonical_basis;
use crate::snapshot::ValidatedSnapshotReadRecord;
use sha2::{Digest, Sha256};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralFingerprintRecordValueEvidenceSet {
    records: Arc<[StructuralFingerprintRecordValueEvidence]>,
    canonical_basis: Arc<str>,
}

impl StructuralFingerprintRecordValueEvidenceSet {
    pub(super) fn from_validated_records(records: &[ValidatedSnapshotReadRecord]) -> Self {
        let mut evidence = records
            .iter()
            .map(StructuralFingerprintRecordValueEvidence::from_validated_record)
            .collect::<Vec<_>>();
        evidence.sort_by(|left, right| left.correlation_id().cmp(right.correlation_id()));
        Self::from_evidence(evidence)
    }

    pub(super) fn empty() -> Self {
        Self::from_evidence(Vec::new())
    }

    fn from_evidence(records: Vec<StructuralFingerprintRecordValueEvidence>) -> Self {
        let canonical_basis = structural_record_value_evidence_set_canonical_basis(&records);
        Self {
            records: Arc::from(records),
            canonical_basis,
        }
    }

    pub fn records(&self) -> &[StructuralFingerprintRecordValueEvidence] {
        &self.records
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralFingerprintRecordValueEvidence {
    correlation_id: Arc<str>,
    aspect_value_digest: Arc<str>,
    canonical_basis: Arc<str>,
}

impl StructuralFingerprintRecordValueEvidence {
    pub(super) fn from_validated_record(record: &ValidatedSnapshotReadRecord) -> Self {
        let value_basis = record
            .validated_value_posture()
            .map(validated_snapshot_read_value_canonical_basis)
            .unwrap_or_else(|| "validated-snapshot-read-value|posture=absent".to_string());
        let aspect_value_digest = Sha256::digest(value_basis.as_bytes());
        let aspect_value_digest = Arc::<str>::from(format!(
            "structural-record-aspect-value:sha256:{aspect_value_digest:x}"
        ));
        let correlation_id = Arc::<str>::from(record.correlation_id().as_str());
        let canonical_basis = Arc::<str>::from(format!(
            "structural-record-value-evidence|correlation={}|aspect-value={}",
            correlation_id.as_ref(),
            aspect_value_digest.as_ref(),
        ));
        Self {
            correlation_id,
            aspect_value_digest,
            canonical_basis,
        }
    }

    pub fn correlation_id(&self) -> &str {
        self.correlation_id.as_ref()
    }

    pub fn aspect_value_digest(&self) -> &str {
        self.aspect_value_digest.as_ref()
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralFingerprintEquivalenceMemberSet {
    members: Arc<[StructuralFingerprintEquivalenceMemberEvidence]>,
    canonical_basis: Arc<str>,
}

impl StructuralFingerprintEquivalenceMemberSet {
    pub(super) fn from_record_value_evidence(
        records: &StructuralFingerprintRecordValueEvidenceSet,
    ) -> Self {
        let members = records
            .records()
            .iter()
            .map(StructuralFingerprintEquivalenceMemberEvidence::from_record_value_evidence)
            .collect::<Vec<_>>();
        Self::from_members(members)
    }

    pub(super) fn empty() -> Self {
        Self::from_members(Vec::new())
    }

    fn from_members(members: Vec<StructuralFingerprintEquivalenceMemberEvidence>) -> Self {
        let canonical_basis = structural_equivalence_member_set_canonical_basis(&members);
        Self {
            members: Arc::from(members),
            canonical_basis,
        }
    }

    pub fn members(&self) -> &[StructuralFingerprintEquivalenceMemberEvidence] {
        &self.members
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralFingerprintEquivalenceMemberEvidence {
    aspect_value_digest: Arc<str>,
    canonical_basis: Arc<str>,
}

impl StructuralFingerprintEquivalenceMemberEvidence {
    pub(super) fn from_record_value_evidence(
        record: &StructuralFingerprintRecordValueEvidence,
    ) -> Self {
        let aspect_value_digest = Arc::<str>::from(record.aspect_value_digest());
        let canonical_basis = Arc::<str>::from(format!(
            "structural-equivalence-member|aspect-value={}",
            aspect_value_digest.as_ref(),
        ));
        Self {
            aspect_value_digest,
            canonical_basis,
        }
    }

    pub fn aspect_value_digest(&self) -> &str {
        self.aspect_value_digest.as_ref()
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }
}

fn structural_record_value_evidence_set_canonical_basis(
    records: &[StructuralFingerprintRecordValueEvidence],
) -> Arc<str> {
    Arc::<str>::from(format!(
        "structural-record-value-evidence-set|count={}|records={}",
        records.len(),
        records
            .iter()
            .map(StructuralFingerprintRecordValueEvidence::canonical_basis)
            .collect::<Vec<_>>()
            .join("|"),
    ))
}

fn structural_equivalence_member_set_canonical_basis(
    members: &[StructuralFingerprintEquivalenceMemberEvidence],
) -> Arc<str> {
    Arc::<str>::from(format!(
        "structural-equivalence-member-set|count={}|members={}",
        members.len(),
        members
            .iter()
            .map(StructuralFingerprintEquivalenceMemberEvidence::canonical_basis)
            .collect::<Vec<_>>()
            .join("|"),
    ))
}
