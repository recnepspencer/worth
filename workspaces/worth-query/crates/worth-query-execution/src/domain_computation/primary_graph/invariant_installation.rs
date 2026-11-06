use std::collections::BTreeMap;
use std::marker::PhantomData;

mod descriptor;
mod program_activation_work_reserve;
use descriptor::{is_mandatory_candidate_registration, lower_descriptor};
mod schema_resolver;
mod workflow_publication;
pub use schema_resolver::WorthQueryApplicationInvariantSchemaResolver;

use worth_query_declaration::facade::application_schema::{
    ApplicationInvariantExecutionPoint, ApplicationSchema, ApplicationSchemaBindingIdentity,
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationInvariant, WorthQueryInstalledApplicationSchema,
};
use worth_relational::facade::runtime::CustomInvariantRegistration;

use {crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenial, crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenialKind};

use super::application_invariant::{
    ErasedApplicationInvariantRule, WorthQueryApplicationInvariantRule,
    WorthQueryProgramRuleSelection,
};
use super::program_occurrence::WorthQueryProgramActivationCell;

/// What a program-hosted installation carries into invariant lowering so every
/// lowered adapter is born knowing which rostered programs declare its rule.
///
/// The roster is admitted before the graph is lowered and never changes, so no
/// post-hoc table exists to disagree with it.
pub(super) struct WorthQueryInvariantProgramBasis<Schema> {
    pub(super) roster:
        std::sync::Arc<worth_query_installation::facade::WorthQueryProgramSupportRoster<Schema>>,
    pub(super) activation: WorthQueryProgramActivationCell,
}

impl<Schema> WorthQueryInvariantProgramBasis<Schema> {
    pub(super) const fn admitted(
        roster: std::sync::Arc<
            worth_query_installation::facade::WorthQueryProgramSupportRoster<Schema>,
        >,
        activation: WorthQueryProgramActivationCell,
    ) -> Self {
        Self { roster, activation }
    }
}

type Factory<Schema> = Box<
    dyn for<'resolver> FnOnce(
            &WorthQueryApplicationInvariantSchemaResolver<'resolver, Schema>,
        )
            -> Result<Box<dyn ErasedApplicationInvariantRule<Schema>>, String>
        + Send,
>;

pub struct WorthQueryApplicationInvariantFactories<Schema> {
    binding_identity: ApplicationSchemaBindingIdentity,
    expected: BTreeMap<
        (String, ApplicationInvariantExecutionPoint),
        worth_query_installation::facade::WorthQueryInstalledApplicationInvariantDescriptor,
    >,
    factories: BTreeMap<(String, ApplicationInvariantExecutionPoint), Factory<Schema>>,
    installed_order: Vec<(String, ApplicationInvariantExecutionPoint)>,
    program_basis: Option<WorthQueryInvariantProgramBasis<Schema>>,
    managed_indexes: super::application_query::managed_index::ManagedIndexProviderCatalog<Schema>,
    _schema: PhantomData<fn() -> Schema>,
}

