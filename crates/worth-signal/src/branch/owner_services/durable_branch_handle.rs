//! A live branch handle retains its real lease and proof-carried readmission phase.
use crate::branch::AdmittedSignalBranchBasis;
use crate::durability::SignalDurableBranchDescriptor;
use worth_proof::{AuthorityMarker, HomeIncarnationFence, SignalBranchReadmitted};

#[derive(Debug)]
struct SignalBranchAdmissionCustody {
    basis: AdmittedSignalBranchBasis,
    descriptor: SignalDurableBranchDescriptor,
    fence: HomeIncarnationFence,
}
impl AuthorityMarker for SignalBranchAdmissionCustody {}
/// Runtime custody: the actual branch lease has a lifetime and releases on Drop.
/// Its pure phase is carried by worth-proof, not by a caller assertion.
#[derive(Debug)]
pub struct SignalDurableBranchHandle {
    phase: SignalBranchReadmitted<SignalBranchAdmissionCustody>,
}
impl SignalDurableBranchHandle {
    pub(in crate::branch::owner_services) fn from_owner(
        basis: AdmittedSignalBranchBasis,
        descriptor: SignalDurableBranchDescriptor,
        fence: HomeIncarnationFence,
    ) -> Self {
        Self {
            phase: SignalBranchReadmitted::from_owner(SignalBranchAdmissionCustody {
                basis,
                descriptor,
                fence,
            }),
        }
    }
    pub fn descriptor(&self) -> &SignalDurableBranchDescriptor {
        &self.phase.admission().descriptor
    }
    pub fn basis(&self) -> &AdmittedSignalBranchBasis {
        &self.phase.admission().basis
    }
    pub(in crate::branch::owner_services) fn fence(&self) -> HomeIncarnationFence {
        self.phase.admission().fence
    }
}
