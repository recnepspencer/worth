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
    ) -> Result<worth_execution::ExecutionAllocationPolicy<'_, '_>, WorthQueryCapturePolicyRefusal>
    {
        if !self.request.is_leased() {
            return Err(WorthQueryCapturePolicyRefusal::UnleasedFunding);
        }
        Ok(self.request.allocation_policy())
    }

    /// Admits the capture policy a caller named for this request. Capture is
    /// capped by that lease, which must be the request's funding lease or
    /// descend from it; a sibling or another lineage's lease refuses here,
    /// before any work or charge, rather than inside the capture after ACK.
    pub(in crate::domain_computation::primary_graph) fn admit_capture_policy<
        'policy,
        'authority,
    >(
        &self,
        capture: worth_execution::ExecutionAllocationPolicy<'policy, 'authority>,
    ) -> Result<
        worth_execution::ExecutionAllocationPolicy<'policy, 'authority>,
        WorthQueryCapturePolicyRefusal,
    > {
        use worth_execution::ExecutionAllocationPolicy as Policy;
        let funding = self.funded_capture_policy()?;
        match (funding, capture) {
            (Policy::Execution(funding), Policy::Execution(lease))
                if lease.descends_from(funding) =>
            {
                Ok(capture)
            }
            (Policy::Execution(_), Policy::Execution(_)) => {
                Err(WorthQueryCapturePolicyRefusal::ForeignCaptureLease)
            }
            // Naming no lease never inherits the funding lease; funding that is
            // not leased was already refused by `funded_capture_policy`.
            (Policy::Execution(_) | Policy::SystemAllocation, Policy::SystemAllocation)
            | (Policy::SystemAllocation, Policy::Execution(_)) => {
                Err(WorthQueryCapturePolicyRefusal::UnleasedCapture)
            }
        }
    }
}

/// Why a checkpoint capture's allocation policy was refused, before any work or charge.
/// Installation reports it as an execution denial.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryCapturePolicyRefusal {
    /// The transition's own request is not leased.
    UnleasedFunding,
    /// The named capture lease does not descend from the transition's funding lease.
    ForeignCaptureLease,
    /// Capture named no lease, or the funding is system allocation.
    UnleasedCapture,
}

impl<'scope> WorthQueryBootstrapAdvancementPhase<'scope> {
    pub(super) fn new(request: ExecutionRequest<'scope, 'scope>) -> Self {
        Self {
            request,
            _thread: std::marker::PhantomData,
        }
    }
}
