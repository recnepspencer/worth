use super::super::branching::{BranchManager, SignalOwnerPartitionDenial};
use crate::branch::owner_services::{
    SignalBranchBasisPort, SignalBranchLifecyclePort, SignalBranchMutationPort,
    SignalOwnerServiceIssuanceDenial, SignalOwnerServicePorts,
    DEFAULT_MAXIMUM_LIVE_SIGNAL_BRANCHES,
};

use super::SignalRuntime;

type SignalOwnerPortSlots<D, I, E, Ctx, T> = (
    SignalBranchBasisPort<D, I, T>,
    SignalBranchMutationPort<D, I, E, Ctx, T>,
    SignalBranchLifecyclePort<D, I, T>,
);

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    /// Issue a separate conditional capability for an exact sealed definition.
    /// The ordinary component bundle grants no access to this issuance route.
    pub fn issue_conditional_execution_service(
        &self,
        basis: &crate::branch::AdmittedSignalBranchBasis,
        claimant: &crate::data::aspect::SignalAspectLoweringOwner,
        source_authority: &worth_proof::ConditionalSourceObservationAuthority,
    ) -> Result<
        crate::branch::SignalConditionalExecutionPort<D, I, T>,
        crate::branch::SignalConditionalServiceIssuanceDenial,
    > {
        let owner = self
            .owner_services
            .downgrade_owner()
            .map_err(crate::branch::SignalConditionalServiceIssuanceDenial::OwnerUnavailable)?;
        crate::branch::SignalConditionalExecutionPort::issue(
            owner,
            basis,
            claimant,
            source_authority,
        )
    }

    /// Issue deterministic control over the real owner progression in test builds.
    #[cfg(feature = "test-operation-control")]
    pub fn owner_operation_control(
        &self,
    ) -> Result<
        crate::branch::owner_services::SignalOwnerOperationControl,
        crate::branch::owner_services::SignalOwnerUnavailable,
    > {
        self.owner_services.operation_control()
    }

    #[cfg(test)]
    pub(crate) fn inject_merge_participation_unwind_for_owner_contract(
        &mut self,
        source_branch_id: crate::state::SignalBranchId,
        target_branch_id: crate::state::SignalBranchId,
    ) {
        self.branches
            .mark_merge_participants(source_branch_id, target_branch_id);
        panic!("inject unwind after canonical merge participation begins");
    }

    pub(crate) fn owner_service_issuance_capability(
        &self,
    ) -> Result<(), SignalOwnerServiceIssuanceDenial> {
        if self.owner_services.is_sealed() {
            return Ok(());
        }
        if !self.event_bus.independent_branch_service_compatible() {
            return Err(SignalOwnerServiceIssuanceDenial::EventSubscriberStateConfigured);
        }
        if self.observations.registration_count() != 0 {
            return Err(SignalOwnerServiceIssuanceDenial::ObservationRegistrationStateConfigured);
        }
        let bound_queue_count = self.resource.bound_managed_queue_count();
        if bound_queue_count != 0 {
            return Err(
                SignalOwnerServiceIssuanceDenial::ManagedQueueStateConfigured { bound_queue_count },
            );
        }
        match self.branches.validate_owner_partition(
            self.graph.current_branch().id,
            DEFAULT_MAXIMUM_LIVE_SIGNAL_BRANCHES,
        ) {
            Ok(()) => Ok(()),
            Err(SignalOwnerPartitionDenial::LiveBranchCapacityExhausted {
                maximum_live_branches,
            }) => Err(
                SignalOwnerServiceIssuanceDenial::LiveBranchCapacityExhausted {
                    maximum_live_branches,
                },
            ),
            Err(SignalOwnerPartitionDenial::RetirementReceiptCapacityExhausted {
                maximum_retained_receipts,
            }) => Err(
                SignalOwnerServiceIssuanceDenial::RetirementReceiptCapacityExhausted {
                    maximum_retained_receipts,
                },
            ),
            Err(denial) => panic!("Signal branch owner partition invariant failed: {denial:?}"),
        }?;
        if !self.branches.conditional_retention_budget_matches(
            self.graph
                .installed_runtime_policy()
                .conditional_evaluation_budget(),
            self.graph
                .installed_runtime_policy()
                .conditional_temporal_budget(),
        ) {
            return Err(SignalOwnerServiceIssuanceDenial::ConditionalRetentionBudgetMismatch);
        }
        Ok(())
    }

    pub(crate) fn owner_port_slots(
        &mut self,
    ) -> Result<SignalOwnerPortSlots<D, I, E, Ctx, T>, SignalOwnerServiceIssuanceDenial>
    where
        D: Send + Sync + 'static,
        I: Send + Sync + 'static,
        E: Send + Sync + 'static,
        Ctx: Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        self.owner_service_issuance_capability()?;
        if !self.owner_services.is_sealed() {
            let conditional_budget = self
                .graph
                .installed_runtime_policy()
                .conditional_evaluation_budget();
            let runtime_instance_id = self.branches.owner_runtime_instance_id();
            let temporal_budget = self
                .graph
                .installed_runtime_policy()
                .conditional_temporal_budget();
            let active_state = self
                .take_heavy_active_branch_state()
                .expect("issuance preflight rejects non-transferable active state");
            let empty_legacy =
                BranchManager::with_live_catalog(Default::default(), runtime_instance_id);
            let branches = std::mem::replace(&mut self.branches, empty_legacy);
            let partition = branches.into_owner_partition(active_state);
            self.owner_services
                .seal(partition, conditional_budget, temporal_budget);
        }
        let weak_owner = self
            .owner_services
            .downgrade_owner()
            .expect("a sealed owner root always downgrades");
        let diagnostic_owner_runtime_instance_id = self.branches.owner_runtime_instance_id();
        Ok((
            SignalBranchBasisPort::new(weak_owner.clone(), diagnostic_owner_runtime_instance_id),
            SignalBranchMutationPort::new(weak_owner.clone(), diagnostic_owner_runtime_instance_id),
            SignalBranchLifecyclePort::new(weak_owner, diagnostic_owner_runtime_instance_id),
        ))
    }

    pub(crate) fn sealed_owner_port_slots(&self) -> Option<SignalOwnerPortSlots<D, I, E, Ctx, T>> {
        let weak_owner = self.owner_services.downgrade_owner().ok()?;
        let diagnostic_owner_runtime_instance_id = self.branches.owner_runtime_instance_id();
        Some((
            SignalBranchBasisPort::new(weak_owner.clone(), diagnostic_owner_runtime_instance_id),
            SignalBranchMutationPort::new(weak_owner.clone(), diagnostic_owner_runtime_instance_id),
            SignalBranchLifecyclePort::new(weak_owner, diagnostic_owner_runtime_instance_id),
        ))
    }

    /// Seal this runtime's canonical branch partition and issue weak component services.
    pub fn owner_component_services(
        &mut self,
    ) -> Result<SignalOwnerServicePorts<D, I, E, Ctx, T>, SignalOwnerServiceIssuanceDenial>
    where
        D: Send + Sync + 'static,
        I: Send + Sync + 'static,
        E: Send + Sync + 'static,
        Ctx: Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        let (basis, mutation, lifecycle) = self.owner_port_slots()?;
        Ok(SignalOwnerServicePorts::new(basis, mutation, lifecycle))
    }

    /// Claim the one non-cloneable definition-publication capability for the
    /// Runtime World that owns this sealed Signal component.
    pub fn runtime_world_definition_publication_port(
        &mut self,
    ) -> Result<
        crate::branch::SignalConditionalDefinitionPublicationPort<D, I, E, Ctx, T>,
        SignalOwnerServiceIssuanceDenial,
    >
    where
        D: Send + Sync + 'static,
        I: Send + Sync + 'static,
        E: Send + Sync + 'static,
        Ctx: Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        let (_, mutation, _) = self.owner_port_slots()?;
        if !self.owner_services.claim_definition_publication() {
            return Err(SignalOwnerServiceIssuanceDenial::DefinitionPublicationAlreadyIssued);
        }
        Ok(crate::branch::SignalConditionalDefinitionPublicationPort::new(mutation))
    }
}
