use super::{ExecutionRequest, QueryRequestExecution};

/// Runtime custody of an active request, never retained in a product.
pub(in crate::domain_computation::primary_graph) struct ActiveAdvancement<'scope> {
    pub(super) source_instance: Option<u64>,
    pub(super) owner: Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity>,
    pub(super) execution: &'scope QueryRequestExecution<'scope>,
}

/// The receiving installed runtime differs from the one that lent the phase.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct WorthQueryForeignAdvancementPhase(());

impl From<WorthQueryForeignAdvancementPhase> for super::WorthQueryAdvancementDenial {
    fn from(_: WorthQueryForeignAdvancementPhase) -> Self {
        Self::ForeignPhase
    }
}

/// A phase borrows custody during one higher-ranked callback.
/// Completion consumes it; neither a caller nor a product can manufacture it.
///
/// A callback cannot return its phase.
/// ```compile_fail
/// use worth_query_execution::facade::product::WorthQueryPrimaryGraphApplicationRuntime;
/// use worth_query_installation::facade::ApplicationSchema;
/// use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
/// fn demonstrate<Schema: ApplicationSchema + 'static>(
///     runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
///     request: &WorthQueryRequestScope,
/// ) {
///     let _ = runtime.with_application_advancement(request, |phase| phase);
/// }
/// ```
/// ```
/// use worth_query_execution::facade::product::WorthQueryPrimaryGraphApplicationRuntime;
/// use worth_query_installation::facade::ApplicationSchema;
/// use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
/// fn demonstrate<Schema: ApplicationSchema + 'static>(
///     runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
///     request: &WorthQueryRequestScope,
/// ) {
///     let _ = runtime.with_application_advancement(request, |phase| phase.complete());
/// }
/// ```
///
/// A paused slot outside the callback cannot retain a phase.
/// ```compile_fail
/// use worth_query_execution::facade::product::WorthQueryPrimaryGraphApplicationRuntime;
/// use worth_query_installation::facade::ApplicationSchema;
/// use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
/// fn demonstrate<Schema: ApplicationSchema + 'static>(
///     runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
///     request: &WorthQueryRequestScope,
/// ) {
///     let mut paused = None;
///     let _ = runtime.with_application_advancement(request, |phase| {
///         paused = Some(phase);
///     });
/// }
/// ```
/// ```
/// use worth_query_execution::facade::product::WorthQueryPrimaryGraphApplicationRuntime;
/// use worth_query_installation::facade::ApplicationSchema;
/// use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
/// fn demonstrate<Schema: ApplicationSchema + 'static>(
///     runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
///     request: &WorthQueryRequestScope,
/// ) {
///     let mut paused = None;
///     let _ = runtime.with_application_advancement(request, |phase| {
///         paused = Some(phase.complete());
///     });
/// }
/// ```
///
/// A host-held product cannot contain borrowed execution custody.
/// ```compile_fail
/// use worth_query_execution::facade::product::WorthQueryPrimaryGraphApplicationRuntime;
/// use worth_query_installation::facade::ApplicationSchema;
/// use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
/// struct HostProduct<T: 'static> { evidence: T }
/// fn hand_to_host<T: 'static>(_: HostProduct<T>) {}
/// fn demonstrate<Schema: ApplicationSchema + 'static>(
///     runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
///     request: &WorthQueryRequestScope,
/// ) {
///     let _ = runtime.with_application_advancement(request, |phase| {
///         hand_to_host(HostProduct { evidence: phase });
///     });
/// }
/// ```
/// ```
/// use worth_query_execution::facade::product::WorthQueryPrimaryGraphApplicationRuntime;
/// use worth_query_installation::facade::ApplicationSchema;
/// use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
/// struct HostProduct<T: 'static> { evidence: T }
/// fn hand_to_host<T: 'static>(_: HostProduct<T>) {}
/// fn demonstrate<Schema: ApplicationSchema + 'static>(
///     runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
///     request: &WorthQueryRequestScope,
/// ) {
///     let _ = runtime.with_application_advancement(request, |phase| {
///         hand_to_host(HostProduct { evidence: phase.complete() });
///     });
/// }
/// ```
///
/// Completion consumes custody, so another use of the phase is forbidden.
/// ```compile_fail,E0382
/// use worth_query_execution::facade::product::WorthQueryPrimaryGraphApplicationRuntime;
/// use worth_query_installation::facade::ApplicationSchema;
/// use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
/// fn complete<Schema: ApplicationSchema + 'static>(
///     runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
///     request: &WorthQueryRequestScope,
/// ) {
///     let _ = runtime.with_application_advancement(request, |phase| {
///         phase.complete();
///         let _ = phase.complete();
///     });
/// }
/// ```
/// ```
/// use worth_query_execution::facade::product::WorthQueryPrimaryGraphApplicationRuntime;
/// use worth_query_installation::facade::ApplicationSchema;
/// use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
/// fn complete<Schema: ApplicationSchema + 'static>(
///     runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
///     request: &WorthQueryRequestScope,
/// ) {
///     let _ = runtime.with_application_advancement(request, |phase| {
///         phase.complete();
///         let _ = ();
///     });
/// }
/// ```
#[doc = include_str!("phase_request_contracts.md")]
pub struct WorthQueryAdvancementPhase<'scope> {
    pub(super) active: &'scope ActiveAdvancement<'scope>,
}

