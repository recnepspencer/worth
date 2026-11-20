use std::ops::Deref;
use worth_execution::ExecutionRequest;
use worth_query_execution_contracts::facade::{
    WorthQueryManagedComputationInterruption, WorthQueryManagedComputationResourceDenial,
    WorthQueryMemoryLimitLevel,
};
use worth_query_execution_runtime::facade::ports::ProductRuntimeRef;
use worth_query_execution_runtime::facade::request_execution as rt_req;

pub type Resource = WorthQueryManagedComputationResourceDenial;

pub use rt_req::{
    advancement_observations_on_this_thread_for_test, advancement_requests_on_this_thread_for_test,
    bound_advancement_requests_on_this_thread_for_test, certified_wave_seed_for_test,
    execution_denial, place_managed_computations_on_this_thread_for_test,
    query_test_execution_authority_for_test, record_event, test_execution_workers_for_test,
    test_placement::{self, test_policy}, MemberRequestExecution, QueryDispatch, QueryMemoryReservation,
    QueryRequestExecution, WorthQueryAdvancementEventForTest,
    WorthQueryAdvancementEventKindForTest, WorthQueryAdvancementIdentityForTest,
    WorthQueryAdvancementObservationForTest, WorthQueryExecutionPlacementForTest,
    WorthQueryForeignAdvancementPhase,
};

mod bootstrap;
pub use bootstrap::{
    with_bootstrap_advancement, with_bootstrap_advancement_from_request,
    WorthQueryBootstrapAdvancementPhase,
};

/// Branded advancement custody borrowed for one installed host/application call.
#[repr(transparent)]
pub struct WorthQueryAdvancementPhase<'scope>(pub(crate) rt_req::WorthQueryAdvancementPhase<'scope>);

impl<'scope> Deref for WorthQueryAdvancementPhase<'scope> {
    type Target = rt_req::WorthQueryAdvancementPhase<'scope>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'scope> WorthQueryAdvancementPhase<'scope> {
    pub fn checked_execution_request_for(
        &self,
        runtime: &worth_query_execution_runtime::facade::execution_runtime::product_world::WorthQueryProductRuntime,
    ) -> Result<ExecutionRequest<'scope, 'scope>, WorthQueryAdvancementDenial> {
        self.0
            .checked_execution_request_for(runtime)
            .map_err(Into::into)
    }

    pub fn check_live_for(
        &self,
        runtime: &worth_query_execution_runtime::facade::execution_runtime::product_world::WorthQueryProductRuntime,
    ) -> Result<(), WorthQueryAdvancementDenial> {
        self.0.check_live_for(runtime).map_err(Into::into)
    }

    pub fn test_host_request(&self) -> TestHostRequest<'scope> {
        TestHostRequest(self.0.test_host_request())
    }

    #[doc(hidden)]
    pub fn test_host_request_for_test(&self) -> TestHostRequest<'scope> {
        self.test_host_request()
    }
}

/// Fixture host custody distinct from an installed runtime's phase.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct TestHostRequest<'scope>(pub(crate) rt_req::TestHostRequest<'scope>);

impl<'scope> Deref for TestHostRequest<'scope> {
    type Target = rt_req::TestHostRequest<'scope>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'scope> TestHostRequest<'scope> {
    pub fn test_host_request(&self) -> Self {
        *self
    }

    #[doc(hidden)]
    pub fn test_host_request_for_test(&self) -> Self {
        *self
    }

