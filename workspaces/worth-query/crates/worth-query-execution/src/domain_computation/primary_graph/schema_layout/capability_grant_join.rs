use std::collections::BTreeMap;

use worth_query_declaration::facade::application_capability::ErasedApplicationCapabilityContract;
use worth_query_installation::facade::{
    ApplicationSchemaMember, ErasedApplicationSchemaDeclaration,
};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::indexes::{
    DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind, RelationJoinDefinition,
    RelationJoinLeg, RelationJoinSharedEndpoint,
};

use super::{
    invalid_member, required_kind, WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryRelationLayout,
};

#[derive(Debug)]
pub(super) struct WorthQueryCapabilityGrantJoinLayout {
    grantee_relation: String,
    resource_relation: String,
    left_relation_kind: KindId,
    left_entity_kind: KindId,
    right_relation_kind: KindId,
    right_entity_kind: KindId,
    shared_entity_kind: KindId,
    index_id: DerivedIndexId,
}

impl WorthQueryCapabilityGrantJoinLayout {
    fn lower(
        contract: &ErasedApplicationCapabilityContract,
        entity_kinds: &BTreeMap<String, KindId>,
        relations: &BTreeMap<String, WorthQueryPrimaryRelationLayout>,
    ) -> Result<Self, WorthQueryPrimaryGraphInstallationDenial> {
        let grantee = contract.delegation().grantee();
        let resource = contract.target().resource();
        if grantee.to() != contract.grant_entity() || resource.from() != contract.grant_entity() {
            return Err(invalid_member(contract.name()));
        }
        let grantee_layout = required_relation(relations, grantee.relation())?;
        let resource_layout = required_relation(relations, resource.relation())?;
        let left_entity_kind = required_kind(entity_kinds, grantee.from())?;
        let shared_entity_kind = required_kind(entity_kinds, contract.grant_entity())?;
        let right_entity_kind = required_kind(entity_kinds, resource.to())?;
        if grantee_layout.from != left_entity_kind
            || grantee_layout.to != shared_entity_kind
            || resource_layout.from != shared_entity_kind
            || resource_layout.to != right_entity_kind
        {
            return Err(invalid_member(contract.name()));
        }
        Ok(Self {
            grantee_relation: grantee.relation().to_string(),
            resource_relation: resource.relation().to_string(),
            left_relation_kind: grantee_layout.kind,
            left_entity_kind,
            right_relation_kind: resource_layout.kind,
            right_entity_kind,
            shared_entity_kind,
            index_id: DerivedIndexId(0),
        })
    }

    pub(super) fn definition(&self, ordinal: usize) -> DerivedIndexDefinition {
        DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: format!("application-capability.grant-join.{ordinal}"),
            kind: DerivedIndexKind::RelationJoin(RelationJoinDefinition::new(
                RelationJoinLeg::new(
                    self.left_relation_kind,
                    RelationJoinSharedEndpoint::Target,
                    self.left_entity_kind,
                ),
                RelationJoinLeg::new(
                    self.right_relation_kind,
                    RelationJoinSharedEndpoint::Source,
                    self.right_entity_kind,
                ),
                self.shared_entity_kind,
            )),
            branch_scoped: false,
        }
    }

    pub(super) fn bind_index(&mut self, index_id: DerivedIndexId) {
        self.index_id = index_id;
    }

    pub(super) const fn index_id(&self) -> DerivedIndexId {
        self.index_id
    }
}

pub(super) fn lower_capability_grant_joins(
    schema: &ErasedApplicationSchemaDeclaration,
    entity_kinds: &BTreeMap<String, KindId>,
    relations: &BTreeMap<String, WorthQueryPrimaryRelationLayout>,
) -> Result<
    BTreeMap<(String, String), WorthQueryCapabilityGrantJoinLayout>,
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let mut joins = BTreeMap::new();
    for contract in schema.members().iter().filter_map(|member| match member {
        ApplicationSchemaMember::ApplicationCapability { contract } => Some(contract),
        _ => None,
    }) {
        let join = WorthQueryCapabilityGrantJoinLayout::lower(contract, entity_kinds, relations)?;
        let key = (
            join.grantee_relation.clone(),
            join.resource_relation.clone(),
        );
        joins.entry(key).or_insert(join);
    }
    Ok(joins)
}

fn required_relation<'a>(
    relations: &'a BTreeMap<String, WorthQueryPrimaryRelationLayout>,
    name: &str,
) -> Result<&'a WorthQueryPrimaryRelationLayout, WorthQueryPrimaryGraphInstallationDenial> {
    relations.get(name).ok_or_else(|| invalid_member(name))
}

impl super::WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation::primary_graph) fn register_capability_grant_joins<E>(
        &mut self,
        mut register: impl FnMut(DerivedIndexDefinition) -> Result<DerivedIndexId, E>,
    ) -> Result<(), E> {
        for (ordinal, join) in self.capability_grant_joins.values_mut().enumerate() {
            let index_id = register(join.definition(ordinal))?;
            join.bind_index(index_id);
        }
        Ok(())
    }

    pub(in crate::domain_computation) fn capability_grant_join_index_id(
        &self,
        grantee_relation: &str,
        resource_relation: &str,
    ) -> Option<DerivedIndexId> {
        self.capability_grant_joins
            .get(&(grantee_relation.to_string(), resource_relation.to_string()))
            .map(WorthQueryCapabilityGrantJoinLayout::index_id)
    }

    pub(in crate::domain_computation::primary_graph) fn capability_grant_join_index_ids(
        &self,
    ) -> impl Iterator<Item = DerivedIndexId> + '_ {
        self.capability_grant_joins
            .values()
            .map(WorthQueryCapabilityGrantJoinLayout::index_id)
    }
}
