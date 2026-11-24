use super::super::RuntimeWorldOwnerRoot;
use crate::branch::{
    ProductBranchObservation, ProductBranchRetirementReport, RuntimeWorldBranchRetirementDenial,
};

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Retirement releases the product reference this world owns and reports
    /// the component branches it created but must not delete itself.
    pub(super) fn retire_observed_branch(
        &self,
        observed: &ProductBranchObservation,
        execution: Option<&worth_execution::ExecutionRequest<'_, '_>>,
    ) -> Result<ProductBranchRetirementReport, RuntimeWorldBranchRetirementDenial> {
        if let Err(cause) = self
            .owner_identity()
            .admit_incarnation(observed.owner_identity())
        {
            return Err(match cause {
                worth_proof::HomeIncarnationDenial::ForeignHome => {
                    RuntimeWorldBranchRetirementDenial::OwnerUnavailable
                }
                worth_proof::HomeIncarnationDenial::StaleIncarnation => {
                    RuntimeWorldBranchRetirementDenial::StaleIncarnation
                }
            });
        }
        let branch = observed.branch_identity();
        if observed.owner_identity() != self.owner_identity() {
            return Err(RuntimeWorldBranchRetirementDenial::OwnerUnavailable);
        }
        if !self.branch_service_is_available() {
            return Err(RuntimeWorldBranchRetirementDenial::OwnerUnavailable);
        }
        let _operation = self
            .reserve_creation_operation()
            .map_err(|()| RuntimeWorldBranchRetirementDenial::OwnerUnavailable)?;
        let released_product_reference = branch.clone();
        let mut owner_retirement_work = self
            .state
            .custody
            .reserve_retirement_work(branch, observed.lifecycle_incarnation())
            .map_err(|()| RuntimeWorldBranchRetirementDenial::CapacityExhausted)?;
        let (cell, incarnation, retirement, retirement_boundary) =
            if let Some(binding) = self.state.durable.as_ref() {
                self.retire_durable_observed_branch(binding, observed, execution)?
            } else {
                self.state
                    .branches
                    .retire(observed)
                    .map_err(map_retirement_denial)?
            };
        let (retired_head, released_product_head) = retirement.into_parts();
        // Drop the product-head and history custody after the registry lock
        // has been released. No component lifecycle/delete port is called.
        drop(cell);
        drop(released_product_head);
        // The component branches this exact occurrence asked its owners to
        // create become typed work for those owners. Runtime World releases the
        // custody charge by naming the work, never by deleting a component
        // reference itself.
        self.state.custody.drain_retirement_work_into(
            branch,
            incarnation,
            &mut owner_retirement_work,
        );
        Ok(ProductBranchRetirementReport::new(
            released_product_reference,
            retired_head,
            retirement_boundary,
            owner_retirement_work,
        ))
    }

    fn retire_durable_observed_branch(
        &self,
        binding: &std::sync::Arc<crate::durability::WorldDurableMediumBinding>,
        observed: &ProductBranchObservation,
        caller_execution: Option<&worth_execution::ExecutionRequest<'_, '_>>,
    ) -> Result<
        (
            crate::branch::ProductBranchReferenceCell,
            crate::identity::ProductBranchIncarnation,
            crate::branch::ProductBranchReferenceRetirement,
            Option<crate::identity::CompositeCommitIdentity>,
        ),
        RuntimeWorldBranchRetirementDenial,
    > {
        use crate::branch::ProductBranchReferenceCellDenial as CellDenial;
        use crate::durability::{
            WorldArtifactInput, WorldDurableArtifact, WorldDurablePublicationStop as Stop,
            WorldHeadInput,
        };
        let cell = self
            .state
            .branches
            .branch_cell_by_lifecycle(observed.lifecycle_incarnation())
            .ok_or(RuntimeWorldBranchRetirementDenial::AlreadyRetired)?;
        let (permit, snapshot) = cell.begin_durable_retirement().map_err(|denial| match denial {
            CellDenial::Retired => RuntimeWorldBranchRetirementDenial::AlreadyRetired,
            _ => RuntimeWorldBranchRetirementDenial::OwnerUnavailable,
        })?;
        if snapshot.branch() != observed.branch_identity()
            || snapshot.lifecycle() != observed.lifecycle_incarnation()
        {
            permit.proven_no_effect();
            return Err(RuntimeWorldBranchRetirementDenial::AlreadyRetired);
        }
        let policy = self.state.execution.request_policy();
        let fallback_serial = worth_execution::SerialRequest::from_policy(
            &policy,
            worth_execution::CancellationSource::new().token(),
            None,
        );
        let execution = caller_execution
            .copied()
            .unwrap_or_else(|| worth_execution::ExecutionRequest::serial(&fallback_serial));
        let head = match WorldHeadInput::from_snapshot(&snapshot) {
            Ok(head) => head,
            Err(_) => {
                permit.proven_no_effect();
                return Err(RuntimeWorldBranchRetirementDenial::OwnerUnavailable);
            }
        };
        let subject = match WorldDurableArtifact::from_input(
            WorldArtifactInput::BranchRetired {
                head,
                retiring_incarnation: binding.binding().incarnation(),
            },
            &execution,
        ) {
            Ok(subject) => subject,
            Err(_) => {
                permit.proven_no_effect();
                return Err(RuntimeWorldBranchRetirementDenial::CapacityExhausted);
            }
        };
        loop {
            match binding.publish(&subject, &execution) {
                Ok(_committed) => break,
                Err(Stop::Pending) => std::thread::yield_now(),
                Err(Stop::Refused(_) | Stop::Busy | Stop::BlockedByPriorBarrier { .. }) => {
                    permit.proven_no_effect();
                    return Err(RuntimeWorldBranchRetirementDenial::OwnerUnavailable);
                }
                Err(Stop::Unresolved | Stop::EvidenceRequired) => {
                    // Dropping permit without proven_no_effect retains
                    // RetirementRecoveryRequired on the cell until reopen.
                    drop(permit);
                    return Err(RuntimeWorldBranchRetirementDenial::OwnerUnavailable);
                }
            }
        }
        self.state
            .branches
            .retire_permitted(observed, permit)
            .map_err(map_retirement_denial)
    }
}

fn map_retirement_denial(
    denial: crate::branch::registry::ProductBranchRegistryDenial,
) -> RuntimeWorldBranchRetirementDenial {
    use crate::branch::registry::ProductBranchRegistryDenial;

    match denial {
        ProductBranchRegistryDenial::AlreadyRetired => {
            RuntimeWorldBranchRetirementDenial::AlreadyRetired
        }
        ProductBranchRegistryDenial::CapacityExhausted => {
            RuntimeWorldBranchRetirementDenial::CapacityExhausted
        }
        ProductBranchRegistryDenial::ForeignOwner
        | ProductBranchRegistryDenial::AlreadyInstalled
        | ProductBranchRegistryDenial::ReservationMissing
        | ProductBranchRegistryDenial::IdentityMismatch
        | ProductBranchRegistryDenial::NameAlreadyReserved
        | ProductBranchRegistryDenial::NameAlreadyInstalled
        | ProductBranchRegistryDenial::BranchAlreadyInstalled
        | ProductBranchRegistryDenial::LifecycleAlreadyInstalled => {
            RuntimeWorldBranchRetirementDenial::OwnerUnavailable
        }
    }
}
