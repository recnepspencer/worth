use worth_runtime_world::facade::{RuntimeWorldBranchAdmissionDenial, RuntimeWorldServiceDenial};

use super::WorthQueryProductRuntime;
use crate::basis::{WorthQueryProductBranchAdmissionDenial, WorthQueryProductBranchLease};

impl WorthQueryProductRuntime {
    pub fn admit_product_branch(
        &self,
        identity: &worth_runtime_world::facade::ProductBranchIdentity,
    ) -> Result<WorthQueryProductBranchLease, WorthQueryProductBranchAdmissionDenial> {
        self.with_product_observation(identity, |observation| {
            let bridge_source = self
                .source
                .retain_branch_basis_for_bridge(observation.basis().relational_basis())
                .map_err(|_| WorthQueryProductBranchAdmissionDenial::BridgeSourceUnavailable)?;
            Ok(WorthQueryProductBranchLease::new(
                super::WorthQueryProductPublicationBinding::new(
                    observation,
                    self.owner.publication_port(),
                    self.owner.recovery_port(),
                    self.clock.clone(),
                    self.root_identity(),
                ),
                bridge_source,
            ))
        })
    }

    /// Resolve through World once while the selected executable package is stable.
    /// The caller composes only the custody needed by its own entry purpose.
    pub(crate) fn with_product_observation<Output>(
        &self,
        identity: &worth_runtime_world::facade::ProductBranchIdentity,
        admit: impl FnOnce(
            worth_runtime_world::facade::ProductBranchObservation,
        ) -> Result<Output, WorthQueryProductBranchAdmissionDenial>,
    ) -> Result<Output, WorthQueryProductBranchAdmissionDenial> {
        if identity.owner_identity() != self.owner.owner_identity() {
            return Err(WorthQueryProductBranchAdmissionDenial::ForeignOwner);
        }
        let gate = match self.activations.gate(identity) {
            Ok(gate) => gate,
            Err(super::activation::WorthQueryProductActivationDenial::UnknownProductBranch) => {
                // Only World decides whether an uncoordinated name is unknown,
                // retired, or temporarily awaiting executable-package activation.
                self.owner
                    .observation_port()
                    .observe_product_branch(identity)
                    .map_err(map_world_denial)?;
                return Err(WorthQueryProductBranchAdmissionDenial::ProductActivationUnavailable);
            }
            Err(denial) => return Err(denial.into()),
        };
        gate.with_admission(|| {
            let observation = self
                .owner
                .observation_port()
                .observe_product_branch(identity)
                .map_err(map_world_denial)?;
            if observation.owner_identity() != self.owner.owner_identity() {
                return Err(WorthQueryProductBranchAdmissionDenial::ForeignOwner);
            }
            admit(observation)
        })
    }
}

fn map_world_denial(
    denial: RuntimeWorldServiceDenial<RuntimeWorldBranchAdmissionDenial>,
) -> WorthQueryProductBranchAdmissionDenial {
    match denial {
        RuntimeWorldServiceDenial::OwnerUnavailable(_) => {
            WorthQueryProductBranchAdmissionDenial::OwnerUnavailable
        }
        RuntimeWorldServiceDenial::Denied(RuntimeWorldBranchAdmissionDenial::ForeignOwner) => {
            WorthQueryProductBranchAdmissionDenial::ForeignOwner
        }
        RuntimeWorldServiceDenial::Denied(RuntimeWorldBranchAdmissionDenial::RetiredBranch) => {
            WorthQueryProductBranchAdmissionDenial::RetiredBranch
        }
        RuntimeWorldServiceDenial::Denied(_) => {
            WorthQueryProductBranchAdmissionDenial::ObservationRejected
        }
    }
}
