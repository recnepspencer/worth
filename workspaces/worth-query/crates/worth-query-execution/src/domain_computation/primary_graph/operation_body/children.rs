//! Static parent slots plus request-funded retention of actual candidate-created handles.
use super::super::{
    application_contribution::QueryMemoryReservation, WorthQueryApplicationEffectEntity,
};
pub(in crate::domain_computation::primary_graph) struct PreparedBodyChildren<S> {
    pub(in crate::domain_computation::primary_graph) targets:
        Vec<WorthQueryApplicationEffectEntity<S, ()>>,
    pub(in crate::domain_computation::primary_graph) _memory: QueryMemoryReservation,
}
pub(super) fn parent_indices(
    body: &worth_query_declaration::facade::application_schema::ApplicationOperationBodyProgram,
) -> Vec<usize> {
    let Some(root) = body.creation() else {
        return Vec::new();
    };
    let mut sites = std::collections::BTreeMap::from([(root.key(), 0)]);
    root.children()
        .iter()
        .enumerate()
        .map(|(index, child)| {
            let parent = *sites
                .get(child.parent_key.as_str())
                .expect("installed prior parent");
            sites.insert(child.creation.key(), index + 1);
            parent
        })
        .collect()
}
