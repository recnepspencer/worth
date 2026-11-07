//! Compiler-issued physical roles are DATA, independent of Source issuers.
use super::*;
use worth_foundational::facade::{AggregateDefinitionIdentity, AspectIdentity};
use worth_relational::facade::schema::{AggregateIdentityStorage, SchemaAuthoritySnapshot};
mod allocation;
mod predecessor;
mod source_application;
pub(super) use allocation::PhysicalIdentityAllocation;
pub(super) use predecessor::private_entity_role;

#[derive(Debug, Clone)]
pub(in crate::domain_computation::primary_graph) struct PhysicalIdentityBindings {
    pub(super) entities: BTreeMap<String, KindId>,
    pub(super) relations: BTreeMap<String, KindId>,
    pub(super) entity_ids: BTreeSet<KindId>,
    pub(super) relation_ids: BTreeSet<KindId>,
    platform_first: KindId,
    platform_aspects: [AspectIdentity; 18],
    aggregate_storage: Option<AggregateIdentityStorage>,
    memberships: BTreeMap<AggregateDefinitionIdentity, KindId>,
    bootstrap: (KindId, AspectIdentity),
    maximum_kind: Option<KindId>,
    maximum_aspect: Option<AspectIdentity>,
}
impl PhysicalIdentityBindings {
    pub(super) fn from_compiled(
        registry: &RelationalSchemaRegistry,
        entities: BTreeMap<String, KindId>,
        relations: BTreeMap<String, KindId>,
        maximum_kind: Option<KindId>,
        maximum_aspect: Option<AspectIdentity>,
    ) -> Result<Self, WorthQueryPrimaryGraphInstallationDenial> {
        let snapshot = registry.authority_snapshot();
        Self::from_snapshot(
            registry,
            &snapshot,
            entities,
            relations,
            maximum_kind,
            maximum_aspect,
        )
    }
    fn from_snapshot(
        registry: &RelationalSchemaRegistry,
        snapshot: &SchemaAuthoritySnapshot,
        entities: BTreeMap<String, KindId>,
        relations: BTreeMap<String, KindId>,
        maximum_kind: Option<KindId>,
        maximum_aspect: Option<AspectIdentity>,
    ) -> Result<Self, WorthQueryPrimaryGraphInstallationDenial> {
        let entity_ids = entities.values().copied().collect();
        let relation_ids = relations.values().copied().collect();
        let (platform_first, platform_aspects) =
            super::platform_identity_allocator::readmit_roles(registry, snapshot, &entity_ids)?;
        let (aggregate_storage, memberships) =
            super::aggregate_identity::readmit_physical_roles(registry, snapshot)?;
        let bootstrap =
            super::bootstrap_publication::readmit_role(registry, snapshot, &entity_ids)?;
        let mut result = Self {
            entities,
            relations,
            entity_ids,
            relation_ids,
            platform_first,
            platform_aspects,
            aggregate_storage,
            memberships,
            bootstrap,
            maximum_kind,
            maximum_aspect,
        };
        result.reserve_snapshot(registry, snapshot);
        Ok(result)
    }
    fn reserve_snapshot(
        &mut self,
        registry: &RelationalSchemaRegistry,
        snapshot: &SchemaAuthoritySnapshot,
    ) {
        for kind in &snapshot.entity_kinds {
            self.maximum_kind = self.maximum_kind.max(Some(kind.kind_id));
            let entry = registry
                .entity_registration(kind.kind_id)
                .expect("snapshot entry");
            for aspect in &entry.aspect_contract_declarations.aspects {
                self.maximum_aspect = self.maximum_aspect.max(Some(aspect.contract.identity()));
            }
        }
        for kind in &snapshot.relation_kinds {
            self.maximum_kind = self.maximum_kind.max(Some(kind.kind_id));
            let entry = registry
                .relation_registration(kind.kind_id)
                .expect("snapshot entry");
            for aspect in &entry.aspect_contract_declarations.aspects {
                self.maximum_aspect = self.maximum_aspect.max(Some(aspect.contract.identity()));
            }
        }
    }
    fn platform_identity(&self, identity: AspectIdentity) -> bool {
        self.platform_aspects.contains(&identity)
            || identity == self.bootstrap.1
            || self.aggregate_storage.is_some_and(|storage| {
                identity == storage.aggregate_aspect
                    || identity == storage.lineage_aspect
                    || storage.projected.is_some_and(|projected| {
                        identity == projected.coordinate_aspect
                            || identity == projected.membership_aspect
                    })
            })
    }
    pub(super) fn require_authored_identities(
        &self,
        identities: impl Iterator<Item = AspectIdentity>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        if identities
            .into_iter()
            .any(|identity| self.platform_identity(identity))
        {
            return Err(invalid_member(
                "authored aspect identity collides with a retained platform role",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod value_layout;
