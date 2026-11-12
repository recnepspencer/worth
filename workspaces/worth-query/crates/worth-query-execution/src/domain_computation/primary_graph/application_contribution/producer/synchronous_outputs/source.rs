//! Typed demands lower through the existing coupled source-argument owner.
use super::*;
use crate::{
    domain_computation::primary_graph::application_attempt::coupled_outputs::source_arguments,
    equality_type_id::EqualityTypeId,
};
use worth_query_declaration::facade::{
    application_program::WorthQueryProducerOutputFamily,
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
};
pub(super) type Family<S, D> = <D as WorthQueryApplicationOutputDemand<S>>::OutputFamily;
pub(super) type Source<S, D> = <Family<S, D> as WorthQueryProducerOutputFamily<S>>::Source;
pub(super) type SourceInput<S, D> = <Source<S, D> as ApplicationQueryBinding<S>>::Input;
pub(super) fn capture<S, D>(connection: &'static str, demand: D) -> DeclaredCoupledOutputDemand
where
    S: ApplicationSchema,
    D: WorthQueryApplicationOutputDemand<S> + Send,
    SourceInput<S, D>: ApplicationQueryIntent<S, Binding = Source<S, D>>,
    <Source<S, D> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<
        S,
        <Source<S, D> as ApplicationQueryBinding<S>>::PrincipalIdentity,
    >,
{
    DeclaredCoupledOutputDemand {
        reuse_prepared: true,
        connection,
        family: Family::<S, D>::IDENTITY,
        family_type: EqualityTypeId::of::<Family<S, D>>(),
        source_binding_type: EqualityTypeId::of::<Source<S, D>>(),
        demand: Box::new(demand),
        lower_source: source_arguments::lower_demand::<S, D>,
    }
}
