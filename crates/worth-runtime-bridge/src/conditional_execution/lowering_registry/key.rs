use std::sync::Arc;
use worth_signal::facade::branch::SignalBranchIdentity;
use worth_signal::facade::{InstalledSignalConditionalContract, NodeId};

/// Registry identity for one installed conditional definition on one exact
/// Signal branch lineage.
///
/// The branch identity is descriptive rather than authority. Bridge derives it
/// only from an admitted Signal basis and still requires the exact live
/// lowering lease before an operation can proceed.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(in crate::conditional_execution) struct BridgeConditionalLoweringKey {
    branch: Arc<SignalBranchIdentity>,
    node: NodeId,
    definition_generation: u64,
}

impl BridgeConditionalLoweringKey {
    pub(in crate::conditional_execution) fn new(
        branch: SignalBranchIdentity,
        contract: &InstalledSignalConditionalContract,
    ) -> Self {
        Self {
            branch: Arc::new(branch),
            node: contract.node(),
            definition_generation: contract.generation(),
        }
    }

    pub(in crate::conditional_execution) fn successor(
        branch: SignalBranchIdentity,
        node: NodeId,
        definition_generation: u64,
    ) -> Self {
        Self {
            branch: Arc::new(branch),
            node,
            definition_generation,
        }
    }

    pub(in crate::conditional_execution) fn branch(&self) -> &SignalBranchIdentity {
        &self.branch
    }
}
