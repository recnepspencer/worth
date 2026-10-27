//! Original Native cohorts and their whole-slot query meaning, never a member cache.
use super::*;
use crate::domain_computation::primary_graph::component_selection::ComponentSelection;
use std::sync::Arc;
use worth_query_declaration::facade::application_query::ApplicationQueryComponentMeaning;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph::application_query) struct DeclaredComponentSelection
{
    pub meaning: Arc<ApplicationQueryComponentMeaning>,
    pub body: Arc<ComponentSelection>,
    pub binding: ApplicationSchemaBindingIdentity,
}
impl DeclaredComponentSelection {
    pub(super) fn append_descriptor(&self, digest: &mut Sha256) {
        digest.update(b"declared-components:v1");
        self.meaning.visit_canonical_parts(|key, value| {
            crate::domain_computation::primary_graph::source_facts::encode_text(digest, key);
            crate::domain_computation::primary_graph::source_facts::encode_text(digest, value);
        });
        digest.update(self.body.commitment());
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn matches_rows(
        &self,
        rows: impl IntoIterator<Item = EntityId>,
    ) -> bool {
        let Some(aggregates) = self.body.physical_aggregates() else {
            return false;
        };
        aggregates
            .iter()
            .flat_map(|aggregate| aggregate.native().members())
            .eq(rows)
    }
}
