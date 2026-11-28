//! Real SDK-created partitions, judged only by the native endpoint law.
pub use super::{installation::*, outcome::*, read};
pub use worth_query_host::facade::{
    application_invariants::EntityReference,
    primary_graph::{InvariantViolationFields, WorthQueryApplicationCommitDenial},
};

pub fn endpoints(
    denial: &WorthQueryApplicationCommitDenial,
) -> (&EntityReference, &EntityReference) {
    let Some(InvariantViolationFields::PartitionIsolation {
        contract_id,
        source_partition_id,
        target_partition_id,
        source,
        target,
        ..
    }) = denial.relation_law()
    else {
        panic!("original partition law: {denial:?}")
    };
    assert_eq!(contract_id.as_str(), "Precedes:same_partition_endpoints");
    assert_eq!(*source_partition_id, source.partition_id());
    assert_eq!(*target_partition_id, target.partition_id());
    assert_ne!(source_partition_id, target_partition_id);
    (source, target)
}
