//! Lower each declared demand with the existing intent and scope vocabulary.
use super::*;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeBinding,
        ApplicationQueryScopeResolution,
    },
    application_schema::{fragment::RecordSelection, ApplicationScalarValueBinding},
};
pub type Family<S, C> = <<C as WorthQueryApplicationRequiredOutputConnection<S>>::Demand as WorthQueryApplicationOutputDemand<S>>::OutputFamily;
pub type Source<S, C> = <Family<S, C> as WorthQueryProducerOutputFamily<S>>::Source;

pub struct CoupledSourceArguments {
    pub parameters: Box<dyn Any>,
    pub entity: &'static str,
    pub aspect: &'static str,
    pub field: &'static str,
    pub value: worth_foundational::facade::AspectValue,
}

pub fn lower<Schema, Connection>(
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

pub fn lower_demand<Schema, D>(
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
