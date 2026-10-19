//! One current workflow interpretation; unsupported predecessors refuse.
pub(in crate::domain_computation::primary_graph) const WORKFLOW_FACT_PROTOCOL_VERSION: u64 = 2;
/// Performed operation transitions require their complete recorded output context.
pub(in crate::domain_computation::primary_graph) const WORKFLOW_OPERATION_OUTPUT_PROTOCOL_VERSION: u64 = 3;
pub(in crate::domain_computation::primary_graph) const WORKFLOW_INSTANCE_FACT_PROTOCOL_VERSION:
    u64 = 4;
/// Birth uses an edge to its newly created subject and carries the current actor/provenance fields.
pub(in crate::domain_computation::primary_graph) const WORKFLOW_BIRTH_FACT_PROTOCOL_VERSION: u64 =
    5;
