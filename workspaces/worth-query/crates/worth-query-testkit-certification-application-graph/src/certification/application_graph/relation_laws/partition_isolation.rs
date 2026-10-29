//! Real SDK-created partitions, judged only by the native endpoint law.
pub use super::{installation::*, outcome::*, read};
pub use worth_query_host::facade::{
    application_invariants::EntityReference,
    primary_graph::{WorthQueryApplicationCommitDenial, WorthQueryRelationLawDenial},
};

pub fn endpoints(
    denial: &WorthQueryApplicationCommitDenial,
) -> (&EntityReference, &EntityReference) {
    let Some(WorthQueryRelationLawDenial::PartitionIsolation {
        contract_id,
        source,
        target,
        ..
    }) = denial.relation_law()
    else {
        panic!("original partition law: {denial:?}")
    };
    assert_eq!(contract_id.as_str(), "Precedes:same_partition_endpoints");
    assert_ne!(source.partition_id(), target.partition_id());
    (source, target)
}