impl<Schema> WorthQueryApplicationInvariantFactories<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn for_installed_schema(schema: &WorthQueryInstalledApplicationSchema<Schema>) -> Self {
        let expected = schema
            .invariants()
            .descriptors()
            .map(|descriptor| {
                (
                    (
                        descriptor.identifier().to_owned(),
                        descriptor.execution_point(),
                    ),
                    descriptor.clone(),
                )
            })
            .collect();
        Self {
            binding_identity: schema.binding_identity(),
            expected,
            factories: BTreeMap::new(),
            installed_order: Vec::new(),
            program_basis: None,
            managed_indexes: Default::default(),
            _schema: PhantomData,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn with_managed_index_catalog(
        mut self,
        catalog: super::application_query::managed_index::ManagedIndexProviderCatalog<Schema>,
    ) -> Self {
        self.managed_indexes = catalog;
        self
    }

    /// Binds the admitted support roster this installation lowers under, so a
    /// rule only decides candidates running under a program that declares it.
    pub(super) fn select_by_program(&mut self, basis: WorthQueryInvariantProgramBasis<Schema>) {
        self.program_basis = Some(basis);
    }

    pub fn bind<Invariant, Rule: WorthQueryApplicationInvariantRule<Schema>>(
        &mut self,
        installed: &WorthQueryInstalledApplicationInvariant<Schema, Invariant>,
        factory: impl for<'resolver> FnOnce(
                &WorthQueryApplicationInvariantSchemaResolver<'resolver, Schema>,
            ) -> Result<Rule, String>
            + Send
            + 'static,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        if installed.binding_identity() != &self.binding_identity {
            return Err(denial(
                WorthQueryPrimaryGraphInstallationDenialKind::ForeignInvariantFactory,
                installed.descriptor().identifier(),
            ));
        }
        let key = (
            installed.descriptor().identifier().to_owned(),
            installed.descriptor().execution_point(),
        );
        if self.factories.contains_key(&key) {
            return Err(denial(
                WorthQueryPrimaryGraphInstallationDenialKind::DuplicateInvariantFactory,
                installed.descriptor().identifier(),
            ));
        }
        self.installed_order.push(key.clone());
        self.factories.insert(
            key,
            Box::new(move |resolver| {
                factory(resolver)
                    .map(|rule| Box::new(rule) as Box<dyn ErasedApplicationInvariantRule<Schema>>)
            }),
        );
        Ok(())
    }

    pub(super) fn lower(
        mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding_identity: ApplicationSchemaBindingIdentity,
        layout: &crate::domain_computation::graph_layout::WorthQueryPrimaryGraphLayout,
    ) -> Result<
        (
            Vec<CustomInvariantRegistration>,
            super::application_query::managed_index::ManagedSourceInstallation,
            Option<super::application_query::managed_index::SourceInitializationIssuer>,
        ),
        WorthQueryPrimaryGraphInstallationDenial,
    > {
        if binding_identity != self.binding_identity {
            return Err(denial(
                WorthQueryPrimaryGraphInstallationDenialKind::ForeignInvariantFactory,
                "factory inventory belongs to another installed schema",
            ));
        }
        if self.factories.len() != self.expected.len() {
            return Err(denial(
                WorthQueryPrimaryGraphInstallationDenialKind::MissingInvariantFactory,
                "factory inventory does not match the installed invariant catalog",
            ));
        }
        // Compile supported physical Source shapes and bind the actual configured
        // providers before publishing factories. Unsupported shapes still refuse
        // in this compiler; every supported rule consumes Native's staged loans.
        let (sources, initializer) =
            super::application_query::managed_index::ManagedSourceInstallation::compile(
                execution,
                &self.managed_indexes,
                layout,
            )?;
        let mut lowered = Vec::with_capacity(self.expected.len());
        for key in self.installed_order {
            let installed = self.expected.remove(&key).ok_or_else(|| {
                denial(
                    WorthQueryPrimaryGraphInstallationDenialKind::ForeignInvariantFactory,
                    &key.0,
                )
            })?;
            if !self.factories.contains_key(&key) {
                return Err(denial(
                    WorthQueryPrimaryGraphInstallationDenialKind::MissingInvariantFactory,
                    &key.0,
                ));
            }
            let factory = self
                .factories
                .remove(&key)
                .expect("checked factory remains present");
            let resolver = WorthQueryApplicationInvariantSchemaResolver {
                layout,
                descriptor: &installed,
                binding_identity: self.binding_identity.clone(),
                _schema: PhantomData,
            };
            let rule = factory(&resolver).map_err(|detail| {
                denial(
                    WorthQueryPrimaryGraphInstallationDenialKind::InvariantFactoryRejected,
                    detail,
                )
            })?;
            let descriptor = lower_descriptor(&installed, layout, self.program_basis.is_some())?;
            let program_selection = self
                .program_basis
                .as_ref()
                .map(|basis| {
                    WorthQueryProgramRuleSelection::for_installed_rule(
                        &installed,
                        &basis.roster,
                        basis.activation.clone(),
                        layout.program_activation(),
                    )
                })
                .transpose()
                .map_err(|detail| {
                    denial(
                        WorthQueryPrimaryGraphInstallationDenialKind::InvariantFactoryRejected,
                        detail,
                    )
                })?;
            let programs = installed
                .managed_index_sources()
                .iter()
                .map(|requirement| {
                    layout
                        .managed_source(requirement.index())
                        .cloned()
                        .ok_or_else(|| {
                            denial(
                    WorthQueryPrimaryGraphInstallationDenialKind::UnsupportedInvariantSource,
                    requirement.index(),
                )
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let registration = rule
                .into_registration(descriptor, self.binding_identity.clone(), program_selection)
                .map_err(|detail| {
                    denial(
                        WorthQueryPrimaryGraphInstallationDenialKind::InvariantFactoryRejected,
                        detail,
                    )
                })?
                .with_managed_index_sources(programs)
                .map_err(|cause| {
                    denial(
                        WorthQueryPrimaryGraphInstallationDenialKind::UnsupportedInvariantSource,
                        format!("{}: {cause:?}", installed.identifier()),
                    )
                })?;
            if !is_mandatory_candidate_registration(&registration) {
                return Err(denial(
                    WorthQueryPrimaryGraphInstallationDenialKind::InvariantFactoryMeaningMismatch,
                    &key.0,
                ));
            }
            lowered.push(registration);
        }
        lowered.push(workflow_publication::registration(layout.workflow())?);
        Ok((lowered, sources, initializer))
    }
}

fn denial(
    kind: WorthQueryPrimaryGraphInstallationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(kind, subject)
}

mod court_doors_for_test;
