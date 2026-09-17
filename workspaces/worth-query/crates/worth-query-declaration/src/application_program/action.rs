use std::any::TypeId;
use std::marker::PhantomData;

use crate::application_operation::ApplicationMutationBinding;
use crate::application_schema::{
    ApplicationOperationMarkerIdentity, ApplicationSchema, ApplicationStructuredValueBinding,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;

use super::{ApplicationCompositionInstance, ApplicationFeature, ApplicationRootComposition};

/// One program-owned mutation binding attached to an explicit nested instance.
pub(crate) struct ApplicationActionInstanceRef<Schema, Instance, Feature, Binding> {
    marker: PhantomData<fn() -> (Schema, Instance, Feature, Binding)>,
}

/// One program-owned specialized operation whose effects are issued by Query.
pub(crate) struct ApplicationOperationActionRef<Schema, Feature, Operation> {
    marker: PhantomData<fn() -> (Schema, Feature, Operation)>,
}

/// A program operation issued only by its installed conditional owner.
pub(crate) struct ApplicationConditionalOperationActionRef<Schema, Feature, Operation> {
    marker: PhantomData<fn() -> (Schema, Feature, Operation)>,
}

/// Conditional operation owned by a feature in a named composition instance.
pub(crate) struct ApplicationConditionalOperationActionInstanceRef<
    Schema,
    Instance,
    Feature,
    Operation,
> {
    marker: PhantomData<fn() -> (Schema, Instance, Feature, Operation)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationActionDeclaration {
    composition_instance: &'static str,
    feature: &'static str,
    binding: &'static str,
    action_type: TypeId,
    mutation_binding_type: Option<TypeId>,
    operation_type: TypeId,
    operation_input_type: TypeId,
    operation_input_identity: WorthQueryPortableTypeIdentity,
    conditional_only: bool,
}

impl ApplicationActionDeclaration {
    pub const fn composition_instance(&self) -> &'static str {
        self.composition_instance
    }

    pub const fn feature(&self) -> &'static str {
        self.feature
    }

    pub const fn binding(&self) -> &'static str {
        self.binding
    }

    pub const fn action_type(&self) -> TypeId {
        self.action_type
    }

    pub const fn mutation_binding_type(&self) -> Option<TypeId> {
        self.mutation_binding_type
    }

    pub const fn operation_type(&self) -> TypeId {
        self.operation_type
    }

    pub const fn operation_input_type(&self) -> TypeId {
        self.operation_input_type
    }

    pub const fn operation_input_identity(&self) -> &WorthQueryPortableTypeIdentity {
        &self.operation_input_identity
    }

    pub const fn conditional_only(&self) -> bool {
        self.conditional_only
    }
}

mod sealed {
    pub trait ActionShape {}
}

pub(crate) trait ApplicationActionShape<Schema>:
    sealed::ActionShape + Sized + 'static
where
    Schema: ApplicationSchema,
{
    fn declaration() -> ApplicationActionDeclaration;
}

impl<Schema, Instance, Feature, Binding> sealed::ActionShape
    for ApplicationActionInstanceRef<Schema, Instance, Feature, Binding>
{
}

impl<Schema, Instance, Feature, Binding> ApplicationActionShape<Schema>
    for ApplicationActionInstanceRef<Schema, Instance, Feature, Binding>
where
    Schema: ApplicationSchema,
    Instance: ApplicationCompositionInstance,
    Feature: ApplicationFeature<Schema>,
    Binding: ApplicationMutationBinding<Schema>,
{
    fn declaration() -> ApplicationActionDeclaration {
        ApplicationActionDeclaration {
            composition_instance: Instance::PATH,
            feature: Feature::IDENTITY,
            binding: Binding::IDENTITY,
            action_type: TypeId::of::<Binding>(),
            mutation_binding_type: Some(TypeId::of::<Binding>()),
            operation_type: TypeId::of::<Binding::Operation>(),
            operation_input_type: TypeId::of::<Binding::Input>(),
            operation_input_identity:
                <Binding::InputBinding as ApplicationStructuredValueBinding>::IDENTITY,
            conditional_only: false,
        }
    }
}

impl<Schema, Feature, Operation> sealed::ActionShape
    for ApplicationOperationActionRef<Schema, Feature, Operation>
{
}

impl<Schema, Feature, Operation> ApplicationActionShape<Schema>
    for ApplicationOperationActionRef<Schema, Feature, Operation>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
{
    fn declaration() -> ApplicationActionDeclaration {
        ApplicationActionDeclaration {
            composition_instance: ApplicationRootComposition::PATH,
            feature: Feature::IDENTITY,
            binding: Operation::IDENTIFIER,
            action_type: TypeId::of::<Operation>(),
            mutation_binding_type: None,
            operation_type: TypeId::of::<Operation>(),
            operation_input_type: TypeId::of::<
                <Operation::InputBinding as ApplicationStructuredValueBinding>::Value,
            >(),
            operation_input_identity: Operation::InputBinding::IDENTITY,
            conditional_only: false,
        }
    }
}

impl<Schema, Feature, Operation> sealed::ActionShape
    for ApplicationConditionalOperationActionRef<Schema, Feature, Operation>
{
}

impl<Schema, Feature, Operation> ApplicationActionShape<Schema>
    for ApplicationConditionalOperationActionRef<Schema, Feature, Operation>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
{
    fn declaration() -> ApplicationActionDeclaration {
        let mut action = ApplicationOperationActionRef::<Schema, Feature, Operation>::declaration();
        action.conditional_only = true;
        action
    }
}

impl<Schema, Instance, Feature, Operation> sealed::ActionShape
    for ApplicationConditionalOperationActionInstanceRef<Schema, Instance, Feature, Operation>
{
}

impl<Schema, Instance, Feature, Operation> ApplicationActionShape<Schema>
    for ApplicationConditionalOperationActionInstanceRef<Schema, Instance, Feature, Operation>
where
    Schema: ApplicationSchema,
    Instance: ApplicationCompositionInstance,
    Feature: ApplicationFeature<Schema>,
    Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
{
    fn declaration() -> ApplicationActionDeclaration {
        let mut action =
            ApplicationConditionalOperationActionRef::<Schema, Feature, Operation>::declaration();
        action.composition_instance = Instance::PATH;
        action
    }
}
