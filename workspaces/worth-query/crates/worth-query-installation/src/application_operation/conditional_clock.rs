use crate::domain_operation::{
    WorthQueryConditionalTrigger, WorthQueryNamedClock, WorthQueryNamedClockFailure,
    WorthQueryNamedClockObservation, WorthQueryNamedClockSource, WorthQueryTemporalWake,
};

use super::{
    WorthQueryConditionalApplicationOperationDenial,
    WorthQueryConditionalApplicationOperationDenialKind,
    WorthQueryInstalledHostConditionalProvider,
};

/// Move-only temporal-node installation after its exact provider and named
/// clock source have both been admitted.
pub struct WorthQueryInstalledNamedClockConditionalNode<
    Schema,
    ApplicationOperation,
    Input,
    D,
    O,
    F,
    N,
    Provider,
    Clock,
    Source,
> {
    provider: WorthQueryInstalledHostConditionalProvider<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        N,
        Provider,
    >,
    source_identity: crate::domain_operation::WorthQueryClockSourceIdentity,
    timeline_identity: crate::domain_operation::WorthQueryClockTimelineIdentity,
    source: Source,
    marker: std::marker::PhantomData<fn() -> Clock>,
}

impl<Schema, ApplicationOperation, Input, D, O, F, N, Provider>
    WorthQueryInstalledHostConditionalProvider<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        N,
        Provider,
    >
where
    Provider: crate::domain_operation::WorthQueryHostConditionalPredicateProvider<N>,
{
    pub fn bind_named_clock<Clock, Source>(
        self,
        source: Source,
    ) -> Result<
        WorthQueryInstalledNamedClockConditionalNode<
            Schema,
            ApplicationOperation,
            Input,
            D,
            O,
            F,
            N,
            Provider,
            Clock,
            Source,
        >,
        WorthQueryConditionalApplicationOperationDenial,
    >
    where
        Clock: WorthQueryNamedClock,
        Source: WorthQueryNamedClockSource<Clock>,
    {
        validate_temporal_node(self.node().declaration())?;
        validate_clock_identity::<Clock>()?;
        let source_identity = source.source_identity();
        let timeline_identity = source.timeline_identity();
        Ok(WorthQueryInstalledNamedClockConditionalNode {
            provider: self,
            source_identity,
            timeline_identity,
            source,
            marker: std::marker::PhantomData,
        })
    }
}

impl<Schema, ApplicationOperation, Input, D, O, F, N, Provider, Clock, Source>
    WorthQueryInstalledNamedClockConditionalNode<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        N,
        Provider,
        Clock,
        Source,
    >
where
    Provider: crate::domain_operation::WorthQueryHostConditionalPredicateProvider<N>,
    Clock: WorthQueryNamedClock,
    Source: WorthQueryNamedClockSource<Clock>,
{
    pub fn provider(
        &self,
    ) -> &WorthQueryInstalledHostConditionalProvider<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        N,
        Provider,
    > {
        &self.provider
    }

    pub fn clock_identity(&self) -> &'static str {
        Clock::PORTABLE_IDENTITY
    }

    pub fn source_identity(&self) -> &crate::domain_operation::WorthQueryClockSourceIdentity {
        &self.source_identity
    }

    pub fn timeline_identity(&self) -> &crate::domain_operation::WorthQueryClockTimelineIdentity {
        &self.timeline_identity
    }

    #[doc(hidden)]
    pub fn observe_for_runtime(
        &self,
    ) -> Result<WorthQueryNamedClockObservation<Clock>, WorthQueryNamedClockFailure> {
        let reading = self.source.observe()?;
        Ok(WorthQueryNamedClockObservation::from_admitted_source(
            self.source_identity.clone(),
            self.timeline_identity.clone(),
            reading,
        ))
    }
}

fn validate_temporal_node(
    declaration: &crate::domain_operation::WorthQueryPortableConditionalNodeDeclaration,
) -> Result<(), WorthQueryConditionalApplicationOperationDenial> {
    match declaration.trigger() {
        WorthQueryConditionalTrigger::Temporal(
            WorthQueryTemporalWake::MonotonicClock | WorthQueryTemporalWake::WallClock,
        ) => Ok(()),
        WorthQueryConditionalTrigger::Temporal(WorthQueryTemporalWake::OnSnapshotAdvance) => {
            Err(conditional_denial(
                WorthQueryConditionalApplicationOperationDenialKind::HostClockNotRequired,
                declaration.identity(),
            ))
        }
        _ => Err(conditional_denial(
            WorthQueryConditionalApplicationOperationDenialKind::NodeNotTemporal,
            declaration.identity(),
        )),
    }
}

fn validate_clock_identity<Clock: WorthQueryNamedClock>(
) -> Result<(), WorthQueryConditionalApplicationOperationDenial> {
    let identity = Clock::PORTABLE_IDENTITY;
    if identity.is_empty()
        || identity.trim() != identity
        || identity.chars().any(char::is_whitespace)
    {
        Err(conditional_denial(
            WorthQueryConditionalApplicationOperationDenialKind::ClockIdentityInvalid,
            identity,
        ))
    } else {
        Ok(())
    }
}

fn conditional_denial(
    kind: WorthQueryConditionalApplicationOperationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryConditionalApplicationOperationDenial {
    WorthQueryConditionalApplicationOperationDenial::new(kind, subject)
}
