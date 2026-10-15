//! Exact declaration-time action occurrence; this grants no permission.
use super::super::{
    ApplicationCompositionInstance, ApplicationFeature, ApplicationFixedActionDelivery,
};
use super::{ApplicationActionDeclaration, ApplicationActionInstanceRef, ApplicationActionShape};
use crate::{
    application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema,
};
use std::marker::PhantomData;

pub struct ApplicationActionReference<Schema, Instance, Feature, Binding, Delivery> {
    marker: PhantomData<fn() -> (Schema, Instance, Feature, Binding, Delivery)>,
}
mod sealed {
    pub trait Action {}
}
pub trait ApplicationFixedAction<Schema, Binding>: sealed::Action + 'static
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    type Delivery: ApplicationFixedActionDelivery<Schema, Binding>;
    fn declaration() -> ApplicationActionDeclaration;
}
impl<Schema, Instance, Feature, Binding, Delivery> sealed::Action
    for ApplicationActionReference<Schema, Instance, Feature, Binding, Delivery>
{
}
impl<
        Schema: ApplicationSchema,
        Instance: ApplicationCompositionInstance,
        Feature: ApplicationFeature<Schema>,
        Binding: ApplicationMutationBinding<Schema>,
        Delivery: ApplicationFixedActionDelivery<Schema, Binding>,
    > ApplicationFixedAction<Schema, Binding>
    for ApplicationActionReference<Schema, Instance, Feature, Binding, Delivery>
{
    type Delivery = Delivery;
    fn declaration() -> ApplicationActionDeclaration {
        let mut action =
            ApplicationActionInstanceRef::<Schema, Instance, Feature, Binding>::declaration();
        action.mark_required_output_source();
        action.attach_fixed_delivery::<Schema, Binding, Delivery>();
        action
    }
}

/// References one declared action occurrence and its concrete delivery implementation.
pub trait ApplicationAction<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>:
    sealed::Action + 'static
{
    type Delivery: super::ApplicationActionDelivery<Schema, Binding>;
    fn declaration() -> ApplicationActionDeclaration;
}
impl<Schema, Instance, Feature, Binding, Delivery> ApplicationAction<Schema, Binding>
    for ApplicationActionReference<Schema, Instance, Feature, Binding, Delivery>
where
    Schema: ApplicationSchema,
    Instance: ApplicationCompositionInstance,
    Feature: ApplicationFeature<Schema>,
    Binding: ApplicationMutationBinding<Schema>,
    Delivery: super::ApplicationActionDelivery<Schema, Binding>,
{
    type Delivery = Delivery;
    fn declaration() -> ApplicationActionDeclaration {
        let mut action =
            ApplicationActionInstanceRef::<Schema, Instance, Feature, Binding>::declaration();
        action.attach_delivery::<Schema, Binding, Delivery>();
        action
    }
}

/// Sealed typed occurrence whose declared output root is genuinely discovered.
pub trait ApplicationDiscoveredAction<S, B>: ApplicationAction<S, B>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    Self::Delivery: super::ApplicationDiscoveredActionDelivery<S, B>,
{
}
impl<S, I, F, B, D> ApplicationDiscoveredAction<S, B> for ApplicationActionReference<S, I, F, B, D>
where
    S: ApplicationSchema,
    I: ApplicationCompositionInstance,
    F: ApplicationFeature<S>,
    B: ApplicationMutationBinding<S>,
    D: super::ApplicationDiscoveredActionDelivery<S, B>,
{
}
