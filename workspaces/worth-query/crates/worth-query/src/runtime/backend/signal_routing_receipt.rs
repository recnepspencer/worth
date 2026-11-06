use std::collections::BTreeSet;

use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::memory_workspace::{
    WorthQueryCommitIdentity, WorthQueryMutationReceipt, WorthQuerySnapshotIdentity,
    WorthQueryWorkspaceError,
};
use crate::runtime::{WorthQueryMutationCausalityEvidence, WorthQueryMutationProvenanceEvidence};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalInvalidationRoutingReceipt {
    commit_identity: WorthQueryCommitIdentity,
    snapshot_identity: WorthQuerySnapshotIdentity,
    causality_evidence: WorthQueryMutationCausalityEvidence,
    provenance_evidence: WorthQueryMutationProvenanceEvidence,
    delta_count: usize,
    routed_collection_count: usize,
    receipt_identity: WorthQueryEvidenceIdentity,
}

impl SignalInvalidationRoutingReceipt {
    pub(crate) fn from_mutation_receipt(
        receipt: &WorthQueryMutationReceipt,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        let Some(authority) = receipt.bridge_authority.as_ref() else {
            return Err(WorthQueryWorkspaceError::new(
                "signal invalidation routing requires bridge-authored mutation authority",
            ));
        };
        if !receipt.has_current_mutation_authority() {
            return Err(WorthQueryWorkspaceError::new(
                "signal invalidation routing requires current runtime commit, snapshot, and mutation-target authority",
            ));
        }
        let routed_collection_count = receipt
            .deltas
            .iter()
            .map(|delta| delta.collection().to_string())
            .collect::<BTreeSet<_>>()
            .len();
        let delta_count = receipt.deltas.len();
        let commit_identity = receipt.commit_identity.clone();
        let snapshot_identity = receipt.snapshot_identity.clone();
        let commit_evidence_identity = commit_identity.evidence_identity();
        let snapshot_evidence_identity = snapshot_identity.evidence_identity();
        let causality_evidence = WorthQueryMutationCausalityEvidence::from_bridge(authority);
        let provenance_evidence = WorthQueryMutationProvenanceEvidence::from_bridge(authority);
        let receipt_identity = routing_identity_basis(
            &commit_evidence_identity,
            &snapshot_evidence_identity,
            &causality_evidence,
            &provenance_evidence,
        )
        .field_usize(WorthQueryEvidenceTag::new("delta_count"), delta_count)
        .field_usize(
            WorthQueryEvidenceTag::new("routed_collection_count"),
            routed_collection_count,
        )
        .seal()?;

        Ok(Self {
            commit_identity,
            snapshot_identity,
            causality_evidence,
            provenance_evidence,
            delta_count,
            routed_collection_count,
            receipt_identity,
        })
    }

    pub fn causality_digest(&self) -> &str {
        self.causality_evidence.causality_digest().as_str()
    }

    pub fn commit_identity(&self) -> &WorthQueryCommitIdentity {
        &self.commit_identity
    }

    pub fn snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn route_digest(&self) -> &str {
        self.causality_evidence.route_digest().as_str()
    }

    pub fn truth_view_digest(&self) -> &str {
        self.causality_evidence.truth_view_digest().as_str()
    }

    pub fn provenance_digest(&self) -> &str {
        self.provenance_evidence.contract_digest().as_str()
    }

    pub fn causality_evidence(&self) -> &WorthQueryMutationCausalityEvidence {
        &self.causality_evidence
    }

    pub fn provenance_evidence(&self) -> &WorthQueryMutationProvenanceEvidence {
        &self.provenance_evidence
    }

    pub fn delta_count(&self) -> usize {
        self.delta_count
    }

    pub fn routed_collection_count(&self) -> usize {
        self.routed_collection_count
    }

    pub fn receipt_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.receipt_identity
    }

    pub(crate) fn drift_from_mutation_receipt(
        &self,
        receipt: &WorthQueryMutationReceipt,
    ) -> Option<String> {
        let Some(authority) = receipt.bridge_authority.as_ref() else {
            return Some(
                "signal invalidation routing receipt cannot be checked without bridge authority"
                    .to_string(),
            );
        };
        let causality_evidence = WorthQueryMutationCausalityEvidence::from_bridge(authority);
        let provenance_evidence = WorthQueryMutationProvenanceEvidence::from_bridge(authority);
        (!self
            .commit_identity()
            .is_same_current_identity_as(&receipt.commit_identity)
            || !self
                .snapshot_identity()
                .is_same_current_identity_as(&receipt.snapshot_identity)
            || self.causality_evidence() != &causality_evidence
            || self.provenance_evidence() != &provenance_evidence)
            .then(|| "signal invalidation routing receipt drifted from write receipt".to_string())
    }
}

fn routing_identity_basis(
    commit: &WorthQueryEvidenceIdentity,
    snapshot: &WorthQueryEvidenceIdentity,
    causality: &WorthQueryMutationCausalityEvidence,
    provenance: &WorthQueryMutationProvenanceEvidence,
) -> crate::evidence_identity::WorthQueryEvidenceIdentityEncoder {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SignalInvalidationRoutingReceipt)
        .field_evidence_identity(WorthQueryEvidenceTag::new("commit_identity"), commit)
        .field_evidence_identity(WorthQueryEvidenceTag::new("snapshot_identity"), snapshot)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("causality"),
            causality.causality_digest().evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("route"),
            causality.route_digest().evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("truth_view"),
            causality.truth_view_digest().evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("provenance"),
            provenance.contract_digest().evidence_identity(),
        )
}
