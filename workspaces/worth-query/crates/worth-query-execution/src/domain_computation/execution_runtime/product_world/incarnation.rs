//! Selected live authorities must belong to this admitted Home incarnation.
use super::WorthQueryProductRuntime;
use crate::basis::{
    WorthQueryProductBranchAdmissionDenial as Denial,
    WorthQueryProductBranchAdmissionRefusalKind as Refusal,
};

impl WorthQueryProductRuntime {
    pub(in crate::domain_computation) fn admit_owner_incarnation(
        &self,
        presented: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    ) -> Result<(), Denial> {
        self.owner
            .owner_identity()
            .admit_incarnation(presented)
            .map_err(|cause| {
                Denial::Refused(match cause {
                    worth_proof::HomeIncarnationDenial::ForeignHome => Refusal::ForeignOwner,
                    worth_proof::HomeIncarnationDenial::StaleIncarnation => {
                        Refusal::StaleIncarnation
                    }
                })
            })
    }
}
