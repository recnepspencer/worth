//! Lower each declared demand with the existing intent and scope vocabulary.
use super::*;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeBinding,
        ApplicationQueryScopeResolution,
    },
    application_schema::{fragment::RecordSelection, ApplicationScalarValueBinding},
};
pub(super) type Family<S, C> = <<C as WorthQueryApplicationRequiredOutputConnection<S>>::Demand as WorthQueryApplicationOutputDemand<S>>::OutputFamily;
pub(super) type Source<S, C> = <Family<S, C> as WorthQueryProducerOutputFamily<S>>::Source;

pub(in crate::domain_computation::primary_graph) struct CoupledSourceArguments {
    pub(in crate::domain_computation::primary_graph) parameters: Box<dyn Any>,
    pub(in crate::domain_computation::primary_graph) entity: &'static str,
    pub(in crate::domain_computation::primary_graph) aspect: &'static str,
    pub(in crate::domain_computation::primary_graph) field: &'static str,
    pub(in crate::domain_computation::primary_graph) value: worth_foundational::facade::AspectValue,
}

pub(in crate::domain_computation::primary_graph) fn lower<Schema, Connection>(
    demand: &dyn Any,
    principal: &dyn Any,
) -> Result<CoupledSourceArguments, Denial>
where
    Schema: ApplicationSchema,
    Connection: WorthQueryApplicationRequiredOutputConnection<Schema>,
    <Source<Schema, Connection> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<Schema, Binding = Source<Schema, Connection>>,
    <Source<Schema, Connection> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <Source<Schema, Connection> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
{
    lower_named::<Schema, Connection::Demand>(Connection::IDENTITY, demand, principal)
}

pub(in crate::domain_computation::primary_graph) fn lower_demand<Schema, D>(
    demand: &dyn Any,
    principal: &dyn Any,
) -> Result<CoupledSourceArguments, Denial>
where
    Schema: ApplicationSchema,
    D: WorthQueryApplicationOutputDemand<Schema>,
    <DemandSource<Schema, D> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<Schema, Binding = DemandSource<Schema, D>>,
    <DemandSource<Schema, D> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <DemandSource<Schema, D> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
{
    lower_named::<Schema, D>(
        <D::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::IDENTITY,
        demand,
        principal,
    )
}
fn lower_named<Schema, D>(
    subject: &str,
    demand: &dyn Any,
    principal: &dyn Any,
) -> Result<CoupledSourceArguments, Denial>
where
    Schema: ApplicationSchema,
    D: WorthQueryApplicationOutputDemand<Schema>,
    <DemandSource<Schema, D> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<Schema, Binding = DemandSource<Schema, D>>,
    <DemandSource<Schema, D> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <DemandSource<Schema, D> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
{
    type Scope<S, D> = <DemandSource<S, D> as ApplicationQueryBinding<S>>::ScopeBinding;
    type Field<S, D> = <Scope<S, D> as ApplicationQueryScopeBinding<S>>::Field;
    let denied = || Denial::new(Kind::DecisionDependencyMismatch, subject);
    let demand = demand.downcast_ref::<D>().ok_or_else(denied)?;
    let principal = principal.downcast_ref::<<DemandSource<Schema,D> as ApplicationQueryBinding<Schema>>::PrincipalIdentity>().ok_or_else(denied)?;
    let intent = demand.source_intent();
    let parameters = intent.parameters();
    let (field, value) = intent.into_scope().into_field_parts(principal);
    let value =
        <Field<Schema, D> as RecordSelection>::Binding::encode(&value).map_err(|_| denied())?;
    Ok(CoupledSourceArguments {
        parameters: Box::new(parameters),
        entity: field.entity(),
        aspect: field.aspect(),
        field: field.field(),
        value,
    })
}
type DemandSource<S, D> =
    <<D as WorthQueryApplicationOutputDemand<S>>::OutputFamily as WorthQueryProducerOutputFamily<
        S,
    >>::Source;