impl<'scope> WorthQueryAdvancementPhase<'scope> {
    fn admitted_request(&self) -> ExecutionRequest<'_, '_> {
        self.active
            .execution
            .execution_request()
            .expect("admitted advancement")
    }
    /// Borrows execution only after the receiving installed runtime accepts custody.
    pub fn execution_request_for(
        &self,
        runtime: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime,
    ) -> Result<ExecutionRequest<'_, '_>, WorthQueryForeignAdvancementPhase> {
        self.request_for_owner(runtime.owner.owner_identity())
    }

    pub(crate) fn request_for_owner(
        &self,
        owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    ) -> Result<ExecutionRequest<'_, '_>, WorthQueryForeignAdvancementPhase> {
        if self.active.owner != Some(owner) {
            return Err(WorthQueryForeignAdvancementPhase(()));
        }
        Ok(self.admitted_request())
    }

    pub(crate) fn request_for_source(
        &self,
        source_instance: u64,
    ) -> Result<ExecutionRequest<'_, '_>, WorthQueryForeignAdvancementPhase> {
        if self
            .active
            .source_instance
            .is_some_and(|issued| issued != source_instance)
        {
            return Err(WorthQueryForeignAdvancementPhase(()));
        }
        Ok(self.admitted_request())
    }

    pub(in crate::domain_computation) fn request_for_managed(
        &self,
        observation: &crate::domain_computation::managed_run::WorthQueryManagedRelationalObservation,
    ) -> Result<ExecutionRequest<'_, '_>, WorthQueryForeignAdvancementPhase> {
        match observation.owner_identity() {
            Some(owner) => self.request_for_owner(owner),
            None => self.request_for_source(observation.identity().runtime_instance_id()),
        }
    }

    pub fn complete(self) {}
    pub(in crate::domain_computation::primary_graph) fn execution_for(
        &self,
        runtime: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime,
    ) -> Result<&QueryRequestExecution<'scope>, WorthQueryForeignAdvancementPhase> {
        self.request_for_owner(runtime.owner.owner_identity())?;
        Ok(self.active.execution)
    }
}

#[cfg(test)]
impl WorthQueryAdvancementPhase<'_> {
    /// Internal installation fixtures borrow their existing test request, without opening again.
    pub(crate) fn bootstrap_for_test(&self) -> super::WorthQueryBootstrapAdvancementPhase<'_> {
        super::WorthQueryBootstrapAdvancementPhase {
            active: self.active,
        }
    }
}
