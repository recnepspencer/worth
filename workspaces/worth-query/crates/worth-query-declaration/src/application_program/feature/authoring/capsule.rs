use std::marker::PhantomData;

use crate::{
    application_operation::ApplicationMutationBinding,
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationSchema},
};

use crate::application_program::action::{
    ApplicationActionShape, ApplicationConditionalOperationActionInstanceRef,
    ApplicationOperationActionRef,
};
use crate::application_program::{
    ApplicationActionDeclaration, ApplicationActionDelivery, ApplicationActionSpecBuilder,
    ApplicationCompositionInstance, ApplicationDerivedArtifact,
    ApplicationDerivedArtifactDeclaration, ApplicationDerivedCollection,
    ApplicationDerivedCollectionDeclaration, ApplicationEvaluatedRequirementRule,
    ApplicationExternalInputProvider, ApplicationManagedComputation,
    ApplicationManagedComputationDeclaration, ApplicationRootComposition,
};

use super::super::{
    ApplicationFeature, ApplicationFeatureDeclaration, ApplicationFeatureInstanceRef,
    ApplicationFeatureOutputDeclaration, ApplicationFeatureShape, ApplicationOutputPort,
};

/// One feature-owned contribution to the canonical application program.
///
/// Its private construction keeps feature membership and every attached action
/// on the same typed composition occurrence. It carries no installation or
/// execution authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationFeatureSpec {
    feature: ApplicationFeatureDeclaration,
    actions: Box<[ApplicationActionDeclaration]>,
}

impl ApplicationFeatureSpec {
    pub fn root<Schema, Feature>(
    ) -> ApplicationFeatureSpecBuilder<Schema, ApplicationRootComposition, Feature>
    where
        Schema: ApplicationSchema + 'static,
        Feature: ApplicationFeature<Schema>,
    {
        ApplicationFeatureSpecBuilder::new()
    }

    pub fn at<Schema, Instance, Feature>(
    ) -> ApplicationFeatureSpecBuilder<Schema, Instance, Feature>
    where
        Schema: ApplicationSchema + 'static,
        Instance: ApplicationCompositionInstance,
        Feature: ApplicationFeature<Schema>,
    {
        ApplicationFeatureSpecBuilder::new()
    }

    pub const fn feature(&self) -> &ApplicationFeatureDeclaration {
        &self.feature
    }

    pub fn actions(&self) -> &[ApplicationActionDeclaration] {
        &self.actions
    }

    pub(in crate::application_program) fn into_parts(
        self,
    ) -> (
        ApplicationFeatureDeclaration,
        Box<[ApplicationActionDeclaration]>,
    ) {
        (self.feature, self.actions)
    }
}

/// Typed authoring progression for one feature occurrence.
pub struct ApplicationFeatureSpecBuilder<Schema, Instance, Feature> {
    feature: ApplicationFeatureDeclaration,
    actions: Vec<ApplicationActionDeclaration>,
    outputs: Vec<ApplicationFeatureOutputDeclaration>,
    derived_artifacts: Vec<ApplicationDerivedArtifactDeclaration>,
    derived_collections: Vec<ApplicationDerivedCollectionDeclaration>,
    managed_computations: Vec<ApplicationManagedComputationDeclaration>,
    marker: PhantomData<fn() -> (Schema, Instance, Feature)>,
}

