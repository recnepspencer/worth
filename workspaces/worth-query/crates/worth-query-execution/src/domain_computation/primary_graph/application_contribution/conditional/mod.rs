use crate::equality_type_id::EqualityTypeId as TypeId;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;
use worth_query_execution_query::facade::ports::ConditionalProducerAccessInternalPort as _;

use worth_query_installation::facade::ApplicationSchema;

use super::super::conditional_operation::{
    WorthQueryConditionalApplicationRuntimeInstallation,
    WorthQueryConditionalRuntimeInstallationDenial,
};
use super::super::{
    WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind as DenialKind,
};
use super::producer::WorthQueryInstalledApplicationProducerRegistry;

pub use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::conditional::{
    output_readiness_contract, validate_required_producer,
    DeclaredConditionalBinding, InstalledConditionalBinding,
    WorthQueryApplicationConditionalPackageContract, WorthQueryApplicationConditionalProducerAccess,
    WorthQueryInstalledApplicationConditionalRegistry, WorthQueryOutputReadinessContractDenial,
};

mod batch_installation;
mod feature;
mod target_coverage;
pub use batch_installation::RefusedConditionalBatch;
use batch_installation::RefusedConditionalConfiguration;
pub use feature::WorthQueryConditionalFeature;

pub trait WorthQueryApplicationConditionalBinding<Schema>: Sized + 'static
where
    Schema: ApplicationSchema + 'static,
{
    type Configuration: 'static;
    type Installed: Send + Sync + 'static;
    type Operation: 'static;

    const IDENTITY: &'static str;
    const REQUIRED_PRODUCERS: &'static [&'static str] = &[];

    fn package_contract(
        declaration: &worth_query_declaration::facade::application_schema::ErasedApplicationSchemaDeclaration,
    ) -> Result<
        WorthQueryApplicationConditionalPackageContract,
        WorthQueryOutputReadinessContractDenial,
    >;

    fn install(
        configuration: Self::Configuration,
        producers: &WorthQueryApplicationConditionalProducerAccess<'_, Schema>,
        installation: &mut WorthQueryConditionalApplicationRuntimeInstallation<Schema>,
    ) -> Result<Self::Installed, WorthQueryConditionalRuntimeInstallationDenial>;
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
    ) -> Result<InstalledConditionalBinding, RefusedConditionalConfiguration>;
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
    ) -> Result<InstalledConditionalBinding, RefusedConditionalConfiguration> {
        let access = WorthQueryApplicationConditionalProducerAccess::from_installed(
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
                return Err(RefusedConditionalConfiguration {
                    cause: denial,
                    installed: None,
                });
            }
        };
        let installed = InstalledConditionalBinding {
            binding_type: TypeId::of::<Binding>(),
            installed_type: TypeId::of::<Binding::Installed>(),
            operation_type: TypeId::of::<Binding::Operation>(),
            value: Arc::new(installed),
        };
        match installation.finish_application_binding_scope() {
            Ok(()) => Ok(installed),
            Err(cause) => Err(RefusedConditionalConfiguration {
                cause,
                installed: Some(installed),
            }),
        }
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
    pub(in crate::domain_computation::primary_graph) fn new(
        declared: BTreeMap<String, DeclaredConditionalBinding>,
    ) -> Self {
        Self {
            declared,
            configured: BTreeMap::new(),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn register<Binding>(
        &mut self,
        installed: &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<Schema>,
        owner: &str,
        configuration: Binding::Configuration,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        Binding: WorthQueryApplicationConditionalBinding<Schema>,
    {
        self.register_with_requirements::<Binding>(
            installed,
            owner,
            configuration,
            Binding::REQUIRED_PRODUCERS
                .iter()
                .map(|identity| (*identity).to_owned())
                .collect(),
        )
    }

    pub(in crate::domain_computation::primary_graph) fn register_with_requirements<Binding>(
        &mut self,
        installed: &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<Schema>,
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
            || declared.contract != Binding::package_contract(installed.installed_declaration())?
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
        declared.contract.seal(installed)?;
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

    pub(in crate::domain_computation::primary_graph) fn validate_complete(
        &self,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
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
}

fn denial(
    kind: DenialKind,
    subject: impl Into<String>,
) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(kind, subject)
}
