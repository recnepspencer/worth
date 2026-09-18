use std::marker::PhantomData;

use crate::{
    application_operation::ApplicationMutationBinding,
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationSchema},
};

use crate::application_program::action::{
    ApplicationActionInstanceRef, ApplicationActionShape,
    ApplicationConditionalOperationActionInstanceRef, ApplicationOperationActionRef,
};
use crate::application_program::{
    ApplicationActionDeclaration, ApplicationChangeShape, ApplicationCompositionInstance,
    ApplicationDerivedArtifact, ApplicationDerivedArtifactDeclaration,
    ApplicationEvaluatedRequirementRule, ApplicationExternalInputProvider,
    ApplicationLocalityScope, ApplicationManagedComputation,
    ApplicationManagedComputationDeclaration, ApplicationRepeatedOptionalMemberCorrespondence,
    ApplicationRootComposition,
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
            managed_computations: Vec::new(),
            marker: PhantomData,
        }
    }

    pub fn mutation<Binding>(mut self) -> Self
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        self.actions.push(ApplicationActionInstanceRef::<
            Schema,
            Instance,
            Feature,
            Binding,
        >::declaration());
        self
    }

    pub fn mutation_with_requirement<Binding, Rule>(mut self) -> Self
    where
        Binding: ApplicationMutationBinding<Schema>,
        Rule: ApplicationEvaluatedRequirementRule<Schema, Binding::Operation>,
    {
        let mut action =
            ApplicationActionInstanceRef::<Schema, Instance, Feature, Binding>::declaration();
        action.attach_evaluated_requirement::<Rule>(Rule::IDENTITY);
        self.actions.push(action);
        self
    }

    pub fn mutation_with_requirement_and_external_input<Binding, Rule, Provider>(mut self) -> Self
    where
        Binding: ApplicationMutationBinding<Schema>,
        Rule: ApplicationEvaluatedRequirementRule<Schema, Binding::Operation>,
        Provider: ApplicationExternalInputProvider<Schema, Binding::Operation>,
    {
        let mut action =
            ApplicationActionInstanceRef::<Schema, Instance, Feature, Binding>::declaration();
        action.attach_evaluated_requirement::<Rule>(Rule::IDENTITY);
        action.attach_external_input::<Provider>(Provider::IDENTITY);
        self.actions.push(action);
        self
    }

    pub fn required_output_mutation<Binding>(mut self) -> Self
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        let mut action =
            ApplicationActionInstanceRef::<Schema, Instance, Feature, Binding>::declaration();
        action.mark_required_output_source();
        self.actions.push(action);
        self
    }

    pub fn mutation_with_locality_and_change<Binding, Scope, Shape>(mut self) -> Self
    where
        Binding: ApplicationMutationBinding<Schema>,
        Scope: ApplicationLocalityScope,
        Shape: ApplicationChangeShape,
    {
        let mut action =
            ApplicationActionInstanceRef::<Schema, Instance, Feature, Binding>::declaration();
        action.attach_locality_and_change::<Scope, Shape>();
        self.actions.push(action);
        self
    }

    pub fn required_output_mutation_with_locality_and_change<Binding, Scope, Shape>(
        mut self,
    ) -> Self
    where
        Binding: ApplicationMutationBinding<Schema>,
        Scope: ApplicationLocalityScope,
        Shape: ApplicationChangeShape,
    {
        let mut action =
            ApplicationActionInstanceRef::<Schema, Instance, Feature, Binding>::declaration();
        action.attach_locality_and_change::<Scope, Shape>();
        action.mark_required_output_source();
        self.actions.push(action);
        self
    }

    pub fn repeated_optional_member<Binding, Correspondence>(mut self) -> Self
    where
        Binding: ApplicationMutationBinding<Schema>,
        Correspondence: ApplicationRepeatedOptionalMemberCorrespondence<Schema, Binding>,
    {
        let mut action =
            ApplicationActionInstanceRef::<Schema, Instance, Feature, Binding>::declaration();
        action.attach_correspondence::<Correspondence>(Correspondence::IDENTITY);
        self.actions.push(action);
        self
    }

    pub fn repeated_optional_member_with_external_input<Binding, Correspondence, Provider>(
        mut self,
    ) -> Self
    where
        Binding: ApplicationMutationBinding<Schema>,
        Correspondence: ApplicationRepeatedOptionalMemberCorrespondence<Schema, Binding>,
        Provider: ApplicationExternalInputProvider<Schema, Binding::Operation>,
    {
        let mut action =
            ApplicationActionInstanceRef::<Schema, Instance, Feature, Binding>::declaration();
        action.attach_correspondence::<Correspondence>(Correspondence::IDENTITY);
        action.attach_external_input::<Provider>(Provider::IDENTITY);
        self.actions.push(action);
        self
    }

    pub fn repeated_optional_member_output<Binding, Correspondence>(mut self) -> Self
    where
        Binding: ApplicationMutationBinding<Schema>,
        Correspondence: ApplicationRepeatedOptionalMemberCorrespondence<Schema, Binding>,
    {
        let mut action =
            ApplicationActionInstanceRef::<Schema, Instance, Feature, Binding>::declaration();
        action.attach_correspondence::<Correspondence>(Correspondence::IDENTITY);
        action.mark_required_output_source();
        self.actions.push(action);
        self
    }

    pub fn repeated_optional_member_output_with_locality_and_change<
        Binding,
        Correspondence,
        Scope,
        Shape,
    >(
        mut self,
    ) -> Self
    where
        Binding: ApplicationMutationBinding<Schema>,
        Correspondence: ApplicationRepeatedOptionalMemberCorrespondence<Schema, Binding>,
        Scope: ApplicationLocalityScope,
        Shape: ApplicationChangeShape,
    {
        let mut action =
            ApplicationActionInstanceRef::<Schema, Instance, Feature, Binding>::declaration();
        action.attach_correspondence::<Correspondence>(Correspondence::IDENTITY);
        action.attach_locality_and_change::<Scope, Shape>();
        action.mark_required_output_source();
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

    pub fn conditional_operation_with_requirement<Operation, Rule>(mut self) -> Self
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Rule: ApplicationEvaluatedRequirementRule<Schema, Operation>,
    {
        let mut action = ApplicationConditionalOperationActionInstanceRef::<
            Schema,
            Instance,
            Feature,
            Operation,
        >::declaration();
        action.attach_evaluated_requirement::<Rule>(Rule::IDENTITY);
        self.actions.push(action);
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

    pub fn finish(mut self) -> ApplicationFeatureSpec {
        self.feature.set_outputs(self.outputs);
        self.feature.set_derived_artifacts(self.derived_artifacts);
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
