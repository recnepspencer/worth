use std::marker::PhantomData;

use crate::{
    application_operation::ApplicationMutationBinding,
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationSchema},
};

use super::super::super::{
    ApplicationActionDeclaration, ApplicationActionInstanceRef, ApplicationActionShape,
    ApplicationCompositionInstance, ApplicationConditionalOperationActionInstanceRef,
    ApplicationFeature, ApplicationFeatureDeclaration, ApplicationFeatureInstanceRef,
    ApplicationFeatureShape, ApplicationRootComposition,
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

    pub fn finish(self) -> ApplicationFeatureSpec {
        ApplicationFeatureSpec {
            feature: self.feature,
            actions: self.actions.into_boxed_slice(),
        }
    }
}