    pub fn bootstrap_for_test(&self) -> WorthQueryBootstrapAdvancementPhase<'_> {
        WorthQueryBootstrapAdvancementPhase(self.0.bootstrap_for_test())
    }

    pub fn phase_for_application<R: ProductRuntimeRef>(
        &self,
        runtime: &R,
    ) -> WorthQueryAdvancementPhase<'scope> {
        WorthQueryAdvancementPhase(self.0.phase_for_application(runtime))
    }

    #[doc(hidden)]
    pub fn phase_for_application_for_test<R: ProductRuntimeRef>(
        &self,
        runtime: &R,
    ) -> WorthQueryAdvancementPhase<'scope> {
        self.phase_for_application(runtime)
    }

    pub fn phase_for_product(
        &self,
        runtime: &worth_query_execution_runtime::facade::execution_runtime::product_world::WorthQueryProductRuntime,
    ) -> WorthQueryAdvancementPhase<'scope> {
        WorthQueryAdvancementPhase(self.0.phase_for_product(runtime))
    }

    pub fn phase_for_direct(
        &self,
        run: &crate::domain_computation::WorthQueryRunningDirectRun,
    ) -> WorthQueryAdvancementPhase<'scope> {
        WorthQueryAdvancementPhase(self.0.phase_for_direct(run))
    }

    #[doc(hidden)]
    pub fn phase_for_direct_for_test(
        &self,
        run: &crate::domain_computation::WorthQueryRunningDirectRun,
    ) -> WorthQueryAdvancementPhase<'scope> {
        self.phase_for_direct(run)
    }

    pub fn phase_for_workflow(
        &self,
        run: &crate::domain_computation::WorthQueryRunningWorkflowRun,
    ) -> WorthQueryAdvancementPhase<'scope> {
        WorthQueryAdvancementPhase(self.0.phase_for_workflow(run))
    }

    #[doc(hidden)]
    pub fn phase_for_workflow_for_test(
        &self,
        run: &crate::domain_computation::WorthQueryRunningWorkflowRun,
    ) -> WorthQueryAdvancementPhase<'scope> {
        self.phase_for_workflow(run)
    }

    pub fn phase_for_managed(
        &self,
        observation: &crate::domain_computation::managed_run::WorthQueryManagedRelationalObservation,
    ) -> WorthQueryAdvancementPhase<'scope> {
        WorthQueryAdvancementPhase(self.0.phase_for_managed(observation))
    }
}

pub fn with_test_advancement<R>(
    body: impl for<'scope> FnOnce(TestHostRequest<'scope>) -> R,
) -> R {
    rt_req::with_test_advancement(|req| body(TestHostRequest(req)))
}

pub fn with_test_bootstrap_advancement<R>(
    work: u64,
    body: impl for<'scope> FnOnce(TestHostRequest<'scope>) -> R,
) -> R {
    rt_req::advancement::with_test_bootstrap_advancement(work, |req| body(TestHostRequest(req)))
}

/// Why an advancement could not return its ordinary result.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorthQueryAdvancementDenial {
    Resource(Resource),
    Interrupted(WorthQueryManagedComputationInterruption),
    /// A public call tried to open an advancement from an active host call or
    /// a member compute.
    NestedOpening,
    NestedStopped,
    Panicked,
}

impl std::fmt::Display for WorthQueryAdvancementDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "advancement request refused: {self:?}")
    }
}

impl std::error::Error for WorthQueryAdvancementDenial {}

impl WorthQueryAdvancementDenial {
    pub const fn is_transient(self) -> bool {
        match self {
            Self::Resource(Resource::NativeAllocationRefused { kind, .. }) => matches!(
                kind,
                worth_execution::ExecutionPhysicalAllocationDenialKind::Allocator
            ),
            Self::Resource(Resource::MemoryLimit { level, .. }) => match level {
                WorthQueryMemoryLimitLevel::Process => true,
                WorthQueryMemoryLimitLevel::Policy | WorthQueryMemoryLimitLevel::Declared => false,
            },
            Self::Resource(
                Resource::WorkExhausted
                | Resource::WorkCounterOverflow
                | Resource::AccountingUnavailable
                | Resource::RetainedBytesExhausted
                | Resource::ScratchCapacityExceeded
                | Resource::ResultCapacityExceeded
                | Resource::CapacityOverflow
                | Resource::ChargedBytesOverflow
                | Resource::WorkerLimit
                | Resource::PolicyMemoryLimit
                | Resource::WorkLimit
                | Resource::NestedLeaseMisuse
                | Resource::NestedAdvancementOpening
                | Resource::ForeignAdmissionClock
                | Resource::ForeignAdvancementPhase
                | Resource::NoActiveExecutionScope
                | Resource::EquivalenceContractUnavailable,
            )
            | Self::Interrupted(
                WorthQueryManagedComputationInterruption::Cancelled
                | WorthQueryManagedComputationInterruption::DeadlineExceeded,
            )
            | Self::NestedOpening
            | Self::NestedStopped
            | Self::Panicked => false,
        }
    }

