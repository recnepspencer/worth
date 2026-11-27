//! The declared timeline and inclusive bounds a grant is valid between.
use super::super::ApplicationCapabilityFieldBinding;
use crate::application_schema::{fragment::RecordSelection, ApplicationSchemaDeclarationDenial};
use worth_foundational::facade::{CanonicalDate, CanonicalTimestamp, ScalarAspectType};

/// The timeline a grant's validity bounds are written on. Every bound is
/// inclusive and judged against the operation's one trusted admission instant.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationCapabilityValidityTimeline {
    UnixEpochSeconds,
    UnixEpochMilliseconds,
    /// UTC instants (`CanonicalTimestamp`, microseconds, pre-1970 allowed).
    UtcTimestamp,
    /// Local civil dates (`CanonicalDate`) of the admission instant under the
    /// Home's capability calendar.
    LocalDate,
}

impl ApplicationCapabilityValidityTimeline {
    pub const fn scalar_family(self) -> ScalarAspectType {
        match self {
            Self::UnixEpochSeconds | Self::UnixEpochMilliseconds => ScalarAspectType::UInt64,
            Self::UtcTimestamp => ScalarAspectType::Timestamp,
            Self::LocalDate => ScalarAspectType::Date,
        }
    }

    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::UnixEpochSeconds => "unix-epoch-seconds",
            Self::UnixEpochMilliseconds => "unix-epoch-milliseconds",
            Self::UtcTimestamp => "utc-timestamp-micros",
            Self::LocalDate => "local-calendar-date",
        }
    }

    /// The one decoder for every durable record naming a timeline.
    pub fn from_canonical_name(name: &str) -> Option<Self> {
        [
            Self::UnixEpochSeconds,
            Self::UnixEpochMilliseconds,
            Self::UtcTimestamp,
            Self::LocalDate,
        ]
        .into_iter()
        .find(|timeline| timeline.canonical_name() == name)
    }

    /// Whether values count units since the Unix epoch, so a duration can be
    /// added to them. Governed elevation requires such a timeline.
    pub const fn counts_unix_epoch_units(self) -> bool {
        matches!(self, Self::UnixEpochSeconds | Self::UnixEpochMilliseconds)
    }

    /// Whether the timeline needs the Home's capability calendar.
    pub const fn requires_calendar(self) -> bool {
        matches!(self, Self::LocalDate)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityValidityDefinition {
    timeline: ApplicationCapabilityValidityTimeline,
    not_before: ApplicationCapabilityFieldBinding,
    not_after: ApplicationCapabilityFieldBinding,
}

impl ApplicationCapabilityValidityDefinition {
    pub fn new(
        timeline: ApplicationCapabilityValidityTimeline,
        not_before: ApplicationCapabilityFieldBinding,
        not_after: ApplicationCapabilityFieldBinding,
    ) -> Self {
        Self {
            timeline,
            not_before,
            not_after,
        }
    }

    /// Valid from `not_before` through `not_after` (both inclusive UTC instants).
    pub fn utc<NotBefore, NotAfter>(
        not_before: NotBefore,
        not_after: NotAfter,
    ) -> Result<Self, ApplicationSchemaDeclarationDenial>
    where
        NotBefore: RecordSelection<Value = CanonicalTimestamp>,
        NotAfter: RecordSelection<Value = CanonicalTimestamp>,
    {
        Ok(Self::new(
            ApplicationCapabilityValidityTimeline::UtcTimestamp,
            ApplicationCapabilityFieldBinding::from_reference(not_before)?,
            ApplicationCapabilityFieldBinding::from_reference(not_after)?,
        ))
    }

    /// Valid from local day `not_before` through local day `not_after`
    /// (inclusive), read on the Home's capability calendar.
    pub fn local_date<NotBefore, NotAfter>(
        not_before: NotBefore,
        not_after: NotAfter,
    ) -> Result<Self, ApplicationSchemaDeclarationDenial>
    where
        NotBefore: RecordSelection<Value = CanonicalDate>,
        NotAfter: RecordSelection<Value = CanonicalDate>,
    {
        Ok(Self::new(
            ApplicationCapabilityValidityTimeline::LocalDate,
            ApplicationCapabilityFieldBinding::from_reference(not_before)?,
            ApplicationCapabilityFieldBinding::from_reference(not_after)?,
        ))
    }

    pub const fn timeline(&self) -> ApplicationCapabilityValidityTimeline {
        self.timeline
    }

    pub fn not_before(&self) -> &ApplicationCapabilityFieldBinding {
        &self.not_before
    }

    pub fn not_after(&self) -> &ApplicationCapabilityFieldBinding {
        &self.not_after
    }
}

#[cfg(test)]
mod tests {
    use super::ApplicationCapabilityValidityTimeline as Timeline;

    #[test]
    fn every_timeline_name_decodes_to_itself_and_unknown_names_refuse() {
        for timeline in [
            Timeline::UnixEpochSeconds,
            Timeline::UnixEpochMilliseconds,
            Timeline::UtcTimestamp,
            Timeline::LocalDate,
        ] {
            assert_eq!(
                Timeline::from_canonical_name(timeline.canonical_name()),
                Some(timeline)
            );
        }
        assert_eq!(Timeline::from_canonical_name("local-date"), None);
    }
}
