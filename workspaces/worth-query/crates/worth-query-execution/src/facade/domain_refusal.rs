//! An application's own structured refusal reason, shared by operation
//! handlers (`HandlerResult::domain_refusal`) and installed invariants
//! (`WorthQueryDomainRefusalBinding`).
pub use worth_query_execution_lineage::facade::primary_graph::domain_refusal::{
    WorthQueryDomainRefusal, WorthQueryDomainRefusalBinding, WorthQueryDomainRefusalCarrier,
    WorthQueryDomainRefusalDenial, WORTH_QUERY_DOMAIN_REFUSAL_CODE_MAXIMUM_BYTES,
    WORTH_QUERY_DOMAIN_REFUSAL_MESSAGE_MAXIMUM_BYTES,
};
