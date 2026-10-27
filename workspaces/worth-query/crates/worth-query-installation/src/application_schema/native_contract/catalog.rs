use std::collections::BTreeMap;

use worth_foundational::facade::AspectIdentity;
use worth_query_declaration::facade::application_schema::ApplicationFieldBindingOwnerKind;

use crate::canonical_work::WorthQueryCanonicalWorkEvidence;

use super::WorthQueryInstalledApplicationAspectContract;

pub(super) type NativeContractsByOwner = BTreeMap<
    ApplicationFieldBindingOwnerKind,
    BTreeMap<String, BTreeMap<String, WorthQueryInstalledApplicationAspectContract>>,
>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryInstalledApplicationSchemaContractCatalogCounters {
    catalogs_compiled: usize,
    contracts_compiled: usize,
    fields_compiled: usize,
    canonical_contract_bases_prepared: usize,
}

impl WorthQueryInstalledApplicationSchemaContractCatalogCounters {
    pub(crate) const fn compiled(contracts: usize, fields: usize) -> Self {
        Self {
            catalogs_compiled: 1,
            contracts_compiled: contracts,
            fields_compiled: fields,
            canonical_contract_bases_prepared: contracts,
        }
    }

    pub const fn catalogs_compiled(self) -> usize {
        self.catalogs_compiled
    }

    pub const fn contracts_compiled(self) -> usize {
        self.contracts_compiled
    }

    pub const fn fields_compiled(self) -> usize {
        self.fields_compiled
    }

    pub const fn canonical_contract_bases_prepared(self) -> usize {
        self.canonical_contract_bases_prepared
    }
}

/// Sealed installed native contracts for one exact application schema binding.
#[derive(Clone, Debug)]
pub struct WorthQueryInstalledApplicationSchemaContractCatalog {
    contracts: NativeContractsByOwner,
    maximum_aspect_identity: Option<AspectIdentity>,
    counters: WorthQueryInstalledApplicationSchemaContractCatalogCounters,
    canonical_work: WorthQueryCanonicalWorkEvidence,
}

impl WorthQueryInstalledApplicationSchemaContractCatalog {
    pub(crate) fn new(
        contracts: NativeContractsByOwner,
        maximum_aspect_identity: Option<AspectIdentity>,
        counters: WorthQueryInstalledApplicationSchemaContractCatalogCounters,
        canonical_work: WorthQueryCanonicalWorkEvidence,
    ) -> Self {
        Self {
            contracts,
            maximum_aspect_identity,
            counters,
            canonical_work,
        }
    }

    pub fn aspect(
        &self,
        entity: &str,
        aspect: &str,
    ) -> Option<&WorthQueryInstalledApplicationAspectContract> {
        self.aspect_for_owner(ApplicationFieldBindingOwnerKind::Entity, entity, aspect)
    }

    pub fn relation_aspect(
        &self,
        relation: &str,
        aspect: &str,
    ) -> Option<&WorthQueryInstalledApplicationAspectContract> {
        self.aspect_for_owner(ApplicationFieldBindingOwnerKind::Relation, relation, aspect)
    }

    pub fn aspect_for_owner(
        &self,
        owner: ApplicationFieldBindingOwnerKind,
        name: &str,
        aspect: &str,
    ) -> Option<&WorthQueryInstalledApplicationAspectContract> {
        self.contracts
            .get(&owner)
            .and_then(|owners| owners.get(name))
            .and_then(|aspects| aspects.get(aspect))
    }

    pub fn contracts(&self) -> impl Iterator<Item = &WorthQueryInstalledApplicationAspectContract> {
        self.contracts
            .values()
            .flat_map(|owners| owners.values())
            .flat_map(|aspects| aspects.values())
    }

    pub fn len(&self) -> usize {
        self.contracts
            .values()
            .flat_map(|owners| owners.values())
            .map(BTreeMap::len)
            .sum()
    }

    pub fn is_empty(&self) -> bool {
        self.contracts.is_empty()
    }

    pub fn maximum_aspect_identity(&self) -> Option<AspectIdentity> {
        self.maximum_aspect_identity
    }

    pub const fn counters(&self) -> WorthQueryInstalledApplicationSchemaContractCatalogCounters {
        self.counters
    }

    pub(crate) const fn canonical_work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.canonical_work
    }
}
