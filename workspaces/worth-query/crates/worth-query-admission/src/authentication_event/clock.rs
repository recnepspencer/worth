use std::marker::PhantomData;

use worth_query_installation::facade::{
    WorthQueryClockSourceIdentity, WorthQueryClockTimelineIdentity, WorthQueryNamedClock,
    WorthQueryNamedClockFailureKind, WorthQueryNamedClockSource,
};

/// An installed source is observed only by the authentication-event owner.
/// Caller-supplied timestamps and copied named-clock observations are not an
/// issuance path.
pub(crate) struct AuthenticationEventClock<Clock, Source> {
    source: Source,
    source_identity: WorthQueryClockSourceIdentity,
    timeline_identity: WorthQueryClockTimelineIdentity,
    last_reading: Option<(u64, u64)>,
    marker: PhantomData<fn() -> Clock>,
}

impl<Clock, Source> AuthenticationEventClock<Clock, Source>
where
    Clock: WorthQueryNamedClock,
    Source: WorthQueryNamedClockSource<Clock>,
{
    pub(crate) fn install(source: Source) -> Result<Self, AuthenticationEventClockDenial> {
        if Clock::PORTABLE_IDENTITY.is_empty()
            || Source::SEMANTIC_IDENTITY.is_empty()
            || Clock::PORTABLE_IDENTITY.chars().any(char::is_whitespace)
            || Source::SEMANTIC_IDENTITY.chars().any(char::is_whitespace)
        {
            return Err(AuthenticationEventClockDenial::InvalidInstallation);
        }
        Ok(Self {
            source_identity: source.source_identity(),
            timeline_identity: source.timeline_identity(),
            source,
            last_reading: None,
            marker: PhantomData,
        })
    }

    pub(crate) fn observe(&mut self) -> Result<u64, AuthenticationEventClockDenial> {
        if self.source.source_identity() != self.source_identity
            || self.source.timeline_identity() != self.timeline_identity
        {
            return Err(AuthenticationEventClockDenial::IdentityChanged);
        }
        let reading = self
            .source
            .observe()
            .map_err(|failure| AuthenticationEventClockDenial::Source(failure.kind()))?;
        if self.source.source_identity() != self.source_identity
            || self.source.timeline_identity() != self.timeline_identity
        {
            return Err(AuthenticationEventClockDenial::IdentityChanged);
        }
        let current = (reading.sequence(), reading.observed_time().nanoseconds());
        if let Some(previous) = self.last_reading {
            if current.0 < previous.0
                || current.1 < previous.1
                || (current.0 == previous.0 && current.1 != previous.1)
            {
                return Err(AuthenticationEventClockDenial::Regressed);
            }
        }
        self.last_reading = Some(current);
        Ok(current.1)
    }

    pub(crate) const fn source_identity(&self) -> &WorthQueryClockSourceIdentity {
        &self.source_identity
    }

    pub(crate) const fn timeline_identity(&self) -> &WorthQueryClockTimelineIdentity {
        &self.timeline_identity
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AuthenticationEventClockDenial {
    InvalidInstallation,
    Source(WorthQueryNamedClockFailureKind),
    IdentityChanged,
    Regressed,
}
