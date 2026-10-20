//! Native-owned immutable consequence records and protected association kinds.
use super::{required_kind, WorthQueryPrimaryGraphInstallationDenial as Denial};
use std::collections::BTreeSet;
use worth_query_installation::facade::{
    ApplicationSchemaMember as Member, ErasedApplicationSchemaDeclaration,
};
use worth_relational::facade::identity::KindId;

#[derive(Debug, Default)]
pub(in crate::domain_computation::primary_graph) struct SigningLifecycleLayout {
    records: BTreeSet<KindId>,
    associations: BTreeSet<KindId>,
    consequences: BTreeSet<KindId>,
}
impl SigningLifecycleLayout {
    pub(super) fn lower(
        schema: &ErasedApplicationSchemaDeclaration,
        entities: &std::collections::BTreeMap<String, KindId>,
        relations: &std::collections::BTreeMap<String, KindId>,
    ) -> Result<Self, Denial> {
        let mut layout = Self::default();
        for member in schema.members() {
            let Member::OperationBody { body, .. } = member else {
                continue;
            };
            let Some(spec) = body
                .creation()
                .and_then(|creation| creation.signing_lifecycle())
            else {
                continue;
            };
            layout
                .records
                .insert(required_kind(entities, spec.consequence_field.entity())?);
            layout
                .associations
                .insert(required_kind(relations, &spec.history_relation)?);
            layout
                .associations
                .insert(required_kind(relations, &spec.current_relation)?);
            layout
                .consequences
                .insert(required_kind(relations, &spec.consequence_relation)?);
        }
        Ok(layout)
    }
    pub(in crate::domain_computation::primary_graph) fn is_record(&self, kind: KindId) -> bool {
        self.records.contains(&kind)
    }
    pub(in crate::domain_computation::primary_graph) fn is_association(
        &self,
        kind: KindId,
    ) -> bool {
        self.associations.contains(&kind)
    }
    pub(in crate::domain_computation::primary_graph) fn is_consequence(
        &self,
        kind: KindId,
    ) -> bool {
        self.consequences.contains(&kind)
    }
    pub(in crate::domain_computation::primary_graph) fn is_protected_relation(
        &self,
        kind: KindId,
    ) -> bool {
        self.is_association(kind) || self.is_consequence(kind)
    }
    pub(in crate::domain_computation::primary_graph) fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}
