//! Exact source address issued after native source disclosure and route selection.
//!
//! This address names a dependency, not a completed output or permission to
//! execute. Its partition survives the first publication and a change of the
//! family's Initial/Preserve binding; changing source epochs are values.

use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use worth_query_installation::facade::{ApplicationSchema, ApplicationSchemaBindingIdentity};
use worth_runtime_world::facade::ProductBranchIncarnation;

use super::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind, WorthQueryProducerOutputFamily,
};
use crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding;
use crate::domain_computation::primary_graph::{
    application_query::WorthQueryApplicationBasisSelectionIdentity,
    output_family_identity::OutputFamilyIdentity,
    output_lineage::invalidation::InvalidationEditAdmission, WorthQueryObservedSource,
};

/// Only native source qualification constructs this stable family address.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) struct OutputSourceAddress {
    runtime: u64,
    schema: ApplicationSchemaBindingIdentity,
    occurrence: ProductBranchIncarnation,
    scope: WorthQueryOperationScopeEntityBinding,
    family: OutputFamilyIdentity,
    partition: [u8; 32],
}

impl Hash for OutputSourceAddress {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.runtime.hash(state);
        self.schema.runtime_ordinal().hash(state);
        self.schema.generation().hash(state);
        self.schema.package_identity().hash(state);
        self.schema.schema_identity().hash(state);
        self.occurrence.hash(state);
        self.scope.hash(state);
        self.family.hash(state);
        self.partition.hash(state);
    }
}

impl Ord for OutputSourceAddress {
    fn cmp(&self, other: &Self) -> Ordering {
        let prefix = |address: &Self| {
            (
                address.runtime,
                address.schema.runtime_ordinal(),
                address.schema.generation(),
                *address.schema.package_identity(),
                *address.schema.schema_identity(),
                address.occurrence,
                address.scope,
            )
        };
        prefix(self)
            .cmp(&prefix(other))
            .then_with(|| self.family.cmp(&other.family))
            .then_with(|| self.partition.cmp(&other.partition))
    }
}

impl PartialOrd for OutputSourceAddress {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A complete native query occurrence and selected installed family route.
/// It deliberately contains no Current/Pending classification.
pub(in crate::domain_computation::primary_graph) struct QualifiedOutputSource {
    address: OutputSourceAddress,
    source_epoch: crate::domain_computation::primary_graph::application_query::WorthQueryRuntimeSourceIdentity,
    generation: u64,
    lifecycle: super::WorthQueryProducerLifecyclePosture,
    read_closure: std::sync::Arc<crate::domain_computation::primary_graph::application_query::SourceQueryReadClosure>,
}

impl QualifiedOutputSource {
    pub(in crate::domain_computation::primary_graph) fn address(&self) -> &OutputSourceAddress {
        &self.address
    }

    pub(in crate::domain_computation::primary_graph) fn source_epoch(
        &self,
    ) -> crate::domain_computation::primary_graph::application_query::WorthQueryRuntimeSourceIdentity
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
    ) -> super::WorthQueryProducerLifecyclePosture {
        self.lifecycle
    }

    pub(in crate::domain_computation::primary_graph) fn generation(&self) -> u64 {
        self.generation
    }
}

impl OutputSourceAddress {
    pub(in crate::domain_computation::primary_graph) fn runtime(&self) -> u64 {
        self.runtime
    }
    pub(in crate::domain_computation::primary_graph) fn schema(
        &self,
    ) -> &ApplicationSchemaBindingIdentity {
        &self.schema
    }
    pub(in crate::domain_computation::primary_graph) fn occurrence(
        &self,
    ) -> ProductBranchIncarnation {
        self.occurrence
    }
    pub(in crate::domain_computation::primary_graph) fn scope(
        &self,
    ) -> WorthQueryOperationScopeEntityBinding {
        self.scope
    }
    pub(in crate::domain_computation::primary_graph) fn family(&self) -> &OutputFamilyIdentity {
        &self.family
    }
    pub(in crate::domain_computation::primary_graph) fn partition(&self) -> [u8; 32] {
        self.partition
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
    admission: &mut InvalidationEditAdmission,
) -> Result<QualifiedOutputSource, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    admission.charge_external_work(8 + 4 * 32).map_err(|_| {
        WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, "")
    })?;
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

impl<Schema: ApplicationSchema + 'static>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    pub(in crate::domain_computation::primary_graph) fn qualify_observed_output_source<Family>(
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
        admission: &mut InvalidationEditAdmission,
    ) -> Result<QualifiedOutputSource, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let (selected_producer, entry) = self.select_output_producer_for_observation::<Family>(
            execution,
            &observed.source,
            Family::profile_kind(&observed.value),
            admission,
            shared,
            native,
        )?;
        qualify::<Schema, Family>(
            &observed.source,
            entry,
            &selected_producer,
            std::sync::Arc::clone(&observed.closure),
            admission,
        )
    }
}
