//! Consuming authoring of one complete action on its feature occurrence.
mod contracts;

use std::marker::PhantomData;

use crate::{
    application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema,
};

use super::super::{
    ApplicationCompositionInstance, ApplicationFeature, ApplicationFeatureSpecBuilder,
};
use super::{
    ApplicationActionDeclaration, ApplicationActionDelivery, ApplicationActionInstanceRef,
    ApplicationActionShape,
};

/// One unfinished action. Only `finish_action` restores feature authoring.
///
/// The contract states are inferred from consuming attachments. An unfinished
/// action cannot append itself or expose mutable canonical meaning.
///
/// A valid action composes every optional clause and returns its feature parent.
///
/// ```no_run
/// use worth_query_declaration::facade::{application_program::*, application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema};
/// fn complete<S, I, F, B, D, L, C, R, P, H, Port>(parent: ApplicationFeatureSpecBuilder<S, I, F>) -> ApplicationFeatureSpec
/// where S: ApplicationSchema + 'static, I: ApplicationCompositionInstance, F: ApplicationFeature<S>,
/// B: ApplicationMutationBinding<S>, D: ApplicationActionDelivery<S, B>,
/// L: ApplicationLocalityScope, H: ApplicationChangeShape,
/// C: ApplicationRepeatedOptionalMemberCorrespondence<S, B>,
/// R: ApplicationEvaluatedRequirementRule<S, B::Operation>, P: ApplicationExternalInputProvider<S, B::Operation>,
/// Port: ApplicationOutputPort<S, F> {
///     parent.action::<B, D>().locality_and_change::<L, H>().correspondence::<C>()
///         .evaluated_requirement::<R>().external_input::<P>().finish_action().provides::<Port>().finish()
/// }
/// ```
///
/// The locality/change pair is single-valued.
///
/// ```compile_fail,E0599
/// use worth_query_declaration::facade::{application_program::*, application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema};
/// fn rejected<S, I, F, B, L, H>(action: ApplicationActionSpecBuilder<S, I, F, B>)
/// where S: ApplicationSchema + 'static, I: ApplicationCompositionInstance, F: ApplicationFeature<S>,
/// B: ApplicationMutationBinding<S>, L: ApplicationLocalityScope, H: ApplicationChangeShape {
///     action.locality_and_change::<L, H>().locality_and_change::<L, H>();
/// }
/// ```
///
/// Correspondence cannot be replaced.
///
/// ```compile_fail,E0599
/// use worth_query_declaration::facade::{application_program::*, application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema};
/// fn rejected<S, I, F, B, C>(action: ApplicationActionSpecBuilder<S, I, F, B>)
/// where S: ApplicationSchema + 'static, I: ApplicationCompositionInstance, F: ApplicationFeature<S>,
/// B: ApplicationMutationBinding<S>, C: ApplicationRepeatedOptionalMemberCorrespondence<S, B> {
///     action.correspondence::<C>().correspondence::<C>();
/// }
/// ```
///
/// An evaluated requirement cannot be replaced.
///
/// ```compile_fail,E0599
/// use worth_query_declaration::facade::{application_program::*, application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema};
/// fn rejected<S, I, F, B, R>(action: ApplicationActionSpecBuilder<S, I, F, B>)
/// where S: ApplicationSchema + 'static, I: ApplicationCompositionInstance, F: ApplicationFeature<S>,
/// B: ApplicationMutationBinding<S>, R: ApplicationEvaluatedRequirementRule<S, B::Operation> {
///     action.evaluated_requirement::<R>().evaluated_requirement::<R>();
/// }
/// ```
///
/// An external-input contract cannot be replaced.
///
/// ```compile_fail,E0599
/// use worth_query_declaration::facade::{application_program::*, application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema};
/// fn rejected<S, I, F, B, P>(action: ApplicationActionSpecBuilder<S, I, F, B>)
/// where S: ApplicationSchema + 'static, I: ApplicationCompositionInstance, F: ApplicationFeature<S>,
/// B: ApplicationMutationBinding<S>, P: ApplicationExternalInputProvider<S, B::Operation> {
///     action.external_input::<P>().external_input::<P>();
/// }
/// ```
///
/// Unfinished action authoring cannot finish the feature.
///
/// ```compile_fail,E0599
/// use worth_query_declaration::facade::{application_program::*, application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema};
/// fn rejected<S, I, F, B>(action: ApplicationActionSpecBuilder<S, I, F, B>)
/// where S: ApplicationSchema + 'static, I: ApplicationCompositionInstance, F: ApplicationFeature<S>,
/// B: ApplicationMutationBinding<S> {
///     action.finish();
/// }
/// ```
///
/// Unfinished action authoring cannot provide a feature output.
///
/// ```compile_fail,E0599
/// use worth_query_declaration::facade::{application_program::*, application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema};
/// fn rejected<S, I, F, B, Port>(action: ApplicationActionSpecBuilder<S, I, F, B>)
/// where S: ApplicationSchema + 'static, I: ApplicationCompositionInstance, F: ApplicationFeature<S>,
/// B: ApplicationMutationBinding<S>, Port: ApplicationOutputPort<S, F> {
///     action.provides::<Port>();
/// }
/// ```
///
/// Provider types must implement the actual operation contract.
///
/// ```compile_fail,E0277
/// use worth_query_declaration::facade::{application_program::*, application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema};
/// fn rejected<S, I, F, B>(action: ApplicationActionSpecBuilder<S, I, F, B>)
/// where S: ApplicationSchema + 'static, I: ApplicationCompositionInstance, F: ApplicationFeature<S>,
/// B: ApplicationMutationBinding<S> {
///     action.external_input::<()>();
/// }
/// ```
///
/// Rule types must implement the actual operation contract.
///
/// ```compile_fail,E0277
/// use worth_query_declaration::facade::{application_program::*, application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema};
/// fn rejected<S, I, F, B>(action: ApplicationActionSpecBuilder<S, I, F, B>)
/// where S: ApplicationSchema + 'static, I: ApplicationCompositionInstance, F: ApplicationFeature<S>,
/// B: ApplicationMutationBinding<S> {
///     action.evaluated_requirement::<()>();
/// }
/// ```
///
/// Correspondence types must implement the actual binding contract.
///
/// ```compile_fail,E0277
/// use worth_query_declaration::facade::{application_program::*, application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema};
/// fn rejected<S, I, F, B>(action: ApplicationActionSpecBuilder<S, I, F, B>)
/// where S: ApplicationSchema + 'static, I: ApplicationCompositionInstance, F: ApplicationFeature<S>,
/// B: ApplicationMutationBinding<S> {
///     action.correspondence::<()>();
/// }
/// ```
///
/// Ordinary delivery is mandatory and typed.
///
/// ```compile_fail,E0277
/// use worth_query_declaration::facade::{application_program::*, application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema};
/// fn rejected<S, I, F, B>(parent: ApplicationFeatureSpecBuilder<S, I, F>)
/// where S: ApplicationSchema + 'static, I: ApplicationCompositionInstance, F: ApplicationFeature<S>,
/// B: ApplicationMutationBinding<S> {
///     parent.action::<B, ()>();
/// }
/// ```
///
/// The retired mutation spelling is unavailable.
///
/// ```compile_fail,E0599
/// use worth_query_declaration::facade::{application_program::*, application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema};
/// fn rejected<S, I, F, B>(parent: ApplicationFeatureSpecBuilder<S, I, F>)
/// where S: ApplicationSchema + 'static, I: ApplicationCompositionInstance, F: ApplicationFeature<S>,
/// B: ApplicationMutationBinding<S> {
///     parent.mutation::<B>();
/// }
/// ```
#[must_use = "finish_action must return the feature parent before feature completion"]
pub struct ApplicationActionSpecBuilder<
    Schema,
    Instance,
    Feature,
    Binding,
    Locality = contracts::Unconfigured,
    Correspondence = contracts::Unconfigured,
    Requirement = contracts::Unconfigured,
    ExternalInput = contracts::Unconfigured,
