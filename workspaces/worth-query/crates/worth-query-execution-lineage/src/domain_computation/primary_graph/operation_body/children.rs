//! Static parent slots plus request-funded retention of actual candidate-created handles.
use super::super::{
    application_contribution::QueryMemoryReservation, WorthQueryApplicationEffectEntity,
};
pub struct PreparedBodyChildren<S> {
    pub targets: Vec<WorthQueryApplicationEffectEntity<S, ()>>,
    pub _memory: QueryMemoryReservation,
}
pub fn parent_indices(
    body: &worth_query_declaration::facade::application_schema::ApplicationOperationBodyProgram,
) -> Vec<usize> {
    let Some(root) = body.creation() else {
        return Vec::new();
    };
    let mut parents = Vec::new();
    for tree in root.creation_roots() {
        let mut sites = std::collections::BTreeMap::from([(tree.key(), 0)]);
        for child in tree.children() {
            let parent = *sites
                .get(child.parent_key.as_str())
                .expect("installed prior parent");
            parents.push(parent);
            sites.insert(child.creation.key(), parents.len());
        }
    }
    parents
}
