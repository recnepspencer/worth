//! Rebuildable descriptive support, never a mutation or source-observation owner.
use super::{inventory_identity, WorkflowCapturedInputContract, WorkflowCapturedTarget};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};

#[derive(Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorkflowCapturedInputSupport {
    proposal_identity: String,
    inventory_identity: [u8; 32],
    targets: Box<[WorkflowCapturedTarget]>,
}
impl WorkflowCapturedInputSupport {
    pub(in crate::domain_computation::primary_graph) fn validate(
        proposal_identity: String,
        identity: [u8; 32],
        targets: Vec<WorkflowCapturedTarget>,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial> {
        if targets.windows(2).any(|pair| pair[0].node >= pair[1].node)
            || inventory_identity(&targets) != Some(identity)
        {
            return Err(WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                "captured input support differs from its committed inventory",
            ));
        }
        Ok(Self {
            proposal_identity,
            inventory_identity: identity,
            targets: targets.into_boxed_slice(),
        })
    }
    pub(in crate::domain_computation::primary_graph) fn target(
        &self,
        proposal_identity: &str,
        identity: [u8; 32],
        count: u64,
        node: &str,
    ) -> Option<&WorkflowCapturedInputContract> {
        if self.proposal_identity != proposal_identity
            || self.inventory_identity != identity
            || usize::try_from(count).ok() != Some(self.targets.len())
        {
            return None;
        }
        self.targets
            .binary_search_by(|target| target.node.as_str().cmp(node))
            .ok()
            .map(|index| &self.targets[index].contract)
    }
    pub(in crate::domain_computation::primary_graph) fn retained_charge_bytes(&self) -> usize {
        // Count the complete shared backing at every retained entry, as existing
        // progress accounting does: sharing never discounts admitted capacity.
        self.targets.iter().fold(
            std::mem::size_of::<Self>()
                .saturating_add(2 * std::mem::size_of::<usize>())
                .saturating_add(self.proposal_identity.capacity())
                .saturating_add(std::mem::size_of_val(self.targets.as_ref())),
            |bytes, target| {
                bytes
                    .saturating_add(target.node.capacity())
                    .saturating_add(target.contract.binding.capacity())
                    .saturating_add(target.contract.input_type.capacity())
            },
        )
    }
}