> {
    parent: ApplicationFeatureSpecBuilder<Schema, Instance, Feature>,
    action: ApplicationActionDeclaration,
    marker: PhantomData<
        fn() -> (
            Binding,
            Locality,
            Correspondence,
            Requirement,
            ExternalInput,
        ),
    >,
}

impl<Schema, Instance, Feature, Binding>
    ApplicationActionSpecBuilder<Schema, Instance, Feature, Binding>
where
    Schema: ApplicationSchema + 'static,
    Instance: ApplicationCompositionInstance,
    Feature: ApplicationFeature<Schema>,
    Binding: ApplicationMutationBinding<Schema>,
{
    pub(in crate::application_program) fn ordinary<Delivery>(
        parent: ApplicationFeatureSpecBuilder<Schema, Instance, Feature>,
    ) -> Self
    where
        Delivery: ApplicationActionDelivery<Schema, Binding>,
    {
        let mut builder = Self::native(parent);
        builder
            .action
            .attach_delivery::<Schema, Binding, Delivery>();
        builder
    }

    pub(in crate::application_program) fn native(
        parent: ApplicationFeatureSpecBuilder<Schema, Instance, Feature>,
    ) -> Self {
        Self {
            parent,
            action: ApplicationActionInstanceRef::<Schema, Instance, Feature, Binding>::declaration(
            ),
            marker: PhantomData,
        }
    }
}

impl<Schema, Instance, Feature, Binding, L, C, R, P>
    ApplicationActionSpecBuilder<Schema, Instance, Feature, Binding, L, C, R, P>
where
    Schema: ApplicationSchema + 'static,
    Instance: ApplicationCompositionInstance,
    Feature: ApplicationFeature<Schema>,
    Binding: ApplicationMutationBinding<Schema>,
{
    /// Append exactly this complete declaration and resume its feature parent.
    pub fn finish_action(self) -> ApplicationFeatureSpecBuilder<Schema, Instance, Feature> {
        self.parent.append_action(self.action)
    }

    fn transition<NextL, NextC, NextR, NextP>(
        self,
    ) -> ApplicationActionSpecBuilder<Schema, Instance, Feature, Binding, NextL, NextC, NextR, NextP>
    {
        ApplicationActionSpecBuilder {
            parent: self.parent,
            action: self.action,
            marker: PhantomData,
        }
    }
}
