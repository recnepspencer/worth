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
    /// Preflight the existing pair before its containing feature registers members.
    pub fn validate_for<S, C>(&self) -> Result<(), Denial>
    where
        S: ApplicationSchema,
        C: WorthQueryRequiredFeatureContract<S>,
        Self: WorthQueryFeatureConfiguration<S, C>,
    {
        sealed::Install::<S, C>::validate(self)
    }
}

pub type WorthQueryFeaturePairConfiguration<S, C, IH, PH> = WorthQueryFeatureRuntime<
    WorthQueryFeatureRoleRuntime<IH,
        <<C as WorthQueryRequiredFeatureContract<S>>::InitialProducer as Producer<S>>::Provider,
        <<C as WorthQueryRequiredFeatureContract<S>>::InitialReadiness as Conditional<S>>::Configuration>,
    WorthQueryFeatureRoleRuntime<PH,
        <<C as WorthQueryRequiredFeatureContract<S>>::PreserveProducer as Producer<S>>::Provider,
        <<C as WorthQueryRequiredFeatureContract<S>>::PreserveReadiness as Conditional<S>>::Configuration>,
>;

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
        fn validate(&self) -> Result<(), Denial>;
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
        fn validate(&self) -> Result<(), Denial> {
            super::super::contract::validate_pair::<S, C>()?;
            let initial = self.initial.as_ref().ok_or_else(|| {
                missing(Kind::MissingProducerBinding, C::InitialProducer::IDENTITY)
            })?;
            let preserve = self.preserve.as_ref().ok_or_else(|| {
                missing(Kind::MissingProducerBinding, C::PreserveProducer::IDENTITY)
            })?;
            for (present, kind, identity) in [
                (
                    initial.handler.is_some(),
                    Kind::MissingMutationHandler,
                    C::InitialMutation::IDENTITY,
                ),
                (
                    initial.provider.is_some(),
                    Kind::MissingProducerProvider,
                    C::InitialProducer::IDENTITY,
                ),
                (
                    initial.readiness.is_some(),
                    Kind::MissingConditionalBinding,
                    C::InitialReadiness::IDENTITY,
                ),
                (
                    preserve.handler.is_some(),
                    Kind::MissingMutationHandler,
                    C::PreserveMutation::IDENTITY,
                ),
                (
                    preserve.provider.is_some(),
                    Kind::MissingProducerProvider,
                    C::PreserveProducer::IDENTITY,
                ),
                (
                    preserve.readiness.is_some(),
                    Kind::MissingConditionalBinding,
                    C::PreserveReadiness::IDENTITY,
                ),
            ] {
                if !present {
                    return Err(missing(kind, identity));
                }
            }
            Ok(())
        }
        fn install(
            self,
            setup: &mut WorthQueryApplicationContributionSetup<'_, S>,
        ) -> Result<(), Denial> {
            <Self as Install<S, C>>::validate(&self)?;
            let initial = self.initial.expect("pair preflight retained Initial");
            let preserve = self.preserve.expect("pair preflight retained Preserve");
            let ih = initial
                .handler
                .expect("pair preflight retained Initial handler");
            let ip = initial
                .provider
                .expect("pair preflight retained Initial provider");
            let ir = initial
                .readiness
                .expect("pair preflight retained Initial readiness");
            let ph = preserve
                .handler
                .expect("pair preflight retained Preserve handler");
            let pp = preserve
                .provider
                .expect("pair preflight retained Preserve provider");
            let pr = preserve
                .readiness
                .expect("pair preflight retained Preserve readiness");
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
