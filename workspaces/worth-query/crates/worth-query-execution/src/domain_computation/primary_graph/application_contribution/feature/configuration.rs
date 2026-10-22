//! Construction values fill required slots; missing configuration never changes meaning.
use super::super::super::{
    application_entry::mutation::OperationHandler, WorthQueryApplicationProjection,
    WorthQueryPrimaryGraphInstallationDenial as Denial,
    WorthQueryPrimaryGraphInstallationDenialKind as Kind,
};
use super::super::producer::{ProducerSourceQuery, ProducerSourceValue};
use super::super::{
    WorthQueryApplicationConditionalBinding as Conditional, WorthQueryApplicationContributionSetup,
    WorthQueryApplicationProducerBinding as Producer,
};
use super::WorthQueryRequiredFeatureContract;
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema,
};

/// A configured feature always names both slots, even when a source disables one.
pub struct WorthQueryFeatureRuntime<Initial, Preserve> {
    pub initial: Option<Initial>,
    pub preserve: Option<Preserve>,
}
impl<Initial, Preserve> WorthQueryFeatureRuntime<Initial, Preserve> {
    pub fn new(initial: Initial, preserve: Preserve) -> Self {
        Self {
            initial: Some(initial),
            preserve: Some(preserve),
        }
    }
}

/// Optional configuration values are checked before any registration in this feature.
pub struct WorthQueryFeatureRoleRuntime<Handler, Provider, Readiness> {
    pub handler: Option<Handler>,
    pub provider: Option<Provider>,
    pub readiness: Option<Readiness>,
}
impl<Handler, Provider, Readiness> WorthQueryFeatureRoleRuntime<Handler, Provider, Readiness> {
    pub fn new(handler: Handler, provider: Provider, readiness: Readiness) -> Self {
        Self {
            handler: Some(handler),
            provider: Some(provider),
            readiness: Some(readiness),
        }
    }
}

/// Sealed to the typed construction values above; this is not an installer callback.
pub trait WorthQueryFeatureConfiguration<S, C>: sealed::Install<S, C>
where
    S: ApplicationSchema,
    C: WorthQueryRequiredFeatureContract<S>,
{
}

impl<S, C, I, P> WorthQueryFeatureConfiguration<S, C> for WorthQueryFeatureRuntime<I, P>
where
    S: ApplicationSchema,
    C: WorthQueryRequiredFeatureContract<S>,
    Self: sealed::Install<S, C>,
{
}

pub(in crate::domain_computation::primary_graph::application_contribution) fn install<S, C>(
    configuration: impl WorthQueryFeatureConfiguration<S, C>,
    setup: &mut WorthQueryApplicationContributionSetup<'_, S>,
) -> Result<(), Denial>
where
    S: ApplicationSchema,
    C: WorthQueryRequiredFeatureContract<S>,
{
    sealed::Install::install(configuration, setup)
}

mod sealed {
    use super::*;
    pub trait Install<S: ApplicationSchema, C: WorthQueryRequiredFeatureContract<S>>:
        Sized
    {
        fn install(
            self,
            setup: &mut WorthQueryApplicationContributionSetup<'_, S>,
        ) -> Result<(), Denial>;
    }

    impl<S, C, IH, PH> Install<S, C>
        for WorthQueryFeatureRuntime<
            WorthQueryFeatureRoleRuntime<
                IH,
                <C::InitialProducer as Producer<S>>::Provider,
                <C::InitialReadiness as Conditional<S>>::Configuration,
            >,
            WorthQueryFeatureRoleRuntime<
                PH,
                <C::PreserveProducer as Producer<S>>::Provider,
                <C::PreserveReadiness as Conditional<S>>::Configuration,
            >,
        >
    where
        S: ApplicationSchema,
        C: WorthQueryRequiredFeatureContract<S>,
        IH: OperationHandler<S, C::InitialMutation>,
        PH: OperationHandler<S, C::PreserveMutation>,
        ProducerSourceValue<S, C::InitialProducer>:
            WorthQueryApplicationProjection<S, ProducerSourceQuery<S, C::InitialProducer>>,
        ProducerSourceValue<S, C::PreserveProducer>:
            WorthQueryApplicationProjection<S, ProducerSourceQuery<S, C::PreserveProducer>>,
    {
        fn install(
            self,
            setup: &mut WorthQueryApplicationContributionSetup<'_, S>,
        ) -> Result<(), Denial> {
            super::super::contract::validate_pair::<S, C>()?;
            let initial = self.initial.ok_or_else(|| {
                missing(Kind::MissingProducerBinding, C::InitialProducer::IDENTITY)
            })?;
            let preserve = self.preserve.ok_or_else(|| {
                missing(Kind::MissingProducerBinding, C::PreserveProducer::IDENTITY)
            })?;
            let ih = initial.handler.ok_or_else(|| {
                missing(Kind::MissingMutationHandler, C::InitialMutation::IDENTITY)
            })?;
            let ip = initial.provider.ok_or_else(|| {
                missing(Kind::MissingProducerProvider, C::InitialProducer::IDENTITY)
            })?;
            let ir = initial.readiness.ok_or_else(|| {
                missing(
                    Kind::MissingConditionalBinding,
                    C::InitialReadiness::IDENTITY,
                )
            })?;
            let ph = preserve.handler.ok_or_else(|| {
                missing(Kind::MissingMutationHandler, C::PreserveMutation::IDENTITY)
            })?;
            let pp = preserve.provider.ok_or_else(|| {
                missing(Kind::MissingProducerProvider, C::PreserveProducer::IDENTITY)
            })?;
            let pr = preserve.readiness.ok_or_else(|| {
                missing(
                    Kind::MissingConditionalBinding,
                    C::PreserveReadiness::IDENTITY,
                )
            })?;
            setup.handler::<C::InitialMutation, _>(ih)?;
            setup.handler::<C::PreserveMutation, _>(ph)?;
            setup.producer::<C::InitialProducer>(ip)?;
            setup.producer::<C::PreserveProducer>(pp)?;
            setup.feature_conditional::<C::InitialProducer, C::InitialReadiness>(ir)?;
            setup.feature_conditional::<C::PreserveProducer, C::PreserveReadiness>(pr)
        }
    }
}

fn missing(kind: Kind, identity: impl Into<String>) -> Denial {
    Denial::new(kind, identity)
}
