use crate::equality_type_id::EqualityTypeId as TypeId;
use std::any::Any;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchema;

pub mod dependency;
pub mod producer_access;
pub use dependency::validate_required_producer;
pub use producer_access::WorthQueryApplicationConditionalProducerAccess;
pub use worth_query_execution_lineage::facade::domain_computation::primary_graph::application_contribution::conditional::output_readiness_contract;
pub use output_readiness_contract::{
    WorthQueryOutputReadinessContract, WorthQueryOutputReadinessContractBuilder,
    WorthQueryOutputReadinessContractDenial, WorthQueryReadinessFieldUse,
};

use worth_query_execution_lineage::facade::domain_computation::primary_graph::application_contribution::conditional::package_contract;
pub use package_contract::WorthQueryApplicationConditionalPackageContract;

pub struct DeclaredConditionalBinding {
    pub owner: String,
    pub contract: WorthQueryApplicationConditionalPackageContract,
    pub required_producers: Vec<String>,
    pub binding_type: TypeId,
    pub operation_type: TypeId,
    pub installed_type: TypeId,
}

pub struct InstalledConditionalBinding {
    pub binding_type: TypeId,
    pub installed_type: TypeId,
    pub operation_type: TypeId,
    pub value: Arc<dyn Any + Send + Sync>,
}

#[derive(Clone)]
pub struct WorthQueryInstalledApplicationConditionalRegistry<Schema> {
    entries: Arc<BTreeMap<String, InstalledConditionalBinding>>,
    marker: PhantomData<fn() -> Schema>,
}

impl<Schema> Default for WorthQueryInstalledApplicationConditionalRegistry<Schema> {
    fn default() -> Self {
        Self {
            entries: Arc::new(BTreeMap::new()),
            marker: PhantomData,
        }
    }
}

impl<Schema> WorthQueryInstalledApplicationConditionalRegistry<Schema> {
    pub fn install_declared_entries<Config, Err>(
        declared: &BTreeMap<String, DeclaredConditionalBinding>,
        mut configured: BTreeMap<String, Config>,
        mut install_one: impl FnMut(
            &DeclaredConditionalBinding,
            Config,
        ) -> Result<InstalledConditionalBinding, Err>,
    ) -> Result<
        Self,
        (
            BTreeMap<String, Config>,
            BTreeMap<String, InstalledConditionalBinding>,
            Err,
        ),
    > {
        let mut completed = BTreeMap::new();
        while let Some((identity, configuration)) = configured.pop_first() {
            let declaration = declared
                .get(&identity)
                .expect("configured identity came from the declaration");
            match install_one(declaration, configuration) {
                Ok(installed) => {
                    completed.insert(identity, installed);
                }
                Err(err) => return Err((configured, completed, err)),
            }
        }
        Ok(Self {
            entries: Arc::new(completed),
            marker: PhantomData,
        })
    }

    pub fn entry_value(
        &self,
        identity: &str,
        binding_type: TypeId,
        installed_type: TypeId,
    ) -> Option<Arc<dyn Any + Send + Sync>> {
        self.entries
            .get(identity)
            .filter(|entry| {
                entry.binding_type == binding_type && entry.installed_type == installed_type
            })
            .map(|entry| Arc::clone(&entry.value))
    }

    pub(crate) fn entry_value_by_type(
        &self,
        binding_type: TypeId,
        installed_type: TypeId,
    ) -> Option<Arc<dyn Any + Send + Sync>> {
        self.entries
            .values()
            .find(|entry| {
                entry.binding_type == binding_type && entry.installed_type == installed_type
            })
            .map(|entry| Arc::clone(&entry.value))
    }

    /// Nonempty definitions retire only through their actual readmitted owner.
    /// Its installation admitted authored definitions without temporal bindings.
    pub fn preflight_home_retirement(
        &self,
        owner: Option<&super::super::managed_bridge::conditional_owner::ConditionalOwner>,
    ) -> Result<(), super::super::WorthQueryHomeRetirementDenial> {
        use super::super::managed_bridge::conditional_owner::ConditionalOwner;
        use super::super::{
            WorthQueryHomeRetirementDenial as Denial, WorthQueryHomeRetirementOwner as Owner,
        };
        if self.entries.is_empty() {
            return Ok(());
        }
        match owner {
            Some(ConditionalOwner::Authored(_)) => Ok(()),
            Some(ConditionalOwner::Memory(_)) | Some(ConditionalOwner::Stopped) | None => {
                Err(Denial::Unsupported(Owner::InstalledConditionals))
            }
        }
    }
}

impl<Schema> WorthQueryInstalledApplicationConditionalRegistry<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub fn operation_types(&self) -> impl Iterator<Item = TypeId> + '_ {
        self.entries.values().map(|entry| entry.operation_type)
    }

    pub fn binding_count(&self) -> usize {
        self.entries.len()
    }

    pub fn contains_operation<Operation: 'static>(&self) -> bool {
        self.operation_types()
            .any(|operation| operation == TypeId::of::<Operation>())
    }
}

mod court_doors_for_test;
