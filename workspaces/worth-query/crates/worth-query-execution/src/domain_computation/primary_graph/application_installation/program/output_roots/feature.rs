//! Required-source visitation follows the same concrete feature delivery roots.
use super::*;
use std::any::TypeId;
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_program::{
        ApplicationActionDelivery, ApplicationActionOutput, ApplicationActionOutputRoots,
        ApplicationFeatureOutputs,
    },
    application_schema::ApplicationSchema,
};

pub trait WorthQueryFeatureOutputMembers<S: ApplicationSchema> {
    fn append_distinct_required(seen: &mut Vec<TypeId>, bindings: &mut impl FnMut(TypeId));
}
impl<S: ApplicationSchema> WorthQueryFeatureOutputMembers<S>
    for worth_query_declaration::facade::application_program::ApplicationNoOutputGraph
{
    fn append_distinct_required(_: &mut Vec<TypeId>, _: &mut impl FnMut(TypeId)) {}
}
impl<S, B, D> WorthQueryFeatureOutputMembers<S> for ApplicationActionOutput<B, D>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    D: ApplicationActionDelivery<S, B>,
    D::Mode: ApplicationActionOutputRoots<S>,
    <D::Mode as ApplicationActionOutputRoots<S>>::Roots: WorthQueryApplicationProgramRoots<S>,
{
    fn append_distinct_required(seen: &mut Vec<TypeId>, bindings: &mut impl FnMut(TypeId)) {
        type Roots<S, B, D> = <<D as ApplicationActionDelivery<S, B>>::Mode as ApplicationActionOutputRoots<S>>::Roots;
        let identity = TypeId::of::<Roots<S, B, D>>();
        if !seen.contains(&identity) {
            seen.push(identity);
            Roots::<S, B, D>::append_required_bindings(bindings);
        }
    }
}
impl<S, A, B> WorthQueryFeatureOutputMembers<S> for (A, B)
where
    S: ApplicationSchema,
    A: WorthQueryFeatureOutputMembers<S>,
    B: WorthQueryFeatureOutputMembers<S>,
{
    fn append_distinct_required(seen: &mut Vec<TypeId>, bindings: &mut impl FnMut(TypeId)) {
        A::append_distinct_required(seen, bindings);
        B::append_distinct_required(seen, bindings);
    }
}
impl<S, M> WorthQueryApplicationProgramRoots<S> for ApplicationFeatureOutputs<M>
where
    S: ApplicationSchema,
    M: WorthQueryFeatureOutputMembers<S>,
{
    fn append_required_bindings(bindings: &mut impl FnMut(TypeId)) {
        M::append_distinct_required(&mut Vec::new(), bindings);
    }
}
