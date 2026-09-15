use std::marker::PhantomData;

use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_program::{
    ApplicationConnectionIdentity, ApplicationConnectionRef, ApplicationFeature,
    ApplicationInputPort, ApplicationOccurrenceConnectionBinding, ApplicationOutputPort,
    ApplicationProgramDefinition, ApplicationProgramDependentConnection,
    ApplicationProgramInventoryIdentity, ApplicationProgramRequiredConnection,
    ApplicationProgramUnavailableConnection,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationStructuredValueBinding,
};
use worth_query_execution::facade::application_contribution::{
    WorthQueryApplicationOutputDemand, WorthQueryProducerOutputFamily,
};
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationDependentOutputConnection, WorthQueryApplicationProjection,
    WorthQueryApplicationRequiredOutputConnection, WorthQueryRequiredOutputConnectionDenial,
};

use super::node::{
    downcast_authority, ErasedProgramNode, ErasedProgramSettlement, TypedProgramNode,
};
use crate::application_entry::{
    WorthQueryOutputDemandControls, WorthQueryRequiredOutputPreparationDenial,
};

type Connection<Schema, SourceFeature, SourcePort, TargetFeature, TargetPort, Binding> =
    ApplicationConnectionRef<Schema, SourceFeature, SourcePort, TargetFeature, TargetPort, Binding>;
type Family<Schema, Demand> = <Demand as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type DemandSource<Schema, Demand> =
    <Family<Schema, Demand> as WorthQueryProducerOutputFamily<Schema>>::Source;
type DemandQuery<Schema, Demand> =
    <DemandSource<Schema, Demand> as ApplicationQueryBinding<Schema>>::Query;
type DemandValue<Schema, Demand> = <<DemandSource<Schema, Demand> as ApplicationQueryBinding<
    Schema,
>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type DiscoveryBinding<Schema, Binding> =
    <<Binding as WorthQueryApplicationDependentOutputConnection<Schema>>::Discovery as ApplicationQueryIntent<Schema>>::Binding;
type DiscoveryValue<Schema, Binding> =
    <DiscoveryBinding<Schema, Binding> as ApplicationQueryBinding<Schema>>::ResultBinding;
type DiscoveryRow<Schema, Binding> =
    <DiscoveryValue<Schema, Binding> as ApplicationStructuredValueBinding>::Value;

mod root_connection_seal {
    use super::*;

    pub trait Sealed {}

    impl<Schema, SourceFeature, SourcePort, TargetFeature, TargetPort, Binding> Sealed
        for ApplicationProgramRequiredConnection<
            ApplicationConnectionRef<
                Schema,
                SourceFeature,
                SourcePort,
                TargetFeature,
                TargetPort,
                Binding,
            >,
        >
    {
    }
}

pub trait WorthQueryProgramRootConnection<Schema>:
    root_connection_seal::Sealed + Sized + 'static
where
    Schema: ApplicationSchema,
{
    type Source: ApplicationMutationBinding<Schema>;
    type Demand: WorthQueryApplicationOutputDemand<Schema> + Clone;
    const IDENTITY: &'static str;
    const TARGET_FEATURE: &'static str;
    fn target_feature_type() -> std::any::TypeId;

    fn validate_source(
        source: &<Self::Source as ApplicationMutationBinding<Schema>>::Input,
    ) -> Result<(), WorthQueryRequiredOutputConnectionDenial>;

    fn demand_from_committed_source(
        source: &<Self::Source as ApplicationMutationBinding<Schema>>::Input,
        result: &<Self::Source as ApplicationMutationBinding<Schema>>::Result,
    ) -> Self::Demand;
}

impl<Schema, SourceFeature, SourcePort, TargetFeature, TargetPort, Binding>
    WorthQueryProgramRootConnection<Schema>
    for ApplicationProgramRequiredConnection<
        Connection<Schema, SourceFeature, SourcePort, TargetFeature, TargetPort, Binding>,
    >
where
    Schema: ApplicationSchema,
    SourceFeature: ApplicationFeature<Schema>,
    TargetFeature: ApplicationFeature<Schema>,
    SourcePort: ApplicationOutputPort<Schema, SourceFeature>,
    TargetPort: ApplicationInputPort<Schema, TargetFeature, Value = SourcePort::Value>,
    Binding: ApplicationOccurrenceConnectionBinding<Schema, SourceFeature, TargetFeature>
        + WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding::Demand: Clone,
{
    type Source = Binding::Source;
    type Demand = Binding::Demand;
    const IDENTITY: &'static str = <Binding as ApplicationConnectionIdentity>::IDENTITY;
    const TARGET_FEATURE: &'static str = TargetFeature::IDENTITY;

    fn target_feature_type() -> std::any::TypeId {
        std::any::TypeId::of::<TargetFeature>()
    }

    fn validate_source(
        source: &<Self::Source as ApplicationMutationBinding<Schema>>::Input,
    ) -> Result<(), WorthQueryRequiredOutputConnectionDenial> {
        Binding::validate_source(source)
    }

    fn demand_from_committed_source(
        source: &<Self::Source as ApplicationMutationBinding<Schema>>::Input,
        result: &<Self::Source as ApplicationMutationBinding<Schema>>::Result,
    ) -> Self::Demand {
        Binding::demand_from_committed_source(source, result)
    }
}

