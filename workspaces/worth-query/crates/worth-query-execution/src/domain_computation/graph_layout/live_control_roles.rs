//! Installation derives record roles whose payload carries live security authority.
use std::collections::{BTreeMap, BTreeSet};
use worth_query_installation::facade::{
    ApplicationSchemaMember, ErasedApplicationSchemaDeclaration,
};
use worth_relational::facade::identity::KindId;
use {
    super::required_kind, super::WorthQueryPrimaryGraphLayout,
    crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenial,
};

pub(super) fn lower(
    schema: &ErasedApplicationSchemaDeclaration,
    kinds: &BTreeMap<String, KindId>,
) -> Result<BTreeSet<KindId>, WorthQueryPrimaryGraphInstallationDenial> {
    let mut roles = BTreeSet::new();
    for member in schema.members() {
        match member {
            ApplicationSchemaMember::PrincipalBinding {
                mapping_entity,
                principal_entity,
                ..
            } => {
                roles.insert(required_kind(kinds, mapping_entity)?);
                roles.insert(required_kind(kinds, principal_entity)?);
            }
            ApplicationSchemaMember::ApplicationCapability { contract } => {
                roles.insert(required_kind(kinds, contract.grant_entity())?);
            }
            _ => {}
        }
    }
    Ok(roles)
}

impl WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation) fn is_live_control_entity_kind(&self, kind: KindId) -> bool {
        self.live_control_entity_kinds.contains(&kind)
    }
}
