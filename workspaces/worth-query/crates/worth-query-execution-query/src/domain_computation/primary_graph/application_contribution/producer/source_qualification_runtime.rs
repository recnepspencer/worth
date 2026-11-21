//! Runtime qualification of an applied query source onto an `OutputSourceAddress`.

use worth_query_execution_lineage::facade::ports::ObservedSourceInternalPort as _;
use worth_query_installation::facade::ApplicationSchema;

use super::{
    QualifiedOutputSource, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQueryProducerOutputFamily,
};
use crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding;
use crate::domain_computation::primary_graph::application_contribution::producer::source_qualification::OutputSourceAddress;
use crate::domain_computation::primary_graph::output_family_identity::OutputFamilyIdentity;

impl<Schema: ApplicationSchema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    pub fn qualify_applied_output_source<Family, Value>(
        &self,
        source: &crate::domain_computation::primary_graph::application_query::AppliedQuerySource<
            <Family::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
            Value,
        >,
        before: &crate::basis::WorthQueryProductBranchLease,
        lifecycle: super::WorthQueryProducerLifecyclePosture,
    ) -> Result<QualifiedOutputSource, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let foreign = || {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                Family::IDENTITY,
            )
        };
        let installed = self
            .installed_schema()
            .installed_query_binding::<Family::Source>()
            .map_err(|_| foreign())?;
        if source.runtime_authority != self.runtime.authority_identity().as_u64()
            || !source.has_complete_output_dependencies()
            || source.schema_binding != self.installed_schema.binding_identity()
            || source.binding_order().0 != *installed.query().identity().as_bytes()
            || source.candidate.proposal().branch_observation()
                != before.relational_basis().reference()
            || source.closure.applied_candidate() != Some(&source.candidate)
        {
            return Err(foreign());
        }
        let observation = before.observation();
        Ok(QualifiedOutputSource::from_parts(
            OutputSourceAddress::from_parts(
                source.runtime_authority,
                source.schema_binding.clone(),
                observation.lifecycle_incarnation(),
                WorthQueryOperationScopeEntityBinding::from_entity(source.model_root),
                OutputFamilyIdentity::declared(Family::IDENTITY),
                source.partition(),
            ),
            source.runtime_source_identity(),
            observation.reference_generation().get(),
            lifecycle,
            std::sync::Arc::clone(&source.closure)
                as std::sync::Arc<
                    dyn worth_query_execution_lineage::facade::ports::SourceReadClosurePort,
                >,
        ))
    }
}

pub fn qualify<Schema, Family>(
    source: &crate::domain_computation::primary_graph::WorthQueryObservedSource<
        <Family::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
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
    let crate::domain_computation::primary_graph::application_query::WorthQueryApplicationBasisSelectionIdentity::Product(observation) = &source.selection()
    else {
        return Err(WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::ForeignSource,
            Family::IDENTITY,
        ));
    };
    Ok(QualifiedOutputSource::from_parts(
        OutputSourceAddress::from_parts(
            source.runtime_authority(),
            source.schema_binding().clone(),
            observation.lifecycle_incarnation(),
            WorthQueryOperationScopeEntityBinding::from_entity(source.source_root()),
            OutputFamilyIdentity::declared(Family::IDENTITY),
            source.partition_identity(),
        ),
        source.idempotency_identity(),
        observation.reference_generation().get(),
        selected.applicability.lifecycle(),
        read_closure
            as std::sync::Arc<
                dyn worth_query_execution_lineage::facade::ports::SourceReadClosurePort,
            >,
    ))
}

impl<Schema: ApplicationSchema + 'static>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    pub fn qualify_observed_output_source<Family>(
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
