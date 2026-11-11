//! Actual receiver admission before workflow meaning or registry use.
use super::super::WorthQueryWorkflowRuntimeBindingDenial as Denial;
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use worth_runtime_world::facade::RuntimeWorldOwnerIdentity;

pub(super) fn admit(
    owner: RuntimeWorldOwnerIdentity,
    authority: WorthQueryRuntimeAuthorityIdentity,
    receiver_owner: RuntimeWorldOwnerIdentity,
    receiver_authority: WorthQueryRuntimeAuthorityIdentity,
) -> Result<(), Denial> {
    receiver_owner
        .admit_incarnation(owner)
        .map_err(|cause| match cause {
            worth_proof::HomeIncarnationDenial::ForeignHome => Denial::ForeignRuntime,
            worth_proof::HomeIncarnationDenial::StaleIncarnation => Denial::StaleIncarnation,
        })?;
    if authority != receiver_authority {
        return Err(Denial::ForeignRuntime);
    }
    Ok(())
}
