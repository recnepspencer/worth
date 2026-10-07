use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::identity::{BridgeIdentity, StructuralFingerprintIdentityTag};
use crate::snapshot::{
    MaterializedTruthViewObservation, SnapshotReadPacket, TruthSnapshotIdentity,
};

use super::{AdmittedStructuralComparisonContract, StructuralFingerprintFamily};

mod evidence;
pub use evidence::{
    StructuralFingerprintEquivalenceMemberEvidence, StructuralFingerprintEquivalenceMemberSet,
    StructuralFingerprintRecordValueEvidence, StructuralFingerprintRecordValueEvidenceSet,
};

pub type StructuralFingerprintIdentity = BridgeIdentity<StructuralFingerprintIdentityTag>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralFingerprint {
    fingerprint_identity: StructuralFingerprintIdentity,
    family: StructuralFingerprintFamily,
    semantics_version: Arc<str>,
    contract_identity: Arc<str>,
    truth_view_basis_digest: Arc<str>,
    read_packet: SnapshotReadPacket,
    planned_packet_digest: Arc<str>,
    snapshot_identity: TruthSnapshotIdentity,
    authority_digest: Arc<str>,
    equivalence_digest: Arc<str>,
    record_value_evidence: StructuralFingerprintRecordValueEvidenceSet,
    equivalence_member_evidence: StructuralFingerprintEquivalenceMemberSet,
    record_aspect_value_digest: Arc<str>,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl StructuralFingerprint {
    pub fn from_observation(
        contract: &AdmittedStructuralComparisonContract,
        observation: &MaterializedTruthViewObservation,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, crate::snapshot::BridgeSnapshotReadError> {
        let validated = observation.read_planned_packet(execution)?;
        let record_value_evidence =
            StructuralFingerprintRecordValueEvidenceSet::from_validated_records(
                validated.records(),
            );
        let equivalence_member_evidence =
            StructuralFingerprintEquivalenceMemberSet::from_record_value_evidence(
                &record_value_evidence,
            );
        Ok(Self::from_validated_read(
            contract,
            observation.planned().read_packet(),
            observation.snapshot_identity().clone(),
            record_value_evidence,
            equivalence_member_evidence,
        ))
    }

    pub fn from_snapshot_read_packet(
        contract: &AdmittedStructuralComparisonContract,
        read_packet: &SnapshotReadPacket,
        snapshot_identity: TruthSnapshotIdentity,
    ) -> Self {
        Self::from_validated_read(
            contract,
            read_packet,
            snapshot_identity,
            StructuralFingerprintRecordValueEvidenceSet::empty(),
            StructuralFingerprintEquivalenceMemberSet::empty(),
        )
    }

    fn from_validated_read(
        contract: &AdmittedStructuralComparisonContract,
        read_packet: &SnapshotReadPacket,
        snapshot_identity: TruthSnapshotIdentity,
        record_value_evidence: StructuralFingerprintRecordValueEvidenceSet,
        equivalence_member_evidence: StructuralFingerprintEquivalenceMemberSet,
    ) -> Self {
        let equivalence = contract
            .validated_declaration()
            .declaration()
            .equivalence_contract();
        let record_aspect_value_digest: Arc<str> = {
            let digest = Sha256::digest(record_value_evidence.canonical_basis().as_bytes());
            Arc::from(format!("structural-record-aspect-values:sha256:{digest:x}"))
        };
        let authority_digest: Arc<str> = {
            let digest = Sha256::digest(
                format!(
                    "structural-authority|snapshot={}|planned={}",
                    snapshot_identity.as_str(),
                    read_packet.digest()
                )
                .as_bytes(),
            );
            Arc::from(format!("structural-authority:sha256:{digest:x}"))
        };
        let equivalence_digest: Arc<str> = {
            let equivalence_basis = if equivalence_member_evidence.members().is_empty() {
                read_packet.digest().to_owned()
            } else {
                equivalence_member_evidence.canonical_basis().to_owned()
            };
            let digest = Sha256::digest(
                format!(
                    "structural-equivalence|family:{:?}|semantics={}|members={}",
                    equivalence.fingerprint_family(),
                    equivalence.semantics_version(),
                    equivalence_basis,
                )
                .as_bytes(),
            );
            Arc::from(format!("structural-equivalence:sha256:{digest:x}"))
        };
        let canonical_basis = Arc::<str>::from(format!(
            "structural-fingerprint|family:{:?}|semantics={}|contract={}|truth-view={}|planned={}|snapshot={}|authority={}|equivalence={}|record-values={}|equivalence-members={}",
            equivalence.fingerprint_family(),
            equivalence.semantics_version(),
            contract.contract_identity().as_str(),
            contract
                .validated_declaration()
                .declaration()
                .truth_view_basis()
                .digest(),
            read_packet.digest(),
            snapshot_identity.as_str(),
            authority_digest.as_ref(),
            equivalence_digest.as_ref(),
            record_value_evidence.canonical_basis(),
            equivalence_member_evidence.canonical_basis(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            fingerprint_identity: StructuralFingerprintIdentity::admit_bridge_owned(format!(
                "structural-fingerprint:sha256:{digest:x}"
            )),
            family: equivalence.fingerprint_family(),
            semantics_version: Arc::from(equivalence.semantics_version()),
            contract_identity: Arc::from(contract.contract_identity().as_str()),
            truth_view_basis_digest: Arc::from(
                contract
                    .validated_declaration()
                    .declaration()
                    .truth_view_basis()
                    .digest(),
            ),
            read_packet: read_packet.clone(),
            planned_packet_digest: Arc::from(read_packet.digest()),
            snapshot_identity,
            authority_digest,
            equivalence_digest,
            record_value_evidence,
            equivalence_member_evidence,
            record_aspect_value_digest,
            canonical_basis,
            digest: Arc::from(format!("structural-fingerprint:sha256:{digest:x}")),
        }
    }

    pub fn fingerprint_identity(&self) -> &StructuralFingerprintIdentity {
        &self.fingerprint_identity
    }

    pub fn family(&self) -> StructuralFingerprintFamily {
        self.family
    }

    pub fn semantics_version(&self) -> &str {
        self.semantics_version.as_ref()
    }

    pub fn contract_identity(&self) -> &str {
        self.contract_identity.as_ref()
    }

    pub fn truth_view_basis_digest(&self) -> &str {
        self.truth_view_basis_digest.as_ref()
    }

    pub fn planned_packet_digest(&self) -> &str {
        self.planned_packet_digest.as_ref()
    }

    pub fn read_packet(&self) -> &SnapshotReadPacket {
        &self.read_packet
    }

    pub fn snapshot_identity(&self) -> &TruthSnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn snapshot_identity_text(&self) -> &str {
        self.snapshot_identity.as_str()
    }

    pub fn authority_digest(&self) -> &str {
        self.authority_digest.as_ref()
    }

    pub fn equivalence_digest(&self) -> &str {
        self.equivalence_digest.as_ref()
    }

    pub fn record_value_evidence(&self) -> &StructuralFingerprintRecordValueEvidenceSet {
        &self.record_value_evidence
    }

    pub fn equivalence_member_evidence(&self) -> &StructuralFingerprintEquivalenceMemberSet {
        &self.equivalence_member_evidence
    }

    pub fn record_aspect_value_digest(&self) -> &str {
        self.record_aspect_value_digest.as_ref()
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[cfg(test)]
#[path = "fingerprints_tests.rs"]
mod tests;
