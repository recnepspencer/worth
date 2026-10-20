//! One current workflow interpretation; unsupported predecessors refuse.
pub(in crate::domain_computation::primary_graph) const WORKFLOW_FACT_PROTOCOL_VERSION: u64 = 2;
/// A proposal atomically retains its complete captured target-input inventory.
pub(in crate::domain_computation::primary_graph) const WORKFLOW_CAPTURED_PROPOSAL_PROTOCOL_VERSION: u64 = 6;
/// Performed operation transitions require their complete recorded output context.
pub(in crate::domain_computation::primary_graph) const WORKFLOW_OPERATION_OUTPUT_PROTOCOL_VERSION: u64 = 3;
pub(in crate::domain_computation::primary_graph) const WORKFLOW_INSTANCE_FACT_PROTOCOL_VERSION:
    u64 = 4;
/// Birth uses an edge to its newly created subject and carries the current actor/provenance fields.
pub(in crate::domain_computation::primary_graph) const WORKFLOW_BIRTH_FACT_PROTOCOL_VERSION: u64 =
    5;