pub(super) trait ErasedProgramConnection<'application, Schema, Program, Inventory>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
{
    fn identity(&self) -> &'static str;
    fn source_feature(&self) -> &'static str;
    fn target_feature(&self) -> &'static str;
    fn start(
        &self,
        parent: &ErasedProgramSettlement,
        request: &crate::application_entry::WorthQueryApplicationRequest<
            'application,
            '_,
            '_,
            Schema,
        >,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        controls: WorthQueryOutputDemandControls,
    ) -> Result<
        Vec<Box<dyn ErasedProgramNode<'application, Schema> + 'application>>,
        WorthQueryRequiredOutputPreparationDenial,
    >;
}

struct TypedDependentConnection<Schema, Program, Inventory, SourceFeature, TargetFeature, Binding>(
    PhantomData<
        fn() -> (
            Schema,
            Program,
            Inventory,
            SourceFeature,
            TargetFeature,
            Binding,
        ),
    >,
);

impl<'application, Schema, Program, Inventory, SourceFeature, TargetFeature, Binding>
    ErasedProgramConnection<'application, Schema, Program, Inventory>
    for TypedDependentConnection<
        Schema,
        Program,
        Inventory,
        SourceFeature,
        TargetFeature,
        Binding,
    >
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
    SourceFeature: ApplicationFeature<Schema>,
    TargetFeature: ApplicationFeature<Schema>,
    Binding: WorthQueryApplicationDependentOutputConnection<Schema> + ApplicationConnectionIdentity,
    Binding::RootDemand: Clone + 'static,
    Binding::Demand: Clone + 'static,
    Binding::Discovery: ApplicationQueryIntent<Schema>,
    DiscoveryRow<Schema, Binding>: WorthQueryApplicationProjection<
            Schema,
            <DiscoveryBinding<Schema, Binding> as ApplicationQueryBinding<Schema>>::Query,
        > + Clone,
    <DiscoveryBinding<Schema, Binding> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <DiscoveryBinding<Schema, Binding> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    DemandQuery<Schema, Binding::Demand>: 'static,
    DemandValue<Schema, Binding::Demand>:
        WorthQueryApplicationProjection<Schema, DemandQuery<Schema, Binding::Demand>>
            + Clone
            + 'static,
    <DemandSource<Schema, Binding::Demand> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<Schema, Binding = DemandSource<Schema, Binding::Demand>>,
    <DemandSource<Schema, Binding::Demand> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <DemandSource<Schema, Binding::Demand> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
{
    fn identity(&self) -> &'static str {
        <Binding as ApplicationConnectionIdentity>::IDENTITY
    }

    fn source_feature(&self) -> &'static str {
        SourceFeature::IDENTITY
    }

    fn target_feature(&self) -> &'static str {
        TargetFeature::IDENTITY
    }

    fn start(
        &self,
        parent: &ErasedProgramSettlement,
        request: &crate::application_entry::WorthQueryApplicationRequest<
            'application,
            '_,
            '_,
            Schema,
        >,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        controls: WorthQueryOutputDemandControls,
    ) -> Result<
        Vec<Box<dyn ErasedProgramNode<'application, Schema> + 'application>>,
        WorthQueryRequiredOutputPreparationDenial,
    > {
        let root = parent
            .demand
            .downcast_ref::<Binding::RootDemand>()
            .ok_or_else(|| connection_denial("program parent demand type mismatch"))?;
        let authority = downcast_authority::<
            Schema,
            Program,
            Inventory,
            Binding::RootDemand,
        >(parent)
        .ok_or_else(|| connection_denial("program parent authority type mismatch"))?;
        let discovery = Binding::discovery_from_root(root)
            .map_err(WorthQueryRequiredOutputPreparationDenial::Connection)?;
        let retained = request.at(&parent.observation);
        let discovered = retained
            .query(discovery)
            .execute()
            .map_err(WorthQueryRequiredOutputPreparationDenial::SourceQuery)?;
        let mut nodes = Vec::<Box<dyn ErasedProgramNode<'application, Schema>>>::new();
        for row in discovered.rows() {
            let demands = Binding::demands_from_discovery(row)
                .map_err(WorthQueryRequiredOutputPreparationDenial::Connection)?;
            for demand in demands {
                let handle = retained
                    .demand(demand.clone())
                    .controls(controls)
                    .start_dependent::<Program, Inventory, Binding::RootDemand>(application, authority)
                    .map_err(WorthQueryRequiredOutputPreparationDenial::Demand)?;
                nodes.push(Box::new(TypedProgramNode::new(
                    TargetFeature::IDENTITY,
                    std::any::TypeId::of::<TargetFeature>(),
                    handle,
                )));
            }
        }
        Ok(nodes)
    }
}

fn connection_denial(subject: &str) -> WorthQueryRequiredOutputPreparationDenial {
    WorthQueryRequiredOutputPreparationDenial::Connection(
        WorthQueryRequiredOutputConnectionDenial::new(subject),
    )
}

pub trait WorthQueryProgramConnectionPlan<Schema, Program, Inventory>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
{
    fn connections<'application>(
    ) -> super::WorthQueryProgramConnectionFactories<'application, Schema, Program, Inventory>
    where
        Schema: 'application,
        Program: 'application,
        Inventory: 'application;
}

mod tuple;
