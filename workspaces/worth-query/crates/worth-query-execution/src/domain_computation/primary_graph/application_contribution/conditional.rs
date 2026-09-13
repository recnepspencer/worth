use std::any::{Any, TypeId};
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryPortableApplicationConditionalOperationBinding,
    WorthQueryPortableDomainOperationDefinition,
};

use super::super::conditional_operation::{
    WorthQueryConditionalApplicationRuntimeInstallation,
    WorthQueryConditionalRuntimeInstallationDenial,
};
use super::super::{
    WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind as DenialKind,
};
use super::producer::WorthQueryInstalledApplicationProducerRegistry;

mod producer_access;
pub use producer_access::WorthQueryApplicationConditionalProducerAccess;
mod dependency;
pub(super) use dependency::validate_required_producer;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationConditionalPackageContract {
    domain_operation: WorthQueryPortableDomainOperationDefinition,
    operation_binding: WorthQueryPortableApplicationConditionalOperationBinding,
    node_identity: String,
}

impl WorthQueryApplicationConditionalPackageContract {
    pub fn new(
        domain_operation: WorthQueryPortableDomainOperationDefinition,
        operation_binding: WorthQueryPortableApplicationConditionalOperationBinding,
        node_identity: impl Into<String>,
    ) -> Self {
        Self {
            domain_operation,
            operation_binding,
            node_identity: node_identity.into(),
        }
    }

    pub(super) fn domain_operation(&self) -> WorthQueryPortableDomainOperationDefinition {
        self.domain_operation.clone()
    }

    pub(super) fn operation_binding(
        &self,
    ) -> WorthQueryPortableApplicationConditionalOperationBinding {
        self.operation_binding.clone()
    }

    pub(super) fn node_identity(&self) -> &str {
        &self.node_identity
    }
}

pub trait WorthQueryApplicationConditionalBinding<Schema>: Sized + 'static
where
    Schema: ApplicationSchema + 'static,
{
    type Configuration: 'static;
    type Installed: Send + Sync + 'static;

    const IDENTITY: &'static str;
    const REQUIRED_PRODUCERS: &'static [&'static str];

    fn package_contract() -> WorthQueryApplicationConditionalPackageContract;

    fn install(
        configuration: Self::Configuration,
        producers: &WorthQueryApplicationConditionalProducerAccess<'_, Schema>,
        installation: &mut WorthQueryConditionalApplicationRuntimeInstallation<Schema>,
    ) -> Result<Self::Installed, WorthQueryConditionalRuntimeInstallationDenial>;
}

pub(super) struct DeclaredConditionalBinding {
    pub(super) owner: String,
    pub(super) contract: WorthQueryApplicationConditionalPackageContract,
    pub(super) required_producers: Vec<String>,
    pub(super) binding_type: TypeId,
    pub(super) installed_type: TypeId,
}

struct InstalledConditionalBinding {
    binding_type: TypeId,
    installed_type: TypeId,
    value: Arc<dyn Any + Send + Sync>,
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

impl<Schema> WorthQueryInstalledApplicationConditionalRegistry<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub fn binding<Binding>(&self) -> Option<Arc<Binding::Installed>>
    where
        Binding: WorthQueryApplicationConditionalBinding<Schema>,
    {
        self.entries
            .get(Binding::IDENTITY)
            .filter(|entry| {
                entry.binding_type == TypeId::of::<Binding>()
                    && entry.installed_type == TypeId::of::<Binding::Installed>()
            })
            .and_then(|entry| {
                Arc::clone(&entry.value)
                    .downcast::<Binding::Installed>()
                    .ok()
            })
    }
}

trait PendingConditionalConfiguration<Schema>: 'static
where
    Schema: ApplicationSchema + 'static,
{
    fn install(
        self: Box<Self>,
        producers: &WorthQueryInstalledApplicationProducerRegistry<Schema>,
        declaration: &DeclaredConditionalBinding,
        installation: &mut WorthQueryConditionalApplicationRuntimeInstallation<Schema>,
    ) -> Result<InstalledConditionalBinding, WorthQueryConditionalRuntimeInstallationDenial>;
}

struct TypedConditionalConfiguration<Schema, Binding>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationConditionalBinding<Schema>,
{
    configuration: Binding::Configuration,
    marker: PhantomData<fn() -> (Schema, Binding)>,
}

