//! Pure discovered delivery binds an actual graph and concrete selector.
use super::super::{ApplicationDiscoveredOutputRoot, ApplicationOutputGraphShape};
use crate::{
    application_operation::ApplicationMutationBinding, application_query::ApplicationQueryBinding,
    application_schema::ApplicationSchema,
};
/// Installed discovered meaning grants no execution or read authority.
/// The root must be a discovered graph, rather than a fixed output graph.
/// Its closed projection shape is shared with fixed delivery.
pub trait ApplicationDiscoveredActionDelivery<S, B>:
    super::ApplicationActionDelivery<
    S,
    B,
    Mode = super::ApplicationDiscoveredDeliveryMode<Self::Root, Self::Projection>,
>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
{
    type Root: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot;
    type Projection: super::ApplicationOutputProjection<S>;
}
/// One concrete declared query selected purely from the original semantic input.
pub trait ApplicationDiscoveredQueryDelivery<S, B>:
    ApplicationDiscoveredActionDelivery<
    S,
    B,
    Projection = super::ApplicationQueryProjection<Self::Read>,
>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
{
    type Read: ApplicationQueryBinding<S>;
    fn read_input(input: &B::Input) -> <Self::Read as ApplicationQueryBinding<S>>::Input;
}
