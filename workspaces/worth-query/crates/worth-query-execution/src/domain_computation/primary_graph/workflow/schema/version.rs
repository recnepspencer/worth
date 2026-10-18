//! One current workflow interpretation; unsupported predecessors refuse.
pub(in crate::domain_computation::primary_graph) const WORKFLOW_FACT_PROTOCOL_VERSION: u64 = 2;
pub(in crate::domain_computation::primary_graph) const WORKFLOW_INSTANCE_FACT_PROTOCOL_VERSION:
    u64 = 4;
/// Birth uses an edge to its newly created subject and carries the current actor/provenance fields.
pub(in crate::domain_computation::primary_graph) const WORKFLOW_BIRTH_FACT_PROTOCOL_VERSION: u64 =
    5;
