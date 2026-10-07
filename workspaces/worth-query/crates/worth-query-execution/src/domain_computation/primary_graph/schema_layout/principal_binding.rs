use std::collections::BTreeMap;

use worth_foundational::facade::AspectFieldLocator;
use worth_query_installation::facade::{
    ApplicationSchemaMember, ErasedApplicationSchemaDeclaration,
};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::indexes::DerivedIndexId;

use super::{
    contract_space_exhausted, planned_field_locator, required_kind,
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphLayout,
};

impl WorthQueryPrimaryGraphLayout {
    /// Comparison Work before the one borrowed installed-principal lookup.
    /// The selected key has no owned copy; each comparison checks the tag and
    /// at most its initialized UTF-8 bytes.
    pub(in crate::domain_computation::primary_graph) fn principal_lookup_work(
        &self,
        name: &str,
    ) -> Option<u64> {
        let mut levels = usize::from(!self.principal_bindings.is_empty());
        let mut minimum_keys = 1usize;
        while let Some(next) = minimum_keys.checked_mul(6).and_then(|n| n.checked_add(5)) {
            if next > self.principal_bindings.len() {
                break;
            }
            levels = levels.checked_add(1)?;
            minimum_keys = next;
        }
        // One lookup cannot visit more distinct keys than this installed map
        // contains, regardless of the conservative tree-height estimate.
        let visits = levels.checked_mul(11)?.min(self.principal_bindings.len());
        let work = visits
            .checked_mul(name.len().checked_add(1)?)?
            .checked_add(1)?;
        u64::try_from(work).ok()
    }
}

#[derive(Clone, Debug)]
pub(in crate::domain_computation) struct WorthQueryPrimaryPrincipalBindingLayout {
    pub(in crate::domain_computation::primary_graph) mapping_kind: KindId,
    pub(in crate::domain_computation::primary_graph) identity_locator: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) status_locator: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) relation_kind: KindId,
    pub(in crate::domain_computation::primary_graph) principal_kind: KindId,
    pub(in crate::domain_computation::primary_graph) principal_identity_locator: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) index_id: DerivedIndexId,
}

pub(super) fn lower_principal_bindings(
    schema: &ErasedApplicationSchemaDeclaration,
    entity_kinds: &BTreeMap<String, KindId>,
    relation_kinds: &BTreeMap<String, KindId>,
) -> Result<
    BTreeMap<String, WorthQueryPrimaryPrincipalBindingLayout>,
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let mut bindings = BTreeMap::new();
    for member in schema.members() {
        let ApplicationSchemaMember::PrincipalBinding {
            binding,
            mapping_entity,
            identity_aspect,
            identity_field,
            status_aspect,
            status_field,
            target_relation,
            principal_entity,
            principal_identity_aspect,
            principal_identity_field,
            ..
        } = member
        else {
            continue;
        };
        let index_ordinal =
            u64::try_from(bindings.len() + 1).map_err(|_| contract_space_exhausted())?;
        bindings.insert(
            binding.clone(),
            WorthQueryPrimaryPrincipalBindingLayout {
                mapping_kind: required_kind(entity_kinds, mapping_entity)?,
                identity_locator: planned_field_locator(identity_aspect, identity_field)?,
                status_locator: planned_field_locator(status_aspect, status_field)?,
                relation_kind: required_kind(relation_kinds, target_relation)?,
                principal_kind: required_kind(entity_kinds, principal_entity)?,
                principal_identity_locator: planned_field_locator(
                    principal_identity_aspect,
                    principal_identity_field,
                )?,
                index_id: DerivedIndexId(index_ordinal),
            },
        );
    }
    Ok(bindings)
}
