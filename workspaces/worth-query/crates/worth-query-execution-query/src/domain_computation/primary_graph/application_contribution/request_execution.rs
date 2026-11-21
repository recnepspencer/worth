use worth_query_execution_contracts::facade::WorthQueryManagedComputationResourceDenial;
use worth_query_execution_runtime::facade::request_execution as rt_req;

pub type Resource = WorthQueryManagedComputationResourceDenial;

pub use rt_req::{
    advancement_denial, advancement_observations_on_this_thread_for_test,
    advancement_requests_on_this_thread_for_test, bound_advancement_requests_on_this_thread_for_test,
    certified_wave_seed_for_test, execution_denial,
    place_managed_computations_on_this_thread_for_test, query_test_execution_authority_for_test,
    record_event, test_execution_workers_for_test,
    test_placement::{self, test_policy},
    with_bootstrap_advancement, with_bootstrap_advancement_from_request, with_test_advancement,
    MemberRequestExecution, QueryDispatch, QueryMemoryReservation, QueryRequestExecution,
    TestHostRequest, WorthQueryAdvancementDenial, WorthQueryAdvancementEventForTest,
    WorthQueryAdvancementEventKindForTest, WorthQueryAdvancementIdentityForTest,
    WorthQueryAdvancementObservationForTest, WorthQueryAdvancementPhase,
    WorthQueryBootstrapAdvancementPhase, WorthQueryExecutionPlacementForTest,
    WorthQueryForeignAdvancementPhase,
};
pub use rt_req::advancement::with_test_bootstrap_advancement;

mod bootstrap;
pub use bootstrap::BootstrapCapturePolicyExt;

pub mod advancement {
    pub use super::advancement_denial;
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

    pub mod test_host_policy {
        pub use super::TestHostRequest;
    }

    mod installed_opening;

    pub mod observation {
        pub use worth_query_execution_runtime::facade::request_execution::advancement::observation::*;
        pub mod application;
        pub use application::{
            caller_pass_reports_on_this_thread_for_test, record_caller_pass_for_test,
        };
    }

    pub use observation::{
        caller_pass_reports_on_this_thread_for_test, record_caller_pass_for_test,
    };
}

pub use advancement::{caller_pass_reports_on_this_thread_for_test, record_caller_pass_for_test};
