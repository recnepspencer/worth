use sha2::{Digest, Sha256};

use super::*;

pub struct ValidatedWorkflowOperationReceipt<'receipt> {
    pub receipt: &'receipt WorthQueryApplicationCommitReceipt,
    pub identity: [u8; 32],
}
impl ValidatedWorkflowOperationReceipt<'_> {
    pub fn outputs(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence {
        self.receipt.output_correspondence()
    }
    pub fn principal(&self) -> worth_relational::facade::identity::EntityId {
        let principal = self.receipt.principal_scope().principal();
        worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId::new(principal.partition_id()),
            principal.local_slot(),
            principal.generation(),
        )
    }
    pub fn identity(&self) -> [u8; 32] {
        self.identity
    }
}

pub fn operation_receipt_requires_recovery(receipt: &WorthQueryApplicationCommitReceipt) -> bool {
    receipt.dispatch_outbox().is_some()
        && !receipt
            .external_dispatch()
            .is_some_and(|dispatch| dispatch.is_external_completion())
}

pub fn receipt_identity_from_outcome(
    durable_owner: Option<worth_foundational::facade::DurableWorldOwnerIdentity>,
    runtime_authority: u64,
    outcome_identity: u64,
    operation_definition_identity: &[u8; 32],
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-query.workflow-operation-receipt.v2");
    match durable_owner {
        Some(durable) => {
            digest.update(b"home");
            digest.update(durable.home().bytes());
            digest.update(durable.slot().get().to_le_bytes());
        }
        None => {
            digest.update(b"memory");
            digest.update(runtime_authority.to_le_bytes());
        }
    }
    digest.update(outcome_identity.to_le_bytes());
    digest.update(operation_definition_identity);
    digest.finalize().into()
}
