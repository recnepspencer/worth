//! Minimum counts distinct actual targets across the full proposed mutation.
pub use super::{feeding::FeedsChange, installation::*, outcome::*, read};
pub use worth_query_host::facade::{
    application_invariants::{EntityId, EntityReference},
    primary_graph::{WorthQueryApplicationCommitDenial, WorthQueryRelationLawDenial},
};
pub fn minimum(denial: &WorthQueryApplicationCommitDenial, source_id: EntityId, count: usize) {
    let Some(WorthQueryRelationLawDenial::ConnectivityMinimum {
        contract_id,
        source,
        reachable_target_count,
        minimum_reachable_targets,
        ..
    }) = denial.relation_law()
    else {
        panic!("original minimum refusal: {denial:?}")
    };
    assert_eq!(contract_id.as_str(), "Feeds:reaches_at_least");
    assert_eq!(source, &EntityReference::Existing(source_id));
    assert_eq!(reachable_target_count, count);
    assert_eq!(minimum_reachable_targets, 2);
}
