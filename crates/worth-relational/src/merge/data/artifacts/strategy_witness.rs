use super::correspondence_witness::{lowercase_sha256, CorrespondenceHashWriter};
use super::MergeWitnessRows;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;

use super::strategy_witness_policy_rows::{
    execution_authority_contract_is_honest, RelationalMergeAspectPolicyWitnessRow,
};
use super::strategy_witness_posture_rows::{
    RelationalMergeDeletionStrategyWitnessRow, RelationalMergeTopologyStrategyWitnessRow,
};
use crate::merge::data::MergeExecutionAuthorityContract;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelationalMergeStrategyWitness {
    request_digest: String,
    branch_basis_digest: String,
    execution_authority_contract: MergeExecutionAuthorityContract,
    aspect_policy_rows: MergeWitnessRows<RelationalMergeAspectPolicyWitnessRow>,
    topology_rows: MergeWitnessRows<RelationalMergeTopologyStrategyWitnessRow>,
    deletion_rows: MergeWitnessRows<RelationalMergeDeletionStrategyWitnessRow>,
    witness_digest: String,
}

impl RelationalMergeStrategyWitness {
    pub(crate) fn retain_aspect_policy_rows_memory(
        &self,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Result<(), Option<worth_execution::ExecutionMemoryReservation>> {
        self.aspect_policy_rows.retain_decoded_memory(memory)
    }

    pub(crate) fn retain_topology_rows_memory(
        &self,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Result<(), Option<worth_execution::ExecutionMemoryReservation>> {
        self.topology_rows.retain_decoded_memory(memory)
    }

    pub(crate) fn retain_deletion_rows_memory(
        &self,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Result<(), Option<worth_execution::ExecutionMemoryReservation>> {
        self.deletion_rows.retain_decoded_memory(memory)
    }

    pub(crate) fn retained(
        request_digest: String,
        branch_basis_digest: String,
        execution_authority_contract: MergeExecutionAuthorityContract,
        aspect_policy_rows: Arc<[RelationalMergeAspectPolicyWitnessRow]>,
        topology_rows: Arc<[RelationalMergeTopologyStrategyWitnessRow]>,
        deletion_rows: Arc<[RelationalMergeDeletionStrategyWitnessRow]>,
    ) -> Self {
        let witness_digest = strategy_witness_digest(
            &request_digest,
            &branch_basis_digest,
            &execution_authority_contract,
            &aspect_policy_rows,
            &topology_rows,
            &deletion_rows,
        );
        Self {
            request_digest,
            branch_basis_digest,
            execution_authority_contract,
            aspect_policy_rows: aspect_policy_rows.into(),
            topology_rows: topology_rows.into(),
            deletion_rows: deletion_rows.into(),
            witness_digest,
        }
    }

    pub fn request_digest(&self) -> &str {
        &self.request_digest
    }
    pub fn branch_basis_digest(&self) -> &str {
        &self.branch_basis_digest
    }
    pub fn execution_authority_contract(&self) -> &MergeExecutionAuthorityContract {
        &self.execution_authority_contract
    }
    pub fn aspect_policy_rows(&self) -> &[RelationalMergeAspectPolicyWitnessRow] {
        &self.aspect_policy_rows
    }
    pub fn topology_rows(&self) -> &[RelationalMergeTopologyStrategyWitnessRow] {
        &self.topology_rows
    }
    pub fn deletion_rows(&self) -> &[RelationalMergeDeletionStrategyWitnessRow] {
        &self.deletion_rows
    }
    pub fn witness_digest(&self) -> &str {
        &self.witness_digest
    }

    pub(crate) fn retains_honest_truth(&self) -> bool {
        digest_is_lowercase_sha256_hex(&self.request_digest)
            && digest_is_lowercase_sha256_hex(&self.branch_basis_digest)
            && execution_authority_contract_is_honest(&self.execution_authority_contract)
            && self
                .aspect_policy_rows
                .iter()
                .all(RelationalMergeAspectPolicyWitnessRow::retains_honest_truth)
            && self
                .topology_rows
                .iter()
                .all(RelationalMergeTopologyStrategyWitnessRow::retains_honest_truth)
            && self
                .deletion_rows
                .iter()
                .all(RelationalMergeDeletionStrategyWitnessRow::retains_honest_truth)
            && self.witness_digest
                == strategy_witness_digest(
                    &self.request_digest,
                    &self.branch_basis_digest,
                    &self.execution_authority_contract,
                    &self.aspect_policy_rows,
                    &self.topology_rows,
                    &self.deletion_rows,
                )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct RelationalMergeStrategyWitnessWire {
    request_digest: String,
    branch_basis_digest: String,
    execution_authority_contract: MergeExecutionAuthorityContract,
    aspect_policy_rows: MergeWitnessRows<RelationalMergeAspectPolicyWitnessRow>,
    topology_rows: MergeWitnessRows<RelationalMergeTopologyStrategyWitnessRow>,
    deletion_rows: MergeWitnessRows<RelationalMergeDeletionStrategyWitnessRow>,
    witness_digest: String,
}

impl<'de> Deserialize<'de> for RelationalMergeStrategyWitness {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RelationalMergeStrategyWitnessWire::deserialize(deserializer)?;
        if !digest_is_lowercase_sha256_hex(&wire.request_digest) {
            return Err(D::Error::custom(
                "merge strategy witness request digest is not valid lowercase sha256 hex",
            ));
        }
        if !digest_is_lowercase_sha256_hex(&wire.branch_basis_digest) {
            return Err(D::Error::custom(
                "merge strategy witness branch basis digest is not valid lowercase sha256 hex",
            ));
        }
        if !execution_authority_contract_is_honest(&wire.execution_authority_contract) {
            return Err(D::Error::custom(
                "merge strategy witness execution authority contract does not match retained lowering contract",
            ));
        }
        let witness_digest = strategy_witness_digest(
            &wire.request_digest,
            &wire.branch_basis_digest,
            &wire.execution_authority_contract,
            &wire.aspect_policy_rows,
            &wire.topology_rows,
            &wire.deletion_rows,
        );
        if witness_digest != wire.witness_digest {
            return Err(D::Error::custom(
                "merge strategy witness digest does not match retained strategy truth",
            ));
        }
        let witness = Self {
            request_digest: wire.request_digest,
            branch_basis_digest: wire.branch_basis_digest,
            execution_authority_contract: wire.execution_authority_contract,
            aspect_policy_rows: wire.aspect_policy_rows,
            topology_rows: wire.topology_rows,
            deletion_rows: wire.deletion_rows,
            witness_digest: wire.witness_digest,
        };
        if !witness.retains_honest_truth() {
            return Err(D::Error::custom(
                "merge strategy witness retained truth is not internally honest",
            ));
        }
        Ok(witness)
    }
}

fn strategy_witness_digest(
    request_digest: &str,
    branch_basis_digest: &str,
    execution_authority_contract: &MergeExecutionAuthorityContract,
    aspect_policy_rows: &[RelationalMergeAspectPolicyWitnessRow],
    topology_rows: &[RelationalMergeTopologyStrategyWitnessRow],
    deletion_rows: &[RelationalMergeDeletionStrategyWitnessRow],
) -> String {
    let mut digest = Sha256::new();
    rmp_serde::encode::write_named(
        &mut CorrespondenceHashWriter(&mut digest),
        &(
            "worth.relational.merge.strategy_witness.v1",
            request_digest,
            branch_basis_digest,
            execution_authority_contract,
            aspect_policy_rows,
            topology_rows,
            deletion_rows,
        ),
    )
    .expect("closed native strategy witness must encode");
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
    use crate::identity::data::{EntityId, PartitionId};
    use crate::merge::data::{
        AspectMergePolicyKind, CustomMergePolicyIdentity, DeletionExecutionClass,
        MergeConflictClass, MergeExecutionReadiness, MergePolicyDecisionBoundary,
        MergePolicyOwnershipSurface, MergePolicyProofBoundary, ResolvedAspectMergePolicy,
        TopologyExecutionClass,
    };
    use crate::transactions::data::RecordRef;
    use worth_foundational::facade::AspectKey;

    #[test]
    fn streamed_strategy_witness_preserves_each_prior_named_tuple_digest() {
        let record = RecordRef::Entity(EntityId::new(PartitionId(1), 2, 1));
        let aspect = RelationalMergeAspectPolicyWitnessRow::retained(
            record.clone(),
            None,
            MergeConflictClass::SourceOnlyAddition,
            Arc::from([]),
            Arc::from([ResolvedAspectMergePolicy {
                aspect_key: AspectKey::new("counter").unwrap(),
                policy: AspectMergePolicyKind::Custom(CustomMergePolicyIdentity {
                    name: Arc::<str>::from("policy-λ").into(),
                    semantic_version: 2,
                }),
            }]),
            MergePolicyProofBoundary {
                ownership_surface: MergePolicyOwnershipSurface::ContainsCustomPolicy,
                decision_boundary: MergePolicyDecisionBoundary::AutoResolved,
            },
        );
        let old_aspect = rmp_serde::to_vec_named(&(
            "worth.relational.merge.strategy_witness.aspect_policy_row.v1",
            aspect.record(),
            aspect.target_record(),
            aspect.classification(),
            aspect.aspect_resolutions(),
            aspect.applied_policies(),
            aspect.proof_boundary(),
        ))
        .unwrap();
        assert_eq!(
            aspect.row_digest(),
            format!("{:x}", Sha256::digest(old_aspect))
        );
        let topology = RelationalMergeTopologyStrategyWitnessRow::retained(
            record.clone(),
            None,
            TopologyExecutionClass::RelationEndpointStable,
            MergeExecutionReadiness::Admitted,
            None,
        );
        let old_topology = rmp_serde::to_vec_named(&(
            "worth.relational.merge.strategy_witness.topology_row.v1",
            topology.record(),
            topology.target_record(),
            topology.topology_class(),
            topology.readiness(),
            topology.blocked_reason(),
        ))
        .unwrap();
        assert_eq!(
            topology.row_digest(),
            format!("{:x}", Sha256::digest(old_topology))
        );
        let deletion = RelationalMergeDeletionStrategyWitnessRow::retained(
            record,
            None,
            DeletionExecutionClass::DeletedOnBothSides,
            MergeExecutionReadiness::Admitted,
            None,
        );
        let old_deletion = rmp_serde::to_vec_named(&(
            "worth.relational.merge.strategy_witness.deletion_row.v1",
            deletion.record(),
            deletion.target_record(),
            deletion.deletion_class(),
            deletion.readiness(),
            deletion.blocked_reason(),
        ))
        .unwrap();
        assert_eq!(
            deletion.row_digest(),
            format!("{:x}", Sha256::digest(old_deletion))
        );
        let witness = RelationalMergeStrategyWitness::retained(
            "a".repeat(64),
            "b".repeat(64),
            crate::merge::lowered_artifact_execution_authority_contract(),
            Arc::from([aspect]),
            Arc::from([topology]),
            Arc::from([deletion]),
        );
        let old = rmp_serde::to_vec_named(&(
            "worth.relational.merge.strategy_witness.v1",
            witness.request_digest(),
            witness.branch_basis_digest(),
            witness.execution_authority_contract(),
            witness.aspect_policy_rows(),
            witness.topology_rows(),
            witness.deletion_rows(),
        ))
        .unwrap();
        assert_eq!(
            witness.witness_digest(),
            format!("{:x}", Sha256::digest(old))
        );
        let wire = rmp_serde::to_vec_named(&witness).unwrap();
        assert_eq!(
            rmp_serde::from_slice::<RelationalMergeStrategyWitness>(&wire).unwrap(),
            witness
        );
    }
}
