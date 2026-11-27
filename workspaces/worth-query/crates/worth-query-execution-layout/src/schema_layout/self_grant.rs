//! Grantee relations whose grant kind forbids an admitted operation granting
//! to its own principal, lowered once from the installed capability contracts.
use std::collections::{BTreeMap, BTreeSet};

use worth_query_declaration::facade::application_capability::ApplicationCapabilitySelfGrantRule;
use worth_query_installation::facade::{
    ApplicationSchemaMember, ErasedApplicationSchemaDeclaration,
};
use worth_relational::facade::identity::KindId;

use super::{invalid_member, WorthQueryPrimaryRelationLayout, WorthQuerySchemaLayoutDenial};

/// One contract forbidding self-grant through a grantee relation forbids it
/// for every operation that links that relation.
pub(super) fn lower_forbidden_self_grants(
    schema: &ErasedApplicationSchemaDeclaration,
    relations: &BTreeMap<String, WorthQueryPrimaryRelationLayout>,
) -> Result<BTreeSet<KindId>, WorthQuerySchemaLayoutDenial> {
    schema
        .members()
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::ApplicationCapability { contract } => {
                Some(contract.delegation())
            }
            _ => None,
        })
        .filter(|delegation| {
            delegation.self_grant() == ApplicationCapabilitySelfGrantRule::Forbidden
        })
        .map(|delegation| {
            let grantee = delegation.grantee().relation();
            relations
                .get(grantee)
                .map(|relation| relation.kind)
                .ok_or_else(|| invalid_member(grantee))
        })
        .collect()
}

impl super::WorthQueryPrimaryGraphLayout {
    /// Whether linking `relation` from the operation's own admitted principal
    /// would grant that principal a self-grant-forbidden capability.
    pub fn forbids_self_grant(&self, relation: KindId) -> bool {
        self.forbidden_self_grant_relations.contains(&relation)
    }
}
