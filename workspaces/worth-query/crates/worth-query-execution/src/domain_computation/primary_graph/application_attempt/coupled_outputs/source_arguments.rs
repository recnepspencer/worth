//! Lower each declared demand with the existing intent and scope vocabulary.
use super::*;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeBinding,
        ApplicationQueryScopeResolution,
    },
    application_schema::{ApplicationScalarValueBinding, DeclaredApplicationFieldValue},
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

pub(super) fn lower<Schema, Connection>(
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
    type Scope<S, C> = <Source<S, C> as ApplicationQueryBinding<S>>::ScopeBinding;
    type Field<S, C> = <Scope<S, C> as ApplicationQueryScopeBinding<S>>::Field;
    let denied = || Denial::new(Kind::DecisionDependencyMismatch, Connection::IDENTITY);
    let demand = demand
        .downcast_ref::<Connection::Demand>()
        .ok_or_else(denied)?;
    let principal = principal.downcast_ref::<<Source<Schema, Connection> as ApplicationQueryBinding<Schema>>::PrincipalIdentity>().ok_or_else(denied)?;
    let intent = demand.source_intent();
    let parameters = intent.parameters();
    let (field, value) = intent.into_scope().into_field_parts(principal);
    let value =
        <Field<Schema, Connection> as DeclaredApplicationFieldValue>::Binding::encode(&value)
            .map_err(|_| denied())?;
    Ok(CoupledSourceArguments {
        parameters: Box::new(parameters),
        entity: field.entity(),
        aspect: field.aspect(),
        field: field.field(),
        value,
    })
}
