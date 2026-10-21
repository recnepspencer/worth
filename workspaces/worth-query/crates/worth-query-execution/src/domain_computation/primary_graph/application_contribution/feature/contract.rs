//! Producer associated types own both lifecycle slots and readiness correspondence.
use super::super::super::{
    WorthQueryPrimaryGraphInstallationDenial as Denial,
    WorthQueryPrimaryGraphInstallationDenialKind as DenialKind,
};
use super::super::{
    WorthQueryApplicationConditionalBinding as Conditional,
    WorthQueryApplicationProducerBinding as Producer, WorthQueryProducerLifecyclePosture,
    WorthQueryProducerOutputFamily,
};
use std::marker::PhantomData;
use worth_query_declaration::facade::{
    application_operation::{ApplicationMutationBinding, WorthQueryApplicationOutputRole},
    application_program::{
        ApplicationFeature, ApplicationRequiredFeatureOutput,
        ApplicationRequiredFeatureRoleDeclaration as Role,
        ApplicationRequiredFeatureRoleKind as Kind, ApplicationRequiredFeatureRoles,
    },
    application_query::ApplicationQueryBinding,
    application_schema::ApplicationSchema,
};

/// One required Initial/Preserve declaration. It contains no runtime values.
pub struct WorthQueryRequiredFeaturePair<
    Schema,
    Feature,
    Initial,
    InitialReadiness,
    Preserve,
    PreserveReadiness,
>(
    PhantomData<
        fn() -> (
            Schema,
            Feature,
            Initial,
            InitialReadiness,
            Preserve,
            PreserveReadiness,
        ),
    >,
);

/// Host projections retain the exact producer/readiness types of the Decl pair.
pub trait WorthQueryRequiredFeatureContract<Schema>:
    ApplicationRequiredFeatureRoles<Schema>
where
    Schema: ApplicationSchema,
{
    type InitialProducer: Producer<Schema, Operation = Self::InitialMutation>;
    type InitialReadiness: Conditional<
        Schema,
        Operation = <Self::InitialMutation as ApplicationMutationBinding<Schema>>::Operation,
    >;
    type PreserveProducer: Producer<
        Schema,
        Operation = Self::PreserveMutation,
        OutputFamily = <Self::InitialProducer as Producer<Schema>>::OutputFamily,
    >;
    type PreserveReadiness: Conditional<
        Schema,
        Operation = <Self::PreserveMutation as ApplicationMutationBinding<Schema>>::Operation,
    >;
}

impl<S, F, I, IR, P, PR> ApplicationRequiredFeatureRoles<S>
    for WorthQueryRequiredFeaturePair<S, F, I, IR, P, PR>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    I: Producer<S>,
    P: Producer<S, OutputFamily = I::OutputFamily>,
    IR: Conditional<S, Operation = <I::Operation as ApplicationMutationBinding<S>>::Operation>,
    PR: Conditional<S, Operation = <P::Operation as ApplicationMutationBinding<S>>::Operation>,
{
    type Feature = F;
    type InitialMutation = I::Operation;
    type PreserveMutation = P::Operation;
    fn roles() -> [Role; 2] {
        [
            role::<S, I, IR>(Kind::Initial),
            role::<S, P, PR>(Kind::Preserve),
        ]
    }
}

impl<S, F, I, IR, P, PR> WorthQueryRequiredFeatureContract<S>
    for WorthQueryRequiredFeaturePair<S, F, I, IR, P, PR>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    I: Producer<S>,
    P: Producer<S, OutputFamily = I::OutputFamily>,
    IR: Conditional<S, Operation = <I::Operation as ApplicationMutationBinding<S>>::Operation>,
    PR: Conditional<S, Operation = <P::Operation as ApplicationMutationBinding<S>>::Operation>,
{
    type InitialProducer = I;
    type InitialReadiness = IR;
    type PreserveProducer = P;
    type PreserveReadiness = PR;
}

fn role<S, P, R>(kind: Kind) -> Role
where
    S: ApplicationSchema,
    P: Producer<S>,
    R: Conditional<S, Operation = <P::Operation as ApplicationMutationBinding<S>>::Operation>,
{
    Role::of::<S, P::Operation, P, R>(
        kind,
        P::IDENTITY,
        R::IDENTITY,
        ApplicationRequiredFeatureOutput::of::<
            P::OutputFamily,
            <P::OutputFamily as WorthQueryProducerOutputFamily<S>>::Source,
        >(
            P::OutputFamily::IDENTITY,
            <P::OutputFamily as WorthQueryProducerOutputFamily<S>>::Source::IDENTITY,
            <P::OutputRole as WorthQueryApplicationOutputRole>::NAME,
        ),
    )
}

pub(in crate::domain_computation::primary_graph::application_contribution) fn validate_pair<S, C>(
) -> Result<(), Denial>
where
    S: ApplicationSchema,
    C: WorthQueryRequiredFeatureContract<S>,
{
    validate_slot::<S, C::InitialProducer>(Kind::Initial)?;
    validate_slot::<S, C::PreserveProducer>(Kind::Preserve)?;
    let expected = [
        role::<S, C::InitialProducer, C::InitialReadiness>(Kind::Initial),
        role::<S, C::PreserveProducer, C::PreserveReadiness>(Kind::Preserve),
    ];
    if C::roles() != expected || expected[0].output().role() != expected[1].output().role() {
        return Err(Denial::new(
            DenialKind::ProducerBindingMeaningMismatch,
            <C::Feature as ApplicationFeature<S>>::IDENTITY,
        ));
    }
    Ok(())
}

fn validate_slot<S, P>(kind: Kind) -> Result<(), Denial>
where
    S: ApplicationSchema,
    P: Producer<S>,
{
    let expected = match kind {
        Kind::Initial => WorthQueryProducerLifecyclePosture::Initial,
        Kind::Preserve => WorthQueryProducerLifecyclePosture::Preserve,
    };
    if P::APPLICABILITY.is_empty()
        || P::APPLICABILITY
            .iter()
            .any(|case| case.lifecycle() != expected)
    {
        return Err(Denial::new(
            DenialKind::ProducerBindingMeaningMismatch,
            format!("{}/{}", kind.canonical_token(), P::IDENTITY),
        ));
    }
    Ok(())
}

pub(in crate::domain_computation::primary_graph::application_contribution) fn required_producers<
    S,
    P,
    R,
>() -> Vec<String>
where
    S: ApplicationSchema,
    P: Producer<S>,
    R: Conditional<S>,
{
    let mut required = vec![P::IDENTITY.to_owned()];
    for identity in R::REQUIRED_PRODUCERS {
        if !required.iter().any(|existing| existing == identity) {
            required.push((*identity).to_owned());
        }
    }
    required
}
