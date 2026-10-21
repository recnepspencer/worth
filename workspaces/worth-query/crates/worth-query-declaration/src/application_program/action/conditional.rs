//! Conditional ownership over the existing native binding and operation action shapes.
use super::*;

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

/// A concrete native binding issued only by its installed conditional owner.
pub(crate) struct ApplicationConditionalNativeActionInstanceRef<Schema, Instance, Feature, Binding>
{
    marker: PhantomData<fn() -> (Schema, Instance, Feature, Binding)>,
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

impl<Schema, Instance, Feature, Binding> sealed::ActionShape
    for ApplicationConditionalNativeActionInstanceRef<Schema, Instance, Feature, Binding>
{
}

impl<Schema, Instance, Feature, Binding> ApplicationActionShape<Schema>
    for ApplicationConditionalNativeActionInstanceRef<Schema, Instance, Feature, Binding>
where
    Schema: ApplicationSchema,
    Instance: ApplicationCompositionInstance,
    Feature: ApplicationFeature<Schema>,
    Binding: ApplicationMutationBinding<Schema>,
{
    fn declaration() -> ApplicationActionDeclaration {
        let mut action =
            ApplicationActionInstanceRef::<Schema, Instance, Feature, Binding>::declaration();
        action.conditional_only = true;
        action
    }
}
