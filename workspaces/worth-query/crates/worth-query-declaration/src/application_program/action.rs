use std::any::TypeId;
use std::marker::PhantomData;

use crate::application_operation::ApplicationMutationBinding;
use crate::application_schema::{
    ApplicationOperationMarkerIdentity, ApplicationSchema, ApplicationStructuredValueBinding,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;

use super::{
    ApplicationChangeShape, ApplicationChangeShapeDeclaration, ApplicationCompositionInstance,
    ApplicationFeature, ApplicationLocalityDeclaration, ApplicationLocalityScope,
    ApplicationRootComposition,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationActionCorrespondenceDeclaration {
    identity: &'static str,
    correspondence_type: TypeId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationActionEvaluatedRequirementDeclaration {
    identity: &'static str,
    rule_type: TypeId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationActionExternalInputDeclaration {
    identity: &'static str,
    provider_type: TypeId,
}

impl ApplicationActionExternalInputDeclaration {
    pub const fn identity(&self) -> &'static str {
        self.identity
    }
    pub const fn provider_type(&self) -> TypeId {
        self.provider_type
    }
}

impl ApplicationActionEvaluatedRequirementDeclaration {
    pub const fn identity(&self) -> &'static str {
        self.identity
    }

    pub const fn rule_type(&self) -> TypeId {
        self.rule_type
    }
}

impl ApplicationActionCorrespondenceDeclaration {
    pub const fn identity(&self) -> &'static str {
        self.identity
    }

    pub const fn correspondence_type(&self) -> TypeId {
        self.correspondence_type
    }
}

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
    required_output_source: bool,
    correspondence: Option<ApplicationActionCorrespondenceDeclaration>,
    evaluated_requirement: Option<ApplicationActionEvaluatedRequirementDeclaration>,
    external_input: Option<ApplicationActionExternalInputDeclaration>,
    locality: Option<ApplicationLocalityDeclaration>,
    change_shape: Option<ApplicationChangeShapeDeclaration>,
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

    pub const fn required_output_source(&self) -> bool {
        self.required_output_source
    }

    pub const fn correspondence(&self) -> Option<&ApplicationActionCorrespondenceDeclaration> {
        self.correspondence.as_ref()
    }

    pub const fn evaluated_requirement(
        &self,
    ) -> Option<&ApplicationActionEvaluatedRequirementDeclaration> {
        self.evaluated_requirement.as_ref()
    }

    pub const fn external_input(&self) -> Option<&ApplicationActionExternalInputDeclaration> {
        self.external_input.as_ref()
    }

    pub const fn locality(&self) -> Option<&ApplicationLocalityDeclaration> {
        self.locality.as_ref()
    }

    pub const fn change_shape(&self) -> Option<&ApplicationChangeShapeDeclaration> {
        self.change_shape.as_ref()
    }

    pub(crate) fn attach_correspondence<Correspondence: 'static>(
        &mut self,
        identity: &'static str,
    ) {
        self.correspondence = Some(ApplicationActionCorrespondenceDeclaration {
            identity,
            correspondence_type: TypeId::of::<Correspondence>(),
        });
    }

    pub(crate) fn mark_required_output_source(&mut self) {
        self.required_output_source = true;
    }

    pub(crate) fn attach_evaluated_requirement<Rule: 'static>(&mut self, identity: &'static str) {
        self.evaluated_requirement = Some(ApplicationActionEvaluatedRequirementDeclaration {
            identity,
            rule_type: TypeId::of::<Rule>(),
        });
    }

    pub(crate) fn attach_external_input<Provider: 'static>(&mut self, identity: &'static str) {
        self.external_input = Some(ApplicationActionExternalInputDeclaration {
            identity,
            provider_type: TypeId::of::<Provider>(),
        });
    }

    pub(crate) fn attach_locality_and_change<Scope, Shape>(&mut self)
    where
        Scope: ApplicationLocalityScope,
        Shape: ApplicationChangeShape,
    {
        self.locality = Some(ApplicationLocalityDeclaration::of::<Scope>());
        self.change_shape = Some(ApplicationChangeShapeDeclaration::of::<Shape>());
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
            required_output_source: false,
            correspondence: None,
            evaluated_requirement: None,
            external_input: None,
            locality: None,
            change_shape: None,
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
            required_output_source: false,
            correspondence: None,
            evaluated_requirement: None,
            external_input: None,
            locality: None,
            change_shape: None,
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
