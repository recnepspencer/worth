//! Query admission authority.
//!
//! This package owns basis, policy, support, resource, and descriptive
//! graph-read planning decisions together with proof-bearing handoffs accepted
//! by execution, including bounded preparation buffers. Executable plans,
//! provider contact, execution, and publication belong to execution.

#![forbid(unsafe_code)]

mod admission_digest;
mod application_query;
mod authenticated_principal;
mod authentication_event;
mod canonical_identity_derivation;
mod domain_computation;
mod graph_obligation;
mod graph_read_access;

pub mod facade;

#[doc(hidden)]
pub mod integration {
    pub use crate::application_query::requirements::derive_graph_read_access_requirements_for_contract;
    pub use crate::application_query::requirements::derive_graph_read_access_requirements_for_contract_admitted;
    pub use crate::canonical_identity_derivation::WorthQueryCanonicalIdentityStop;
    pub use crate::domain_computation::execution_resource_admission::{
        admit_execution_resource_plan, reserve_execution_resource_plan,
        reserve_workflow_resource_plan, WorthQueryCapacityReservedExecutionResourcePlan,
        WorthQueryCapacityReservedWorkflowResourcePlan, WorthQueryExecutionCapacityReleaseReceipt,
        WorthQueryExecutionCapacityReservationScope,
    };
    pub use crate::graph_obligation::{
        admit_application_operation_graph_work, admit_application_operation_read_graph_work,
        admit_application_query_graph_work, admit_application_query_graph_work_admitted,
        review_application_query_graph_work, review_application_query_graph_work_admitted,
        select_installed_graph_obligations, select_installed_graph_obligations_admitted,
        select_shared_application_query_graph_obligations_admitted,
        WorthQueryGraphObligationSelectionAdmissionStop, WorthQueryGraphWorkCapacityAdmissionStop,
        WorthQueryGraphWorkReviewAdmissionStop, WorthQueryReviewedApplicationQueryGraphWork,
    };
    pub use crate::graph_read_access::plan_review::review_graph_read_access;
    pub use crate::graph_read_access::{
        derive_canonical_graph_read_access_requirements, WorthQueryCanonicalGraphReadPlanningInput,
        WorthQueryGraphReadPlanningIdentity, WorthQueryGraphReadPlanningOrderingField,
        WorthQueryGraphReadPlanningPredicateField, WorthQueryGraphReadPlanningRelation,
        WorthQueryGraphReadPlanningShape,
    };
}
