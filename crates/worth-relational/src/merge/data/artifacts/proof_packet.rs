use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;

use crate::history::data::RelationalMergeBranchBasis;
use crate::merge::data::NormalizedRelationalMergeRequest;
use crate::transactions::data::RecordRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalMergeProofPacketAdmissionPosture {
    ExecutionAdmitted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalMergeAdmittedSurfaceRow {
    record: RecordRef,
    target_record: Option<RecordRef>,
}

impl RelationalMergeAdmittedSurfaceRow {
    pub(crate) fn new(record: RecordRef, target_record: Option<RecordRef>) -> Self {
        Self {
            record,
            target_record,
        }
    }

    pub fn record(&self) -> &RecordRef {
        &self.record
    }

    pub fn target_record(&self) -> Option<&RecordRef> {
        self.target_record.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelationalMergeProofPacket {
    request: NormalizedRelationalMergeRequest,
    branch_basis: RelationalMergeBranchBasis,
    admitted_merge_surface: Arc<[RelationalMergeAdmittedSurfaceRow]>,
    correspondence_witness_digest: String,
    schema_reconciliation_witness_digest: String,
    strategy_witness_digest: String,
    foundational_request_lowering_digest: String,
    admitted_merge_surface_digest: String,
    planning_digest: String,
    execution_digest: String,
    admission_posture: RelationalMergeProofPacketAdmissionPosture,
    packet_digest: String,
}

impl RelationalMergeProofPacket {
    pub(crate) fn retained_execution_admitted(
        request: NormalizedRelationalMergeRequest,
        branch_basis: RelationalMergeBranchBasis,
        admitted_merge_surface: Arc<[RelationalMergeAdmittedSurfaceRow]>,
        correspondence_witness_digest: String,
        schema_reconciliation_witness_digest: String,
        strategy_witness_digest: String,
        foundational_request_lowering_digest: String,
        planning_digest: String,
        execution_digest: String,
    ) -> Self {
        let admission_posture = RelationalMergeProofPacketAdmissionPosture::ExecutionAdmitted;
        let admitted_merge_surface_digest = merge_admitted_surface_digest(&admitted_merge_surface);
        let packet_digest = merge_proof_packet_digest(
            &request,
            &branch_basis,
            &admitted_merge_surface,
            &correspondence_witness_digest,
            &schema_reconciliation_witness_digest,
            &strategy_witness_digest,
            &foundational_request_lowering_digest,
            &admitted_merge_surface_digest,
            &planning_digest,
            &execution_digest,
            admission_posture,
        );
        Self {
            request,
            branch_basis,
            admitted_merge_surface,
            correspondence_witness_digest,
            schema_reconciliation_witness_digest,
            strategy_witness_digest,
            foundational_request_lowering_digest,
            admitted_merge_surface_digest,
            planning_digest,
            execution_digest,
            admission_posture,
            packet_digest,
        }
    }

    pub fn request(&self) -> &NormalizedRelationalMergeRequest {
        &self.request
    }

    pub fn branch_basis(&self) -> &RelationalMergeBranchBasis {
        &self.branch_basis
    }

    pub fn admitted_merge_surface(&self) -> &[RelationalMergeAdmittedSurfaceRow] {
        &self.admitted_merge_surface
    }

    pub fn foundational_request_lowering_digest(&self) -> &str {
        &self.foundational_request_lowering_digest
    }

    pub fn correspondence_witness_digest(&self) -> &str {
        &self.correspondence_witness_digest
    }

    pub fn schema_reconciliation_witness_digest(&self) -> &str {
        &self.schema_reconciliation_witness_digest
    }

    pub fn strategy_witness_digest(&self) -> &str {
        &self.strategy_witness_digest
    }

    pub fn admitted_merge_surface_digest(&self) -> &str {
        &self.admitted_merge_surface_digest
    }

    pub fn planning_digest(&self) -> &str {
        &self.planning_digest
    }

    pub fn execution_digest(&self) -> &str {
        &self.execution_digest
    }

    pub fn admission_posture(&self) -> RelationalMergeProofPacketAdmissionPosture {
        self.admission_posture
    }

    pub fn packet_digest(&self) -> &str {
        &self.packet_digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct RelationalMergeProofPacketWire {
    request: NormalizedRelationalMergeRequest,
    branch_basis: RelationalMergeBranchBasis,
    admitted_merge_surface: Arc<[RelationalMergeAdmittedSurfaceRow]>,
    correspondence_witness_digest: String,
    schema_reconciliation_witness_digest: String,
    strategy_witness_digest: String,
    foundational_request_lowering_digest: String,
    admitted_merge_surface_digest: String,
    planning_digest: String,
    execution_digest: String,
    admission_posture: RelationalMergeProofPacketAdmissionPosture,
    packet_digest: String,
}

impl<'de> Deserialize<'de> for RelationalMergeProofPacket {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RelationalMergeProofPacketWire::deserialize(deserializer)?;
        let admitted_merge_surface_digest =
            merge_admitted_surface_digest(&wire.admitted_merge_surface);
        if admitted_merge_surface_digest != wire.admitted_merge_surface_digest {
            return Err(D::Error::custom(
                "merge proof packet admitted surface digest does not match retained surface truth",
            ));
        }
        if !digest_is_lowercase_sha256_hex(&wire.planning_digest) {
            return Err(D::Error::custom(
                "merge proof packet planning digest is not valid lowercase sha256 hex",
            ));
        }
        if !digest_is_lowercase_sha256_hex(&wire.correspondence_witness_digest) {
            return Err(D::Error::custom(
                "merge proof packet correspondence witness digest is not valid lowercase sha256 hex",
            ));
        }
        if !digest_is_lowercase_sha256_hex(&wire.schema_reconciliation_witness_digest) {
            return Err(D::Error::custom(
                "merge proof packet schema reconciliation witness digest is not valid lowercase sha256 hex",
            ));
        }
        if !digest_is_lowercase_sha256_hex(&wire.strategy_witness_digest) {
            return Err(D::Error::custom(
                "merge proof packet strategy witness digest is not valid lowercase sha256 hex",
            ));
        }
        if !digest_is_lowercase_sha256_hex(&wire.execution_digest) {
            return Err(D::Error::custom(
                "merge proof packet execution digest is not valid lowercase sha256 hex",
            ));
        }
        let packet_digest = merge_proof_packet_digest(
            &wire.request,
            &wire.branch_basis,
            &wire.admitted_merge_surface,
            &wire.correspondence_witness_digest,
            &wire.schema_reconciliation_witness_digest,
            &wire.strategy_witness_digest,
            &wire.foundational_request_lowering_digest,
            &wire.admitted_merge_surface_digest,
            &wire.planning_digest,
            &wire.execution_digest,
            wire.admission_posture,
        );
        if packet_digest != wire.packet_digest {
            return Err(D::Error::custom(
                "merge proof packet digest does not match retained proof truth",
            ));
        }
        Ok(Self {
            request: wire.request,
            branch_basis: wire.branch_basis,
            admitted_merge_surface: wire.admitted_merge_surface,
            correspondence_witness_digest: wire.correspondence_witness_digest,
            schema_reconciliation_witness_digest: wire.schema_reconciliation_witness_digest,
            strategy_witness_digest: wire.strategy_witness_digest,
            foundational_request_lowering_digest: wire.foundational_request_lowering_digest,
            admitted_merge_surface_digest: wire.admitted_merge_surface_digest,
            planning_digest: wire.planning_digest,
            execution_digest: wire.execution_digest,
            admission_posture: wire.admission_posture,
            packet_digest: wire.packet_digest,
        })
    }
}

fn merge_admitted_surface_digest(
    admitted_merge_surface: &[RelationalMergeAdmittedSurfaceRow],
) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"worth.relational.merge.admitted_surface.v1");
    for row in admitted_merge_surface {
        bytes.extend_from_slice(
            &rmp_serde::to_vec_named(row).expect(/* Proof: The serialized value contains only closed native scalar, sequence, map and derived-Serialize record fields; encoding uses an in-memory Vec/hash writer, with no user serializer or fallible I/O. */ "merge admitted surface row must encode"),
        );
    }
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn merge_proof_packet_digest(
    request: &NormalizedRelationalMergeRequest,
    branch_basis: &RelationalMergeBranchBasis,
    admitted_merge_surface: &[RelationalMergeAdmittedSurfaceRow],
    correspondence_witness_digest: &str,
    schema_reconciliation_witness_digest: &str,
    strategy_witness_digest: &str,
    foundational_request_lowering_digest: &str,
    admitted_merge_surface_digest: &str,
    planning_digest: &str,
    execution_digest: &str,
    admission_posture: RelationalMergeProofPacketAdmissionPosture,
) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"worth.relational.merge.proof_packet.v1");
    bytes.extend_from_slice(request.request_digest().as_bytes());
    bytes.extend_from_slice(branch_basis.basis_digest().as_bytes());
    bytes.extend_from_slice(admitted_merge_surface.len().to_string().as_bytes());
    bytes.extend_from_slice(correspondence_witness_digest.as_bytes());
    bytes.extend_from_slice(schema_reconciliation_witness_digest.as_bytes());
    bytes.extend_from_slice(strategy_witness_digest.as_bytes());
    bytes.extend_from_slice(foundational_request_lowering_digest.as_bytes());
    bytes.extend_from_slice(admitted_merge_surface_digest.as_bytes());
    bytes.extend_from_slice(planning_digest.as_bytes());
    bytes.extend_from_slice(execution_digest.as_bytes());
    bytes.push(match admission_posture {
        RelationalMergeProofPacketAdmissionPosture::ExecutionAdmitted => 1,
    });
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn digest_is_lowercase_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
