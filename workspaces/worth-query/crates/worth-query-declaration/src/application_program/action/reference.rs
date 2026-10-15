//! Typed occurrence selection grants neither complete installed meaning nor permission.
use super::super::{ApplicationCompositionInstance, ApplicationFeature};
use super::{
    ApplicationActionDelivery, ApplicationActionDeliveryDeclaration, ApplicationActionDeliveryMode,
};
use crate::{
    application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema,
};
use std::{any::TypeId, marker::PhantomData};

pub struct ApplicationActionReference<Schema, Instance, Feature, Binding, Delivery> {
    marker: PhantomData<fn() -> (Schema, Instance, Feature, Binding, Delivery)>,
}

/// Selection facts only; the installed program owns the complete action declaration.
/// There is no public constructor or conversion to an action declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationActionSelector {
    composition_instance: &'static str,
    feature: &'static str,
    binding: &'static str,
    mutation_binding_type: TypeId,
    delivery: ApplicationActionDeliveryDeclaration,
}
impl ApplicationActionSelector {
    pub const fn composition_instance(&self) -> &'static str {
        self.composition_instance
    }
    pub const fn feature(&self) -> &'static str {
        self.feature
    }
    pub const fn binding(&self) -> &'static str {
        self.binding
    }
    pub const fn mutation_binding_type(&self) -> TypeId {
        self.mutation_binding_type
    }
    pub const fn delivery(&self) -> &ApplicationActionDeliveryDeclaration {
        &self.delivery
    }
}
mod sealed {
    pub trait Action {}
}

/// References one action occurrence and its concrete delivery implementation.
///
/// A selector cannot manufacture complete installed meaning.
/// ```compile_fail,E0308
/// use worth_query_declaration::facade::{application_program::*, application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema};
/// fn partial<S: ApplicationSchema, B: ApplicationMutationBinding<S>, A: ApplicationAction<S, B>>() -> ApplicationActionDeclaration {
///     A::selector()
/// }
/// ```
/// The same typed reference can select its installed occurrence.
/// ```
/// use worth_query_declaration::facade::{application_program::*, application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema};
/// fn select<S: ApplicationSchema, B: ApplicationMutationBinding<S>, A: ApplicationAction<S, B>>() -> ApplicationActionSelector {
///     A::selector()
/// }
/// ```
pub trait ApplicationAction<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>:
    sealed::Action + 'static
{
    type Delivery: ApplicationActionDelivery<Schema, Binding>;
    fn selector() -> ApplicationActionSelector;
}
impl<Schema, Instance, Feature, Binding, Delivery> sealed::Action
    for ApplicationActionReference<Schema, Instance, Feature, Binding, Delivery>
{
}
impl<Schema, Instance, Feature, Binding, Delivery> ApplicationAction<Schema, Binding>
    for ApplicationActionReference<Schema, Instance, Feature, Binding, Delivery>
where
    Schema: ApplicationSchema,
    Instance: ApplicationCompositionInstance,
    Feature: ApplicationFeature<Schema>,
    Binding: ApplicationMutationBinding<Schema>,
    Delivery: ApplicationActionDelivery<Schema, Binding>,
{
    type Delivery = Delivery;
    fn selector() -> ApplicationActionSelector {
        ApplicationActionSelector {
            composition_instance: Instance::PATH,
            feature: Feature::IDENTITY,
            binding: Binding::IDENTITY,
            mutation_binding_type: TypeId::of::<Binding>(),
            delivery: <Delivery::Mode as ApplicationActionDeliveryMode<
                Schema, Binding, Delivery,
            >>::declaration(),
        }
    }
}
