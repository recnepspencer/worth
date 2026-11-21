use super::{ExecutionRequest, QueryRequestExecution};

/// The source is always known; bootstrap and owner-only phases cannot lend a
/// Relational request by omitting a comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AdvancementSource {
    Relational(u64),
    ProductOwner(worth_runtime_world::facade::RuntimeWorldOwnerIdentity),
    Bootstrap,
}

/// Runtime custody of an active request, never retained in a product.
pub struct ActiveAdvancement<'scope> {
    pub(super) execution: &'scope QueryRequestExecution<'scope>,
}

/// The receiving installed runtime differs from the one that lent the phase.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct WorthQueryForeignAdvancementPhase(());

impl TryFrom<super::WorthQueryAdvancementDenial> for WorthQueryForeignAdvancementPhase {
    type Error = super::WorthQueryAdvancementDenial;

    fn try_from(denial: Self::Error) -> Result<Self, Self::Error> {
        match denial {
            super::WorthQueryAdvancementDenial::Resource(
                super::Resource::ForeignAdvancementPhase,
            ) => Ok(Self(())),
            denial => Err(denial),
        }
    }
}

impl From<WorthQueryForeignAdvancementPhase> for super::WorthQueryAdvancementDenial {
    fn from(_: WorthQueryForeignAdvancementPhase) -> Self {
        Self::Resource(super::Resource::ForeignAdvancementPhase)
    }
}

/// A phase borrows custody during one higher-ranked callback.
/// Completion consumes it; neither a caller nor a product can manufacture it.
///
/// A callback cannot return its phase.
///
/// A paused slot outside the callback cannot retain a phase.
///
/// A host-held product cannot contain borrowed execution custody.
///
/// Completion consumes custody, so another use of the phase is forbidden.
pub struct WorthQueryAdvancementPhase<'scope> {
    pub(super) active: &'scope ActiveAdvancement<'scope>,
    pub(super) source: AdvancementSource,
    pub(super) owner: Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity>,
}

impl<'scope> WorthQueryAdvancementPhase<'scope> {
    pub fn world_owner(&self) -> Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity> {
        self.owner
    }

    fn admitted_request(&self) -> ExecutionRequest<'scope, 'scope> {
        self.active
            .execution
            .execution_request()
            .expect("admitted advancement")
    }
    /// Borrows execution only after the receiving installed runtime accepts custody.
    pub fn execution_request_for(
        &self,
        runtime: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime,
    ) -> Result<ExecutionRequest<'scope, 'scope>, WorthQueryForeignAdvancementPhase> {
        self.request_for_owner(runtime.owner.owner_identity())
    }

    /// Lends the admitted request and preserves the shared advancement refusal.
    pub fn checked_execution_request_for(
        &self,
        runtime: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime,
    ) -> Result<ExecutionRequest<'scope, 'scope>, super::WorthQueryAdvancementDenial> {
        self.execution_request_for(runtime).map_err(Into::into)
    }

    /// Checks the receiving runtime and this phase's current interruption state.
    pub fn check_live_for(
        &self,
        runtime: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime,
    ) -> Result<(), super::WorthQueryAdvancementDenial> {
        self.checked_execution_request_for(runtime)?
            .check_live()
            .map_err(|stop| {
                super::advancement_denial(worth_execution::WorkCeilingDenial::Stopped(stop))
            })
    }

    pub fn request_for_owner(
        &self,
        owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    ) -> Result<ExecutionRequest<'scope, 'scope>, WorthQueryForeignAdvancementPhase> {
        if self.owner != Some(owner) {
            return Err(WorthQueryForeignAdvancementPhase(()));
        }
        Ok(self.admitted_request())
    }

    pub fn request_for_source(
        &self,
        source_instance: u64,
    ) -> Result<ExecutionRequest<'scope, 'scope>, WorthQueryForeignAdvancementPhase> {
        if self.source != AdvancementSource::Relational(source_instance) {
            return Err(WorthQueryForeignAdvancementPhase(()));
        }
        Ok(self.admitted_request())
    }

    pub fn request_for_managed(
        &self,
        observation: &crate::domain_computation::managed_run::WorthQueryManagedRelationalObservation,
    ) -> Result<ExecutionRequest<'scope, 'scope>, WorthQueryForeignAdvancementPhase> {
        match observation.owner_identity() {
            Some(owner) => self.request_for_owner(owner),
            None => self.request_for_source(observation.identity().runtime_instance_id()),
        }
    }

    pub fn complete(self) {}
    pub fn execution_for(
        &self,
        runtime: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime,
    ) -> Result<&QueryRequestExecution<'scope>, WorthQueryForeignAdvancementPhase> {
        self.request_for_owner(runtime.owner.owner_identity())?;
        Ok(self.active.execution)
    }
}

impl<'scope> WorthQueryAdvancementPhase<'scope> {
    pub fn test_host_request(&self) -> super::test_host_policy::TestHostRequest<'scope> {
        super::test_host_policy::TestHostRequest::from_installed(self)
    }

    /// Budget-only fixture controls borrow the existing host request.
    pub fn borrowed_request_for_test(&self) -> ExecutionRequest<'_, '_> {
        self.admitted_request()
    }
}

mod court_doors_for_test;