impl<Schema, Instance, Feature> ApplicationFeatureSpecBuilder<Schema, Instance, Feature>
where
    Schema: ApplicationSchema + 'static,
    Instance: ApplicationCompositionInstance,
    Feature: ApplicationFeature<Schema>,
{
    fn new() -> Self {
        Self {
            feature: ApplicationFeatureInstanceRef::<Schema, Instance, Feature>::declaration(),
            actions: Vec::new(),
            outputs: Vec::new(),
            derived_artifacts: Vec::new(),
            derived_collections: Vec::new(),
            managed_computations: Vec::new(),
            marker: PhantomData,
        }
    }

    /// Begin one ordinary action with explicit owning delivery.
    pub fn action<Binding, Delivery>(
        self,
    ) -> ApplicationActionSpecBuilder<Schema, Instance, Feature, Binding>
    where
        Binding: ApplicationMutationBinding<Schema>,
        Delivery: ApplicationActionDelivery<Schema, Binding>,
    {
        ApplicationActionSpecBuilder::ordinary::<Delivery>(self)
    }

    /// Begin a deliberately specialized native action, without owning delivery.
    /// Required native output membership is derived from declared program roots.
    pub fn native_action<Binding>(
        self,
    ) -> ApplicationActionSpecBuilder<Schema, Instance, Feature, Binding>
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        ApplicationActionSpecBuilder::native(self)
    }

    /// Begin a concrete native binding owned exclusively by its installed conditional.
    /// Binding membership, operation/input equality and workflow requirements come
    /// from `Binding`; this declaration supplies no ordinary action delivery.
    pub fn conditional_native_action<Binding>(
        self,
    ) -> ApplicationActionSpecBuilder<Schema, Instance, Feature, Binding>
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        ApplicationActionSpecBuilder::conditional_native(self)
    }

    pub(in crate::application_program) fn append_action(
        mut self,
        action: ApplicationActionDeclaration,
    ) -> Self {
        self.actions.push(action);
        self
    }

    pub fn conditional_operation<Operation>(mut self) -> Self
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
    {
        self.actions
            .push(ApplicationConditionalOperationActionInstanceRef::<
                Schema,
                Instance,
                Feature,
                Operation,
            >::declaration());
        self
    }

    pub fn conditional_operation_with_requirement_and_external_input<Operation, Rule, Provider>(
        mut self,
    ) -> Self
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Rule: ApplicationEvaluatedRequirementRule<Schema, Operation>,
        Provider: ApplicationExternalInputProvider<Schema, Operation>,
    {
        let mut action = ApplicationConditionalOperationActionInstanceRef::<
            Schema,
            Instance,
            Feature,
            Operation,
        >::declaration();
        action.attach_evaluated_requirement::<Rule>(Rule::IDENTITY);
        action.attach_external_input::<Provider>(Provider::IDENTITY);
        self.actions.push(action);
        self
    }

    pub fn provides<Port>(mut self) -> Self
    where
        Port: ApplicationOutputPort<Schema, Feature>,
    {
        self.outputs
            .push(ApplicationFeatureOutputDeclaration::new(Port::IDENTITY));
        self
    }

    pub fn derived_artifact<Artifact>(mut self) -> Self
    where
        Artifact: ApplicationDerivedArtifact<Schema, Feature>,
    {
        let artifact = ApplicationDerivedArtifactDeclaration::of::<Schema, Feature, Artifact>();
        self.outputs
            .push(ApplicationFeatureOutputDeclaration::new(artifact.output()));
        self.derived_artifacts.push(artifact);
        self
    }

    pub fn managed_computation<Computation>(mut self) -> Self
    where
        Computation: ApplicationManagedComputation<Schema, Feature>,
    {
        self.managed_computations
            .push(ApplicationManagedComputationDeclaration::of::<
                Schema,
                Feature,
                Computation,
            >());
        self
    }

    pub fn derived_collection<Collection>(mut self) -> Self
    where
        Collection: ApplicationDerivedCollection<Schema, Feature>,
    {
        self.derived_collections
            .push(ApplicationDerivedCollectionDeclaration::of::<
                Schema,
                Feature,
                Collection,
            >());
        self
    }

    pub fn finish(mut self) -> ApplicationFeatureSpec {
        self.feature.set_outputs(self.outputs);
        self.feature.set_derived_artifacts(self.derived_artifacts);
        self.feature
            .set_derived_collections(self.derived_collections);
        self.feature
            .set_managed_computations(self.managed_computations);
        ApplicationFeatureSpec {
            feature: self.feature,
            actions: self.actions.into_boxed_slice(),
        }
    }
}

impl<Schema, Feature> ApplicationFeatureSpecBuilder<Schema, ApplicationRootComposition, Feature>
where
    Schema: ApplicationSchema + 'static,
    Feature: ApplicationFeature<Schema>,
{
    pub fn operation<Operation>(mut self) -> Self
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
    {
        self.actions
            .push(ApplicationOperationActionRef::<Schema, Feature, Operation>::declaration());
        self
    }
}
