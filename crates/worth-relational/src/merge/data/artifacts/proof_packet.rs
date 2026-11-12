use super::correspondence_witness::{lowercase_sha256, CorrespondenceHashWriter};
use super::MergeWitnessRows;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use std::io::Write;
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
    admitted_merge_surface: MergeWitnessRows<RelationalMergeAdmittedSurfaceRow>,
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
    pub(crate) fn retain_admitted_merge_surface_memory(
        &self,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Result<(), Option<worth_execution::ExecutionMemoryReservation>> {
        self.admitted_merge_surface.retain_decoded_memory(memory)
    }

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
            admitted_merge_surface: admitted_merge_surface.into(),
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
    admitted_merge_surface: MergeWitnessRows<RelationalMergeAdmittedSurfaceRow>,
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
    let mut digest = Sha256::new();
    digest.update(b"worth.relational.merge.admitted_surface.v1");
    for row in admitted_merge_surface {
        rmp_serde::encode::write_named(&mut CorrespondenceHashWriter(&mut digest), row)
            .expect("closed native merge admitted surface row must encode");
    }
    lowercase_sha256(digest.finalize().into())
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
    let mut digest = Sha256::new();
    digest.update(b"worth.relational.merge.proof_packet.v1");
    digest.update(request.request_digest().as_bytes());
    digest.update(branch_basis.basis_digest().as_bytes());
    write!(
        &mut CorrespondenceHashWriter(&mut digest),
        "{}",
        admitted_merge_surface.len()
    )
    .expect("hash writing cannot fail");
    digest.update(correspondence_witness_digest.as_bytes());
    digest.update(schema_reconciliation_witness_digest.as_bytes());
    digest.update(strategy_witness_digest.as_bytes());
    digest.update(foundational_request_lowering_digest.as_bytes());
    digest.update(admitted_merge_surface_digest.as_bytes());
    digest.update(planning_digest.as_bytes());
    digest.update(execution_digest.as_bytes());
    digest.update([match admission_posture {
        RelationalMergeProofPacketAdmissionPosture::ExecutionAdmitted => 1,
    }]);
    lowercase_sha256(digest.finalize().into())
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
    use crate::history::data::{
        BranchId, CommitId, MergeBaseSelectionRule, RelationalCommitReceipt, ResolvedMergeBase,
    };
    use crate::identity::data::{EntityId, PartitionId, VersionId};
    use crate::merge::data::{
        MergeIntent, RelationalMergeCorrespondencePosture,
        RelationalMergeSchemaReconciliationPosture, RelationalMergeTopologyIntent,
    };

    #[test]
    fn streamed_proof_packet_preserves_prior_concatenated_bytes_and_surface_codec() {
        let request = NormalizedRelationalMergeRequest::admit_full_branch(
            BranchId("target".into()),
            BranchId("source-λ".into()),
            MergeIntent::ReconcileIntoTarget,
            RelationalMergeCorrespondencePosture::Advisory,
            RelationalMergeSchemaReconciliationPosture::Participate,
            RelationalMergeTopologyIntent::PreserveTopologySemantics,
        )
        .unwrap();
        let receipt = RelationalCommitReceipt {
            commit_id: CommitId(3),
            version_id: VersionId(4),
            branch_id: BranchId("source-λ".into()),
            parents: vec![CommitId(1)],
        };
        let basis = RelationalMergeBranchBasis {
            source_head: receipt.clone(),
            target_head: receipt.clone(),
            merge_base: ResolvedMergeBase {
                rule: MergeBaseSelectionRule::MaxCommitIdCommonAncestor,
                commit: receipt,
                supporting_left_ancestors: Arc::<[CommitId]>::from([CommitId(1)]).into(),
                supporting_right_ancestors: Arc::<[CommitId]>::from([]).into(),
            },
        };
        let surface = Arc::from([RelationalMergeAdmittedSurfaceRow::new(
            RecordRef::Entity(EntityId::new(PartitionId(1), 2, 1)),
            None,
        )]);
        let packet = RelationalMergeProofPacket::retained_execution_admitted(
            request,
            basis,
            surface,
            "a".repeat(64),
            "b".repeat(64),
            "c".repeat(64),
            "d".repeat(64),
            "e".repeat(64),
            "f".repeat(64),
        );
        let mut prior_surface = b"worth.relational.merge.admitted_surface.v1".to_vec();
        for row in packet.admitted_merge_surface() {
            prior_surface.extend(rmp_serde::to_vec_named(row).unwrap());
        }
        assert_eq!(
            packet.admitted_merge_surface_digest(),
            format!("{:x}", Sha256::digest(prior_surface))
        );
        let mut prior = b"worth.relational.merge.proof_packet.v1".to_vec();
        prior.extend(packet.request().request_digest().as_bytes());
        prior.extend(packet.branch_basis().basis_digest().as_bytes());
        prior.extend(packet.admitted_merge_surface().len().to_string().as_bytes());
        for text in [
            packet.correspondence_witness_digest(),
            packet.schema_reconciliation_witness_digest(),
            packet.strategy_witness_digest(),
            packet.foundational_request_lowering_digest(),
            packet.admitted_merge_surface_digest(),
            packet.planning_digest(),
            packet.execution_digest(),
        ] {
            prior.extend(text.as_bytes());
        }
        prior.push(1);
        assert_eq!(
            packet.packet_digest(),
            format!("{:x}", Sha256::digest(prior))
        );
        let wire = rmp_serde::to_vec_named(&packet).unwrap();
        assert_eq!(
            rmp_serde::from_slice::<RelationalMergeProofPacket>(&wire).unwrap(),
            packet
        );
    }
}