    pub(crate) fn provider_denial_cause(
        self,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderSessionDenialKind,
        crate::domain_computation::WorthQueryProviderSessionControlStopKind,
    > {
        rt_req::WorthQueryAdvancementDenial::from(self).provider_denial_cause()
    }
}

impl From<WorthQueryForeignAdvancementPhase> for WorthQueryAdvancementDenial {
    fn from(_: WorthQueryForeignAdvancementPhase) -> Self {
        Self::Resource(Resource::ForeignAdvancementPhase)
    }
}

impl From<rt_req::WorthQueryAdvancementDenial> for WorthQueryAdvancementDenial {
    fn from(value: rt_req::WorthQueryAdvancementDenial) -> Self {
        match value {
            rt_req::WorthQueryAdvancementDenial::Resource(r) => Self::Resource(r),
            rt_req::WorthQueryAdvancementDenial::Interrupted(i) => Self::Interrupted(i),
            rt_req::WorthQueryAdvancementDenial::NestedOpening => Self::NestedOpening,
            rt_req::WorthQueryAdvancementDenial::NestedStopped => Self::NestedStopped,
            rt_req::WorthQueryAdvancementDenial::Panicked => Self::Panicked,
        }
    }
}

impl From<WorthQueryAdvancementDenial> for rt_req::WorthQueryAdvancementDenial {
    fn from(value: WorthQueryAdvancementDenial) -> Self {
        match value {
            WorthQueryAdvancementDenial::Resource(r) => Self::Resource(r),
            WorthQueryAdvancementDenial::Interrupted(i) => Self::Interrupted(i),
            WorthQueryAdvancementDenial::NestedOpening => Self::NestedOpening,
            WorthQueryAdvancementDenial::NestedStopped => Self::NestedStopped,
            WorthQueryAdvancementDenial::Panicked => Self::Panicked,
        }
    }
}

pub(crate) fn advancement_denial(
    denial: worth_execution::WorkCeilingDenial,
) -> WorthQueryAdvancementDenial {
    rt_req::advancement_denial(denial).into()
}

pub(crate) mod advancement {
    pub(crate) use super::advancement_denial;
    pub use super::{
        with_bootstrap_advancement, with_bootstrap_advancement_from_request,
        with_test_advancement, with_test_bootstrap_advancement, TestHostRequest,
        WorthQueryAdvancementDenial, WorthQueryAdvancementPhase,
        WorthQueryBootstrapAdvancementPhase,
    };
    pub use worth_query_execution_runtime::facade::request_execution::advancement::{
        advancement_observations_on_this_thread_for_test,
        advancement_requests_on_this_thread_for_test, opening_custody, runner,
        WorthQueryAdvancementEventForTest, WorthQueryAdvancementEventKindForTest,
        WorthQueryAdvancementIdentityForTest, WorthQueryAdvancementObservationForTest,
        WorthQueryForeignAdvancementPhase,
    };

    pub(crate) mod test_host_policy {
        pub use super::TestHostRequest;
    }

    mod installed_opening;

    pub(crate) mod observation {
        pub use worth_query_execution_runtime::facade::request_execution::advancement::observation::*;
        pub(in crate::domain_computation::primary_graph) mod application;
        pub use application::{
            caller_pass_reports_on_this_thread_for_test, record_caller_pass_for_test,
        };
    }

    pub use observation::{
        caller_pass_reports_on_this_thread_for_test, record_caller_pass_for_test,
    };
}

pub use advancement::{caller_pass_reports_on_this_thread_for_test, record_caller_pass_for_test};
