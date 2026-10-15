//! Installed action delivery is canonical meaning, never request authority.
use super::super::{ApplicationFixedOutputRoot, ApplicationOutputGraphShape};
use crate::{
    application_operation::ApplicationMutationBinding, application_query::ApplicationQueryBinding,
    application_schema::ApplicationSchema,
};

/// The installed declaration binds this concrete implementation in the running process.
/// `IDENTITY` supplies stable canonical meaning; it does not admit another selector type.
pub trait ApplicationFixedActionDelivery<Schema, Binding>:
    super::ApplicationActionDelivery<
    Schema,
    Binding,
    Mode = super::ApplicationFixedDeliveryMode<Self::Root, Self::Projection>,
>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    type Root: ApplicationOutputGraphShape<Schema> + ApplicationFixedOutputRoot;
    type Projection: super::ApplicationOutputProjection<Schema>;
}

/// A projected fixed delivery binds one concrete query and its original-input selector.
pub trait ApplicationFixedQueryDelivery<Schema, Binding>:
    ApplicationFixedActionDelivery<
    Schema,
    Binding,
    Projection = super::ApplicationQueryProjection<Self::Read>,
>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    type Read: ApplicationQueryBinding<Schema>;
    fn read_input(input: &Binding::Input)
        -> <Self::Read as ApplicationQueryBinding<Schema>>::Input;
}
