use crate::equality_type_id::EqualityTypeId as TypeId;
use std::any::Any;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryConditionalNodeLocation,
    WorthQueryPortableApplicationConditionalOperationBinding,
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
mod target_coverage;
pub(super) use dependency::validate_required_producer;
mod output_readiness_contract;
pub use output_readiness_contract::{
    WorthQueryOutputReadinessContractBuilder, WorthQueryOutputReadinessContractDenial,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationConditionalPackageContract {
    domain_operation: WorthQueryPortableDomainOperationDefinition,
    operation_binding: WorthQueryPortableApplicationConditionalOperationBinding,
    target: WorthQueryConditionalNodeLocation,
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
            target: WorthQueryConditionalNodeLocation::Operation {
                node_identity: node_identity.into(),
            },
        }
    }

    /// Authors a workflow-stage location for this declaration's node. This is
    /// selection only; complete installed coverage and native binding validate it.
    pub fn in_workflow_stage(mut self, stage_identity: impl Into<String>) -> Self {
        self.target = WorthQueryConditionalNodeLocation::WorkflowStage {
            stage_identity: stage_identity.into(),
            node_identity: self.target.node_identity().to_owned(),
        };
        self
    }

    pub(super) fn domain_operation(&self) -> &WorthQueryPortableDomainOperationDefinition {
        &self.domain_operation
    }

    pub(super) fn operation_binding(
        &self,
    ) -> &WorthQueryPortableApplicationConditionalOperationBinding {
        &self.operation_binding
    }

    pub(super) fn node_identity(&self) -> &str {
        self.target.node_identity()
    }

    pub(super) fn target(&self) -> &WorthQueryConditionalNodeLocation {
        &self.target
    }
}

pub trait WorthQueryApplicationConditionalBinding<Schema>: Sized + 'static
where
    Schema: ApplicationSchema + 'static,
{
    type Configuration: 'static;
    type Installed: Send + Sync + 'static;
    type Operation: 'static;

    const IDENTITY: &'static str;
    const REQUIRED_PRODUCERS: &'static [&'static str] = &[];

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
    pub(super) operation_type: TypeId,
    pub(super) installed_type: TypeId,
}

struct InstalledConditionalBinding {
    binding_type: TypeId,
    installed_type: TypeId,
    operation_type: TypeId,
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
    pub(in crate::domain_computation::primary_graph) fn operation_types(
        &self,
    ) -> impl Iterator<Item = TypeId> + '_ {
        self.entries.values().map(|entry| entry.operation_type)
    }

    pub(in crate::domain_computation::primary_graph) fn binding_count(&self) -> usize {
        self.entries.len()
    }

    pub(in crate::domain_computation::primary_graph) fn contains_operation<Operation: 'static>(
        &self,
    ) -> bool {
        self.operation_types()
            .any(|operation| operation == TypeId::of::<Operation>())
    }

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
            declaration.contract.operation_binding().clone(),
            TypeId::of::<Binding::Operation>(),
            declaration.contract.target().clone(),
            declaration.required_producers.clone(),
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
            operation_type: TypeId::of::<Binding::Operation>(),
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
        self.register_with_requirements::<Binding>(
            owner,
            configuration,
            Binding::REQUIRED_PRODUCERS
                .iter()
                .map(|identity| (*identity).to_owned())
                .collect(),
        )
    }

    pub(super) fn register_with_requirements<Binding>(
        &mut self,
        owner: &str,
        configuration: Binding::Configuration,
        required_producers: Vec<String>,
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
            || declared.operation_type != TypeId::of::<Binding::Operation>()
            || declared.installed_type != TypeId::of::<Binding::Installed>()
            || declared.required_producers != required_producers
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

    pub(in crate::domain_computation::primary_graph) fn validate_target_coverage(
        &self,
        index: &worth_query_installation::facade::WorthQueryInstalledPackageIndex,
        owner: &str,
        schema: &str,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        target_coverage::validate(&self.declared, index, owner, schema)
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
