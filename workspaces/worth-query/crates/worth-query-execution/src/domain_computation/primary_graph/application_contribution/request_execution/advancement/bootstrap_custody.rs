use super::ExecutionRequest;

/// Installation custody cannot fund an already installed runtime's public doors.
/// Only an installation callback can borrow this phase; it cannot escape the call.
/// World has not issued an installation identity when this callback opens.
///
/// ```compile_fail,E0308
/// use worth_query_execution::facade::{
///     application_contribution::{WorthQueryAdvancementPhase, WorthQueryBootstrapAdvancementPhase},
///     provider_session::{ExecutionRequest, WorthQueryAdmittedProviderExecutionPlan},
/// };
/// fn call<'run, 'scope>(
///     plan: WorthQueryAdmittedProviderExecutionPlan<'run>,
///     installed: &'run WorthQueryAdvancementPhase<'scope>,
///     bootstrap: &'run WorthQueryBootstrapAdvancementPhase<'scope>,
///     raw: ExecutionRequest<'run, 'run>,
/// ) {
///     let _ = plan.readmit(bootstrap);
/// }
/// ```
/// ```
/// use worth_query_execution::facade::{
///     application_contribution::{WorthQueryAdvancementPhase, WorthQueryBootstrapAdvancementPhase},
///     provider_session::{ExecutionRequest, WorthQueryAdmittedProviderExecutionPlan},
/// };
/// fn call<'run, 'scope>(
///     plan: WorthQueryAdmittedProviderExecutionPlan<'run>,
///     installed: &'run WorthQueryAdvancementPhase<'scope>,
///     bootstrap: &'run WorthQueryBootstrapAdvancementPhase<'scope>,
///     raw: ExecutionRequest<'run, 'run>,
/// ) {
///     let _ = plan.readmit(installed);
/// }
/// ```
/// A host-built raw request cannot readmit an installed provider plan.
/// ```compile_fail,E0308
/// use worth_query_execution::facade::{
///     application_contribution::{WorthQueryAdvancementPhase, WorthQueryBootstrapAdvancementPhase},
///     provider_session::{ExecutionRequest, WorthQueryAdmittedProviderExecutionPlan},
/// };
/// fn call<'run, 'scope>(
///     plan: WorthQueryAdmittedProviderExecutionPlan<'run>,
///     installed: &'run WorthQueryAdvancementPhase<'scope>,
///     bootstrap: &'run WorthQueryBootstrapAdvancementPhase<'scope>,
///     raw: ExecutionRequest<'run, 'run>,
/// ) {
///     let _ = plan.readmit(raw);
/// }
/// ```
/// ```
/// use worth_query_execution::facade::{
///     application_contribution::{WorthQueryAdvancementPhase, WorthQueryBootstrapAdvancementPhase},
///     provider_session::{ExecutionRequest, WorthQueryAdmittedProviderExecutionPlan},
/// };
/// fn call<'run, 'scope>(
///     plan: WorthQueryAdmittedProviderExecutionPlan<'run>,
///     installed: &'run WorthQueryAdvancementPhase<'scope>,
///     bootstrap: &'run WorthQueryBootstrapAdvancementPhase<'scope>,
///     raw: ExecutionRequest<'run, 'run>,
/// ) {
///     let _ = plan.readmit(installed);
/// }
/// ```
pub struct WorthQueryBootstrapAdvancementPhase<'scope> {
    request: ExecutionRequest<'scope, 'scope>,
    _thread: std::marker::PhantomData<std::rc::Rc<()>>,
}

impl WorthQueryBootstrapAdvancementPhase<'_> {
    /// Borrows the same installation request at an execution owner's boundary.
    pub fn execution_request(&self) -> ExecutionRequest<'_, '_> {
        self.request
    }
}

impl WorthQueryBootstrapAdvancementPhase<'_> {
    pub(in crate::domain_computation::primary_graph) fn funded_capture_policy(
        &self,
    ) -> Result<
        worth_execution::ExecutionAllocationPolicy<'_, '_>,
        crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial,
    > {
        use crate::domain_computation::primary_graph::{
            WorthQueryPrimaryGraphInstallationDenial as D,
            WorthQueryPrimaryGraphInstallationDenialKind as K,
        };
        if !self.request.is_leased() {
            return Err(D::new(K::ExecutionDenied {
                kind: crate::domain_computation::WorthQueryProviderSessionDenialKind::ResourceEnvelopeMismatch,
            }, "original-request checkpoint capture requires leased backing; serial system allocation is unsupported"));
        }
        Ok(self.request.allocation_policy())
    }
}

impl<'scope> WorthQueryBootstrapAdvancementPhase<'scope> {
    pub(super) fn new(request: ExecutionRequest<'scope, 'scope>) -> Self {
        Self {
            request,
            _thread: std::marker::PhantomData,
        }
    }
}
