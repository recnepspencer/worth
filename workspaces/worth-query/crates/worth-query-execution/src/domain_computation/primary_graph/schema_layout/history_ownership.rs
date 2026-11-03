//! Install the structural ownership recipes once; ordinary admission borrows them.
use super::*;
use worth_query_installation::facade::ApplicationSchemaMember;
use worth_relational::facade::branch::{RelationalModelOwnership, RelationalModelOwnershipDomain};

pub(super) fn lower(
    schema: &ErasedApplicationSchemaDeclaration,
    entities: &BTreeMap<String, KindId>,
    relations: &BTreeMap<String, WorthQueryPrimaryRelationLayout>,
    live_controls: &BTreeSet<KindId>,
) -> Result<BTreeMap<String, RelationalModelOwnership>, WorthQueryPrimaryGraphInstallationDenial> {
    let mut result = BTreeMap::new();
    for member in schema.members() {
        let ApplicationSchemaMember::ApplicationHistory { definition } = member else {
            continue;
        };
        let parts = definition.parts();
        let root = required_kind(entities, &parts.access.parts().scope_entity)?;
        let mut kinds = Vec::new();
        let mut edges = Vec::new();
        for edge in &parts.ownership {
            let relation = relations
                .get(&edge.relation)
                .ok_or_else(|| invalid_member(&edge.relation))?;
            if [relation.from, relation.to]
                .into_iter()
                .any(|kind| live_controls.contains(&kind))
            {
                return Err(invalid_member(&edge.relation));
            }
            kinds.extend([relation.from, relation.to]);
            edges.push(relation.kind);
        }
        result.insert(
            parts.identity.clone(),
            RelationalModelOwnership::new(vec![RelationalModelOwnershipDomain {
                root,
                entities: kinds,
                relations: edges,
            }]),
        );
    }
    Ok(result)
}

impl WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation::primary_graph) fn history_ownership(
        &self,
        identity: &str,
    ) -> Option<&RelationalModelOwnership> {
        self.history_ownership.get(identity)
    }
    pub(in crate::domain_computation::primary_graph) fn all_history_ownership(
        &self,
    ) -> RelationalModelOwnership {
        RelationalModelOwnership::combine(self.history_ownership.values())
    }
}
