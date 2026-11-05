//! Exact source address issued after native source disclosure and route selection.
//!
//! This address names a dependency, not a completed output or permission to
//! execute. Its partition survives the first publication and a change of the
//! family's Initial/Preserve binding; changing source epochs are values.

use crate::domain_computation::primary_graph::application_contribution::producer::demand::selection::selected_basis::SelectionSelectedBasisScopedOps;

use worth_query_installation::facade::ApplicationSchema;

use super::WorthQueryProducerOutputFamily;
use crate::domain_computation::vocabulary::authorization::operation_scope_binding::WorthQueryOperationScopeEntityBinding;
use crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind;
use crate::domain_computation::vocabulary::primary_graph::application_query_basis_registry::WorthQueryApplicationBasisSelectionIdentity;
use crate::domain_computation::primary_graph::vocabulary::output_family_identity::OutputFamilyIdentity;
use crate::domain_computation::primary_graph::WorthQueryObservedSource;
use crate::domain_computation::primary_graph::vocabulary::application_contribution::source_qualification::OutputSourceAddress;

/// A complete native query occurrence and selected installed family route.
/// It deliberately contains no Current/Pending classification.
pub(in crate::domain_computation::primary_graph) struct QualifiedOutputSource {
    address: OutputSourceAddress,
    source_epoch: crate::domain_computation::primary_graph::vocabulary::application_query::identity_kind::WorthQueryRuntimeSourceIdentity,
    generation: u64,
    lifecycle: crate::domain_computation::primary_graph::vocabulary::application_contribution::producer::WorthQueryProducerLifecyclePosture,
    read_closure: std::sync::Arc<crate::domain_computation::primary_graph::application_query::SourceQueryReadClosure>,
}

impl QualifiedOutputSource {
    pub(in crate::domain_computation::primary_graph) fn address(&self) -> &OutputSourceAddress {
        &self.address
    }

    pub(in crate::domain_computation::primary_graph) fn source_epoch(
        &self,
    ) -> crate::domain_computation::primary_graph::vocabulary::application_query::identity_kind::WorthQueryRuntimeSourceIdentity
{
        self.source_epoch
    }

    pub(in crate::domain_computation::primary_graph) fn read_closure(
        &self,
    ) -> &std::sync::Arc<
        crate::domain_computation::primary_graph::application_query::SourceQueryReadClosure,
    > {
        &self.read_closure
    }

    pub(in crate::domain_computation::primary_graph) fn lifecycle(
        &self,
    ) -> crate::domain_computation::primary_graph::vocabulary::application_contribution::producer::WorthQueryProducerLifecyclePosture{
        self.lifecycle
    }

    pub(in crate::domain_computation::primary_graph) fn generation(&self) -> u64 {
        self.generation
    }
}

/// Invoked only after the native selector has admitted the same source and
/// returned the exact installed entry. Work is constant apart from complete
/// source-identity construction, which is charged by its existing issuer.
pub(super) fn qualify<Schema, Family>(
    source: &WorthQueryObservedSource<
        <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
    >,
    entry: &super::registry::InstalledProducerProvider<Schema>,
    selected: &super::WorthQuerySelectedApplicationProducer,
    read_closure: std::sync::Arc<
        crate::domain_computation::primary_graph::application_query::SourceQueryReadClosure,
    >,
) -> Result<QualifiedOutputSource, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    if !read_closure.admits_source(source)
        || !source.has_complete_output_dependencies()
        || entry.declaration.output_family_type
            != crate::equality_type_id::EqualityTypeId::of::<Family>()
        || entry.declaration.source_type
            != crate::equality_type_id::EqualityTypeId::of::<Family::Source>()
    {
        return Err(WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::IncompleteDependencyCoverage,
            Family::IDENTITY,
        ));
    }
    let WorthQueryApplicationBasisSelectionIdentity::Product(observation) = &source.selection
    else {
        return Err(WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::ForeignSource,
            Family::IDENTITY,
        ));
    };
    Ok(QualifiedOutputSource {
        address: OutputSourceAddress {
            runtime: source.runtime_authority,
            schema: source.schema_binding.clone(),
            occurrence: observation.lifecycle_incarnation(),
            scope: WorthQueryOperationScopeEntityBinding::from_entity(source.source_root()),
            family: OutputFamilyIdentity::declared(Family::IDENTITY),
            partition: source.partition_identity(),
        },
        source_epoch: source.idempotency_identity(),
        generation: observation.reference_generation().get(),
        lifecycle: selected.applicability.lifecycle(),
        read_closure,
    })
}

pub(in crate::domain_computation::primary_graph) trait SourceQualificationScopedOps<Schema: ApplicationSchema + 'static>: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> {
fn qualify_observed_output_source<Family>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observed: &crate::domain_computation::primary_graph::application_query::ObservedQuerySource<
            <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
            <<<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::ResultBinding as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::Value,
        >,
        shared: &crate::domain_computation::primary_graph::SharedSelectedProductOperation<
            '_,
            Schema,
        >,
        native: &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess<'_>,
    ) -> Result<QualifiedOutputSource, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let (selected_producer, entry) = self.select_output_producer_for_observation::<Family>(
            execution,
            &observed.source,
            Family::profile_kind(&observed.value),
            shared,
            native,
        )?;
        qualify::<Schema, Family>(
            &observed.source,
            entry,
            &selected_producer,
            std::sync::Arc::clone(&observed.closure),
        )
    }
}

impl<Schema: ApplicationSchema + 'static, HostRuntime: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> + ?Sized> SourceQualificationScopedOps<Schema> for HostRuntime {}
