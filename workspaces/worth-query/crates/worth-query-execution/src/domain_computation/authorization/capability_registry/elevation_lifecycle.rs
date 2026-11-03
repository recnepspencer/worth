use std::collections::{BTreeMap, BTreeSet};

use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityElevationRule, ApplicationCapabilityTransitionBinding,
    ErasedApplicationCapabilityContract,
};
use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;

use super::WorthQueryInstalledCapabilityPlan;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum WorthQueryElevationLifecycleOperationRole {
    Request,
    Approve,
    Revoke,
    CompleteReview,
}

pub(super) struct WorthQueryInstalledElevationLifecycleOperation {
    governed_capability_identity: [u8; 32],
    command_capability_identity: [u8; 32],
    role: WorthQueryElevationLifecycleOperationRole,
}

#[derive(Default)]
pub(super) struct WorthQueryInstalledElevationLifecycleRegistry {
    operations: BTreeMap<
        String,
        BTreeMap<String, BTreeMap<String, WorthQueryInstalledElevationLifecycleOperation>>,
    >,
    executable_operations: BTreeSet<(String, String)>,
}

impl WorthQueryInstalledElevationLifecycleRegistry {
    pub(super) fn install(
        &mut self,
        governed_capability_identity: [u8; 32],
        plans: &BTreeMap<[u8; 32], WorthQueryInstalledCapabilityPlan>,
        contract: &ErasedApplicationCapabilityContract,
    ) -> Result<(), ()> {
        let ApplicationCapabilityElevationRule::Governed(elevation) = contract.elevation() else {
            return Ok(());
        };
        let lifecycle = elevation.lifecycle();
        for (role, transition) in [
            (
                WorthQueryElevationLifecycleOperationRole::Request,
                lifecycle.request(),
            ),
            (
                WorthQueryElevationLifecycleOperationRole::Approve,
                lifecycle.approve(),
            ),
            (
                WorthQueryElevationLifecycleOperationRole::Revoke,
                lifecycle.revoke(),
            ),
            (
                WorthQueryElevationLifecycleOperationRole::CompleteReview,
                lifecycle.complete_review(),
            ),
        ] {
            let operation = transition.operation();
            let command_capability_identity = command_capability(plans, transition).ok_or(())?;
            if !self.executable_operations.insert((
                operation.operation().to_owned(),
                operation.input_type().to_owned(),
            )) {
                return Err(());
            }
            if self
                .operations
                .entry(operation.operation().to_owned())
                .or_default()
                .entry(operation.input_type().to_owned())
                .or_default()
                .insert(
                    operation.operation_type().to_owned(),
                    WorthQueryInstalledElevationLifecycleOperation {
                        governed_capability_identity,
                        command_capability_identity,
                        role,
                    },
                )
                .is_some()
            {
                return Err(());
            }
        }
        Ok(())
    }

    pub(super) fn operation<Schema, Operation>(
        &self,
        operation: &str,
        input_type: &str,
    ) -> Result<
        Option<(
            [u8; 32],
            [u8; 32],
            WorthQueryElevationLifecycleOperationRole,
        )>,
        (),
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
    {
        let Some(inputs) = self.operations.get(operation) else {
            return Ok(None);
        };
        let Some(markers) = inputs.get(input_type) else {
            return Ok(None);
        };
        let installed = markers.get(Operation::IDENTIFIER).ok_or(())?;
        Ok(Some((
            installed.governed_capability_identity,
            installed.command_capability_identity,
            installed.role,
        )))
    }

    pub(super) fn len(&self) -> usize {
        self.executable_operations.len()
    }
}

fn command_capability(
    plans: &BTreeMap<[u8; 32], WorthQueryInstalledCapabilityPlan>,
    binding: &ApplicationCapabilityTransitionBinding,
) -> Option<[u8; 32]> {
    plans.iter().find_map(|(identity, plan)| {
        let contract = &plan.contract();
        (contract.name() == binding.capability()
            && contract.capability_type() == binding.capability_type()
            && contract.operation() == binding.operation().operation()
            && contract.operation_type() == binding.operation().operation_type()
            && contract.input_type() == binding.operation().input_type()
            && contract.elevation().definition().is_none())
        .then_some(*identity)
    })
}
