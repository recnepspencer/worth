use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;

use crate::merge::data::{
    AspectPolicyResolutionRecord, MergeConflictClass, MergeExecutionAuthorityContract,
    MergePolicyProofBoundary, ResolvedAspectMergePolicy,
};
use crate::merge::{
    aggregate_record_resolution, lowered_artifact_execution_authority_contract,
    ownership_surface_for_policies,
};
use crate::transactions::data::RecordRef;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelationalMergeAspectPolicyWitnessRow {
    record: RecordRef,
    target_record: Option<RecordRef>,
    classification: MergeConflictClass,
    aspect_resolutions: Arc<[AspectPolicyResolutionRecord]>,
    applied_policies: Arc<[ResolvedAspectMergePolicy]>,
    proof_boundary: MergePolicyProofBoundary,
    row_digest: String,
}

impl RelationalMergeAspectPolicyWitnessRow {
    pub(crate) fn retained(
        record: RecordRef,
        target_record: Option<RecordRef>,
        classification: MergeConflictClass,
        aspect_resolutions: Arc<[AspectPolicyResolutionRecord]>,
        applied_policies: Arc<[ResolvedAspectMergePolicy]>,
        proof_boundary: MergePolicyProofBoundary,
    ) -> Self {
        let row_digest = aspect_policy_row_digest(
            &record,
            target_record.as_ref(),
            classification,
            &aspect_resolutions,
            &applied_policies,
            proof_boundary,
        );
        Self {
            record,
            target_record,
            classification,
            aspect_resolutions,
            applied_policies,
            proof_boundary,
            row_digest,
        }
    }

    pub fn record(&self) -> &RecordRef {
        &self.record
    }
    pub fn target_record(&self) -> Option<&RecordRef> {
        self.target_record.as_ref()
    }
    pub fn classification(&self) -> MergeConflictClass {
        self.classification
    }
    pub fn aspect_resolutions(&self) -> &[AspectPolicyResolutionRecord] {
        &self.aspect_resolutions
    }
    pub fn applied_policies(&self) -> &[ResolvedAspectMergePolicy] {
        &self.applied_policies
    }
    pub fn proof_boundary(&self) -> MergePolicyProofBoundary {
        self.proof_boundary
    }
    pub fn row_digest(&self) -> &str {
        &self.row_digest
    }

    pub(crate) fn retains_honest_truth(&self) -> bool {
        aspect_policy_truth_matches(
            self.classification,
            &self.aspect_resolutions,
            &self.applied_policies,
            self.proof_boundary,
        ) && self.row_digest
            == aspect_policy_row_digest(
                &self.record,
                self.target_record.as_ref(),
                self.classification,
                &self.aspect_resolutions,
                &self.applied_policies,
                self.proof_boundary,
            )
    }
}

pub(crate) fn execution_authority_contract_is_honest(
    contract: &MergeExecutionAuthorityContract,
) -> bool {
    *contract == lowered_artifact_execution_authority_contract()
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct RelationalMergeAspectPolicyWitnessRowWire {
    record: RecordRef,
    target_record: Option<RecordRef>,
    classification: MergeConflictClass,
    aspect_resolutions: Arc<[AspectPolicyResolutionRecord]>,
    applied_policies: Arc<[ResolvedAspectMergePolicy]>,
    proof_boundary: MergePolicyProofBoundary,
    row_digest: String,
}

impl<'de> Deserialize<'de> for RelationalMergeAspectPolicyWitnessRow {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RelationalMergeAspectPolicyWitnessRowWire::deserialize(deserializer)?;
        if !aspect_policy_truth_matches(
            wire.classification,
            &wire.aspect_resolutions,
            &wire.applied_policies,
            wire.proof_boundary,
        ) {
            return Err(D::Error::custom(
                "merge strategy aspect policy row truth does not match retained policy inputs",
            ));
        }
        let row_digest = aspect_policy_row_digest(
            &wire.record,
            wire.target_record.as_ref(),
            wire.classification,
            &wire.aspect_resolutions,
            &wire.applied_policies,
            wire.proof_boundary,
        );
        if row_digest != wire.row_digest {
            return Err(D::Error::custom(
                "merge strategy aspect policy row digest does not match retained truth",
            ));
        }
        Ok(Self {
            record: wire.record,
            target_record: wire.target_record,
            classification: wire.classification,
            aspect_resolutions: wire.aspect_resolutions,
            applied_policies: wire.applied_policies,
            proof_boundary: wire.proof_boundary,
            row_digest: wire.row_digest,
        })
    }
}

fn aspect_policy_truth_matches(
    classification: MergeConflictClass,
    aspect_resolutions: &[AspectPolicyResolutionRecord],
    applied_policies: &[ResolvedAspectMergePolicy],
    proof_boundary: MergePolicyProofBoundary,
) -> bool {
    ownership_surface_for_policies(applied_policies) == proof_boundary.ownership_surface
        && aggregate_record_resolution(classification, aspect_resolutions)
            == proof_boundary.decision_boundary
}

fn aspect_policy_row_digest(
    record: &RecordRef,
    target_record: Option<&RecordRef>,
    classification: MergeConflictClass,
    aspect_resolutions: &[AspectPolicyResolutionRecord],
    applied_policies: &[ResolvedAspectMergePolicy],
    proof_boundary: MergePolicyProofBoundary,
) -> String {
    let digest = Sha256::digest(
        rmp_serde::to_vec_named(&(
            "worth.relational.merge.strategy_witness.aspect_policy_row.v1",
            record,
            target_record,
            classification,
            aspect_resolutions,
            applied_policies,
            proof_boundary,
        ))
        .expect(/* Proof: The serialized value contains only closed native scalar, sequence, map and derived-Serialize record fields; encoding uses an in-memory Vec/hash writer, with no user serializer or fallible I/O. */ "strategy witness policy row must encode"),
    );
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
