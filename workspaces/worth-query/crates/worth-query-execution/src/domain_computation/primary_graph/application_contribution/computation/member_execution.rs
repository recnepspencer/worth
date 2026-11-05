//! The member's bounded execution, without any owner or graph access.
use crate::domain_computation::primary_graph::vocabulary::application_contribution::nested_failure::NestedFailureTransport;
use {crate::domain_computation::primary_graph::vocabulary::application_contribution::checkpoint::WorthQueryManagedComputationExecution, crate::domain_computation::request_execution::execution_refusal::WorthQueryManagedComputationInterruption};
use {
    crate::domain_computation::request_execution::MemberRequestExecution,
    crate::domain_computation::request_execution::QueryRequestExecution,
};


mod court_doors_for_test;
