//! Current-source creation and a consumed exact admission are distinct inputs.
use super::super::{
    WorthQueryProductBranch, WorthQueryProductBranchAdmissionDenial,
    WorthQueryProductBranchAdmissionRefusalKind, WorthQueryProductBranchLease,
};
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime;

pub(super) enum ForkSource {
    Current(WorthQueryProductBranch),
    Expected(WorthQueryProductBranchLease),
}

impl ForkSource {
    pub(super) fn branch(&self) -> WorthQueryProductBranch {
        match self {
            Self::Current(branch) => *branch,
            Self::Expected(source) => source.product_branch(),
        }
    }

    pub(super) fn admit(
        self,
        runtime: &WorthQueryProductRuntime,
    ) -> Result<WorthQueryProductBranchLease, WorthQueryProductBranchAdmissionDenial> {
        match self {
            Self::Current(branch) => runtime.admit_product_occurrence(branch.occurrence()),
            Self::Expected(source) => {
                if source.observation().owner_identity() != runtime.owner.owner_identity() {
                    return Err(WorthQueryProductBranchAdmissionDenial::Refused(
                        WorthQueryProductBranchAdmissionRefusalKind::ForeignOwner,
                    ));
                }
                // World receives this exact retained observation and rechecks its
                // source axes. Re-admitting the occurrence here would select latest.
                Ok(source)
            }
        }
    }
}
