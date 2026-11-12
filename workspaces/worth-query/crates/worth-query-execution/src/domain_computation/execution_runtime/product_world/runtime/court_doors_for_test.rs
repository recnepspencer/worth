//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProductRuntime {
    #[doc(hidden)]
    pub fn default_branch_for_test(&self) -> &ProductBranchIdentity {
        Self::default_branch(self)
    }
}

impl WorthQueryProductRuntime {
    #[doc(hidden)]
    pub fn inspection_port_for_test(
        &self,
    ) -> worth_runtime_world::facade::RuntimeWorldInspectionPort {
        self.owner.inspection_port()
    }
}

impl WorthQueryProductRuntime {
    #[doc(hidden)]
    pub fn owner_cleanup_for_test(&self) -> &WorthQueryProductBranchOwnerCleanupRegistry {
        &self.owner_cleanup
    }
}

impl WorthQueryProductRuntime {
    #[doc(hidden)]
    pub fn source_for_test(&self) -> &RuntimeBridgeRelationalSource {
        self.source()
    }
}

impl WorthQueryProductRuntime {
    #[doc(hidden)]
    pub fn owner_cleanup_mut_for_test(
        &mut self,
    ) -> &mut WorthQueryProductBranchOwnerCleanupRegistry {
        &mut self.owner_cleanup
    }
}

impl WorthQueryProductRuntime {
    #[doc(hidden)]
    pub fn clock_for_test(&self) -> &WorthQueryProductWorldClock {
        &self.clock
    }
}

impl WorthQueryProductRuntime {
    #[doc(hidden)]
    pub fn with_execution_placement_for_test<R>(
        &self,
        body: impl for<'owner> FnOnce(
            worth_runtime_world::facade::RuntimeWorldExecutionPlacement<'owner>,
        ) -> R,
    ) -> Result<R, crate::domain_computation::primary_graph::WorthQueryAdvancementDenial> {
        self.with_execution_placement(body)
    }
}
