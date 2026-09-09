use worth_signal::facade::{branch::SignalBranchBasisAdmissionIdentity, NodeId};

/// O(1)-axis lookup identity for one owner-admitted Signal basis and node.
/// The admission token is descriptive; the returned lowering remains the
/// live authority checked by Bridge before use.
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::conditional_execution) struct BridgeExactConditionalBasisKey {
    basis: SignalBranchBasisAdmissionIdentity,
    node: NodeId,
}

impl BridgeExactConditionalBasisKey {
    pub(super) fn new(basis: SignalBranchBasisAdmissionIdentity, node: NodeId) -> Self {
        Self { basis, node }
    }
}
