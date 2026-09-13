use super::RuntimeWorldOwnerIdentity;
use worth_relational::facade::branch::RelationalBranchBasisAdmissionIdentity;
use worth_runtime_bridge::facade::BridgeCorrespondenceAdmissionIdentity;
use worth_signal::facade::branch::SignalBranchBasisAdmissionIdentity;

/// Owner-issued identity of one exact admitted component/correspondence
/// tuple. It is not a descriptor digest and does not collapse commit history.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CompositeBasisKey {
    owner: RuntimeWorldOwnerIdentity,
    relational: RelationalBranchBasisAdmissionIdentity,
    signal: SignalBranchBasisAdmissionIdentity,
    correspondence: BridgeCorrespondenceAdmissionIdentity,
}

impl CompositeBasisKey {
    pub(super) fn issued(
        owner: RuntimeWorldOwnerIdentity,
        relational: RelationalBranchBasisAdmissionIdentity,
        signal: SignalBranchBasisAdmissionIdentity,
        correspondence: BridgeCorrespondenceAdmissionIdentity,
    ) -> Self {
        Self {
            owner,
            relational,
            signal,
            correspondence,
        }
    }

    pub const fn owner_identity(&self) -> RuntimeWorldOwnerIdentity {
        self.owner
    }
}
