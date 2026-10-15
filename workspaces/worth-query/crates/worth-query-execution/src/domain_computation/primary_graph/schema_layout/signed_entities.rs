//! Native immutable entity classification derived from the authored signing declarations.
use super::{
    required_kind, WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphLayout,
};
use std::collections::{BTreeMap, BTreeSet};
use worth_query_installation::facade::{
    ApplicationSchemaMember, ErasedApplicationSchemaDeclaration,
};
use worth_relational::facade::identity::KindId;

pub(super) fn lower(
    schema: &ErasedApplicationSchemaDeclaration,
    kinds: &BTreeMap<String, KindId>,
) -> Result<BTreeSet<KindId>, WorthQueryPrimaryGraphInstallationDenial> {
    schema
        .members()
        .iter()
        .filter_map(|member| {
            let ApplicationSchemaMember::OperationBody { body, .. } = member else {
                return None;
            };
            body.creation()
                .filter(|creation| creation.signing().is_some())
                .map(|creation| required_kind(kinds, creation.entity()))
        })
        .collect()
}
impl WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation::primary_graph) fn has_signed_entity_kinds(&self) -> bool {
        !self.signed_entity_kinds.is_empty()
    }
    pub(in crate::domain_computation::primary_graph) fn is_signed_entity_kind(
        &self,
        kind: KindId,
    ) -> bool {
        self.signed_entity_kinds.contains(&kind)
    }
}