impl<Schema, Binding> PendingConditionalConfiguration<Schema>
    for TypedConditionalConfiguration<Schema, Binding>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationConditionalBinding<Schema>,
{
    fn install(
        self: Box<Self>,
        producers: &WorthQueryInstalledApplicationProducerRegistry<Schema>,
        declaration: &DeclaredConditionalBinding,
        installation: &mut WorthQueryConditionalApplicationRuntimeInstallation<Schema>,
    ) -> Result<InstalledConditionalBinding, WorthQueryConditionalRuntimeInstallationDenial> {
        let access = WorthQueryApplicationConditionalProducerAccess::new(
            producers,
            &declaration.required_producers,
        );
        installation.begin_application_binding_scope(
            declaration.contract.operation_binding(),
            declaration.contract.node_identity().to_owned(),
        );
        let installed = match Binding::install(self.configuration, &access, installation) {
            Ok(installed) => installed,
            Err(denial) => {
                installation.abandon_application_binding_scope();
                return Err(denial);
            }
        };
        installation.finish_application_binding_scope()?;
        Ok(InstalledConditionalBinding {
            binding_type: TypeId::of::<Binding>(),
            installed_type: TypeId::of::<Binding::Installed>(),
            value: Arc::new(installed),
        })
    }
}

pub(in crate::domain_computation::primary_graph) struct PendingConditionalRegistry<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    declared: BTreeMap<String, DeclaredConditionalBinding>,
    configured: BTreeMap<String, Box<dyn PendingConditionalConfiguration<Schema>>>,
}

impl<Schema> PendingConditionalRegistry<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(super) fn new(declared: BTreeMap<String, DeclaredConditionalBinding>) -> Self {
        Self {
            declared,
            configured: BTreeMap::new(),
        }
    }

    pub(super) fn register<Binding>(
        &mut self,
        owner: &str,
        configuration: Binding::Configuration,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        Binding: WorthQueryApplicationConditionalBinding<Schema>,
    {
        let declared = self
            .declared
            .get(Binding::IDENTITY)
            .ok_or_else(|| denial(DenialKind::MissingConditionalBinding, Binding::IDENTITY))?;
        if declared.owner != owner
            || declared.contract != Binding::package_contract()
            || declared.binding_type != TypeId::of::<Binding>()
            || declared.installed_type != TypeId::of::<Binding::Installed>()
            || declared.required_producers
                != Binding::REQUIRED_PRODUCERS
                    .iter()
                    .map(|identity| (*identity).to_owned())
                    .collect::<Vec<_>>()
        {
            return Err(denial(
                DenialKind::ContributionMemberMismatch,
                Binding::IDENTITY,
            ));
        }
        if self.configured.contains_key(Binding::IDENTITY) {
            return Err(denial(
                DenialKind::DuplicateConditionalBinding,
                Binding::IDENTITY,
            ));
        }
        self.configured.insert(
            Binding::IDENTITY.to_owned(),
            Box::new(TypedConditionalConfiguration::<Schema, Binding> {
                configuration,
                marker: PhantomData,
            }),
        );
        Ok(())
    }

    pub(super) fn validate_complete(&self) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        if let Some(identity) = self
            .declared
            .keys()
            .find(|identity| !self.configured.contains_key(*identity))
        {
            Err(denial(DenialKind::MissingConditionalBinding, identity))
        } else {
            Ok(())
        }
    }

    pub(in crate::domain_computation::primary_graph) fn is_empty(&self) -> bool {
        self.configured.is_empty()
    }

    pub(in crate::domain_computation::primary_graph) fn install_all(
        self,
        producers: &WorthQueryInstalledApplicationProducerRegistry<Schema>,
        installation: &mut WorthQueryConditionalApplicationRuntimeInstallation<Schema>,
    ) -> Result<
        WorthQueryInstalledApplicationConditionalRegistry<Schema>,
        WorthQueryConditionalRuntimeInstallationDenial,
    > {
        let mut installed = BTreeMap::new();
        for (identity, configuration) in self.configured {
            let declaration = self
                .declared
                .get(&identity)
                .expect("configured conditional was resolved from the declaration");
            installed.insert(
                identity,
                configuration.install(producers, declaration, installation)?,
            );
        }
        Ok(WorthQueryInstalledApplicationConditionalRegistry {
            entries: Arc::new(installed),
            marker: PhantomData,
        })
    }
}

fn denial(
    kind: DenialKind,
    subject: impl Into<String>,
) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(kind, subject)
}
