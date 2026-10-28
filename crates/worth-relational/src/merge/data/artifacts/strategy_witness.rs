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
    aspect_policy_rows: Arc<[RelationalMergeAspectPolicyWitnessRow]>,
    topology_rows: Arc<[RelationalMergeTopologyStrategyWitnessRow]>,
    deletion_rows: Arc<[RelationalMergeDeletionStrategyWitnessRow]>,
    witness_digest: String,
}

impl RelationalMergeStrategyWitness {
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
            aspect_policy_rows,
            topology_rows,
            deletion_rows,
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
    aspect_policy_rows: Arc<[RelationalMergeAspectPolicyWitnessRow]>,
    topology_rows: Arc<[RelationalMergeTopologyStrategyWitnessRow]>,
    deletion_rows: Arc<[RelationalMergeDeletionStrategyWitnessRow]>,
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
    let digest = Sha256::digest(
        rmp_serde::to_vec_named(&(
            "worth.relational.merge.strategy_witness.v1",
            request_digest,
            branch_basis_digest,
            execution_authority_contract,
            aspect_policy_rows,
            topology_rows,
            deletion_rows,
        ))
        .expect(/* Proof: The serialized value contains only closed native scalar, sequence, map and derived-Serialize record fields; encoding uses an in-memory Vec/hash writer, with no user serializer or fallible I/O. */ "strategy witness must encode"),
    );
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn digest_is_lowercase_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
