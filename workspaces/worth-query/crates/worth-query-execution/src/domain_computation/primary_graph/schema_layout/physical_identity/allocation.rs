//! One attempt-local allocator reserves every original physical identity.
use super::*;
use worth_relational::facade::schema::{AggregateIdentityStorage, AggregateProjectedMemberStorage};

pub(in crate::domain_computation::primary_graph::schema_layout) struct PhysicalIdentityAllocation<
    'source,
> {
    original: Option<&'source PhysicalIdentityBindings>,
    maximum_kind: Option<KindId>,
    maximum_aspect: Option<AspectIdentity>,
}
impl<'source> PhysicalIdentityAllocation<'source> {
    pub(in crate::domain_computation::primary_graph::schema_layout) fn new(
        original: Option<&'source PhysicalIdentityBindings>,
        receiver: &RelationalSchemaRegistry,
        contracts: &WorthQueryInstalledApplicationSchemaContractCatalog,
    ) -> Result<Self, WorthQueryPrimaryGraphInstallationDenial> {
        Self::for_identities(
            original,
            receiver,
            contracts.maximum_aspect_identity(),
            contracts
                .contracts()
                .map(|entry| entry.contract().identity()),
        )
    }
    pub(in crate::domain_computation::primary_graph::schema_layout) fn for_identities(
        original: Option<&'source PhysicalIdentityBindings>,
        receiver: &RelationalSchemaRegistry,
        maximum_authored: Option<AspectIdentity>,
        authored: impl Iterator<Item = AspectIdentity>,
    ) -> Result<Self, WorthQueryPrimaryGraphInstallationDenial> {
        if let Some(original) = original {
            original.require_authored_identities(authored)?;
        }
        let mut result = Self {
            original,
            maximum_kind: original.and_then(|roles| roles.maximum_kind),
            maximum_aspect: original
                .and_then(|roles| roles.maximum_aspect)
                .max(maximum_authored),
        };
        result.reserve_registry(receiver);
        Ok(result)
    }
    pub(in crate::domain_computation::primary_graph::schema_layout) fn original(
        &self,
    ) -> Option<&PhysicalIdentityBindings> {
        self.original
    }
    pub(in crate::domain_computation::primary_graph::schema_layout) fn reserve_registry(
        &mut self,
        registry: &RelationalSchemaRegistry,
    ) {
        let snapshot = registry.authority_snapshot();
        for kind in snapshot
            .entity_kinds
            .iter()
            .map(|kind| kind.kind_id)
            .chain(snapshot.relation_kinds.iter().map(|kind| kind.kind_id))
        {
            self.maximum_kind = self.maximum_kind.max(Some(kind));
            let declarations = match registry.entity_registration(kind) {
                Ok(entry) => &entry.aspect_contract_declarations,
                Err(_) => {
                    &registry
                        .relation_registration(kind)
                        .expect("snapshot registration")
                        .aspect_contract_declarations
                }
            };
            for binding in &declarations.aspects {
                self.maximum_aspect = self.maximum_aspect.max(Some(binding.contract.identity()));
            }
        }
    }
    fn next_kind(&mut self) -> Result<KindId, WorthQueryPrimaryGraphInstallationDenial> {
        let kind = KindId(
            self.maximum_kind
                .map_or(0, |kind| kind.0)
                .checked_add(1)
                .ok_or_else(kind_space_exhausted)?,
        );
        self.maximum_kind = Some(kind);
        Ok(kind)
    }
    fn next_aspect(&mut self) -> Result<AspectIdentity, WorthQueryPrimaryGraphInstallationDenial> {
        let identity = AspectIdentity(
            self.maximum_aspect
                .map_or(0, |identity| identity.0)
                .checked_add(1)
                .ok_or_else(contract_space_exhausted)?,
        );
        self.maximum_aspect = Some(identity);
        Ok(identity)
    }
    pub(in crate::domain_computation::primary_graph::schema_layout) fn entity(
        &mut self,
        name: &str,
    ) -> Result<KindId, WorthQueryPrimaryGraphInstallationDenial> {
        match self
            .original
            .and_then(|roles| roles.entities.get(name))
            .copied()
        {
            Some(kind) => Ok(kind),
            None => self.next_kind(),
        }
    }
    pub(in crate::domain_computation::primary_graph::schema_layout) fn relation(
        &mut self,
        name: &str,
    ) -> Result<KindId, WorthQueryPrimaryGraphInstallationDenial> {
        match self
            .original
            .and_then(|roles| roles.relations.get(name))
            .copied()
        {
            Some(kind) => Ok(kind),
            None => self.next_kind(),
        }
    }
    pub(in crate::domain_computation::primary_graph::schema_layout) fn platform_first(
        &mut self,
        fresh: impl FnOnce() -> Result<KindId, WorthQueryPrimaryGraphInstallationDenial>,
    ) -> Result<KindId, WorthQueryPrimaryGraphInstallationDenial> {
        match self.original {
            Some(roles) => Ok(roles.platform_first),
            None => fresh(),
        }
    }
    pub(in crate::domain_computation::primary_graph::schema_layout) fn platform_aspects(
        &self,
    ) -> Option<[AspectIdentity; 18]> {
        self.original.map(|roles| roles.platform_aspects)
    }
    pub(in crate::domain_computation::primary_graph::schema_layout) fn aggregate_storage(
        &mut self,
        fresh: impl FnOnce()
            -> Result<AggregateIdentityStorage, WorthQueryPrimaryGraphInstallationDenial>,
        projected: bool,
    ) -> Result<AggregateIdentityStorage, WorthQueryPrimaryGraphInstallationDenial> {
        let Some(original) = self.original else {
            return fresh();
        };
        let mut storage = match original.aggregate_storage {
            Some(storage) => storage,
            None => AggregateIdentityStorage {
                aggregate_kind: self.next_kind()?,
                lineage_kind: self.next_kind()?,
                aggregate_aspect: self.next_aspect()?,
                lineage_aspect: self.next_aspect()?,
                projected: None,
            },
        };
        if projected && storage.projected.is_none() {
            storage.projected = Some(AggregateProjectedMemberStorage {
                coordinate_aspect: self.next_aspect()?,
                membership_aspect: self.next_aspect()?,
            });
        }
        // Disabled projected grammar is not installed merely to preserve a slot.
        if !projected {
            storage.projected = None;
        }
        Ok(storage)
    }
    pub(in crate::domain_computation::primary_graph::schema_layout) fn membership(
        &mut self,
        definition: AggregateDefinitionIdentity,
        fresh: impl FnOnce() -> Result<KindId, WorthQueryPrimaryGraphInstallationDenial>,
    ) -> Result<KindId, WorthQueryPrimaryGraphInstallationDenial> {
        match self.original {
            Some(roles) => match roles.memberships.get(&definition) {
                Some(kind) => Ok(*kind),
                None => self.next_kind(),
            },
            None => fresh(),
        }
    }
    pub(in crate::domain_computation::primary_graph::schema_layout) fn bootstrap(
        &mut self,
        fresh: impl FnOnce()
            -> Result<(KindId, AspectIdentity), WorthQueryPrimaryGraphInstallationDenial>,
    ) -> Result<(KindId, AspectIdentity), WorthQueryPrimaryGraphInstallationDenial> {
        match self.original {
            Some(roles) => Ok(roles.bootstrap),
            None => fresh(),
        }
    }
    pub(in crate::domain_computation::primary_graph::schema_layout) fn finish(
        &self,
        registry: &RelationalSchemaRegistry,
        entities: BTreeMap<String, KindId>,
        relations: BTreeMap<String, KindId>,
    ) -> Result<PhysicalIdentityBindings, WorthQueryPrimaryGraphInstallationDenial> {
        PhysicalIdentityBindings::from_compiled(
            registry,
            entities,
            relations,
            self.maximum_kind,
            self.maximum_aspect,
        )
    }
}
