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
    ) -> Result<ProductBranchRetirementReport, RuntimeWorldBranchRetirementDenial> {
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
        let (cell, incarnation, retirement, retirement_boundary) = self
            .state
            .branches
            .retire(observed)
            .map_err(map_retirement_denial)?;
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
