//! Exact descriptive lookup in the existing owner registry, without claim issuance.
use super::*;
use worth_foundational::facade::{RecordedProductBranchReference, WorldOwnerCoordinate};

impl WorthQueryRecoveryHandleRegistry {
    pub(crate) fn recorded_claim_state(
        &self,
        provider_runtime_instance_id: u64,
        original: &RecordedProductBranchReference,
    ) -> (bool, Option<WorthQueryRecoveryResourceTerminal>) {
        let (world_owner, commit_ordinal) = match original {
            RecordedProductBranchReference::Home(head) => (
                WorldOwnerCoordinate::Home {
                    owner: head.selected_commit().owner(),
                },
                head.selected_commit().ordinal(),
            ),
            RecordedProductBranchReference::Memory(head) => (
                WorldOwnerCoordinate::MemoryOnly {
                    observation: head.owner(),
                },
                head.selected_commit(),
            ),
        };
        self.claim_state(&WorthQueryRecoveryMintClaim {
            provider_runtime_instance_id,
            world_owner,
            commit_ordinal,
        })
    }
}
