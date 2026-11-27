//! The declared timeline and inclusive bounds a grant is valid between.
use super::super::ApplicationCapabilityFieldBinding;
use crate::application_schema::{fragment::RecordSelection, ApplicationSchemaDeclarationDenial};
use worth_foundational::facade::{
    AspectValue, CanonicalDate, CanonicalTimestamp, ScalarAspectType,
};

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

    /// The greatest value of the timeline: the expiry of a grant whose window
    /// is open above, after every instant a trusted clock can sample.
    pub const fn end_of_time(self) -> AspectValue {
        match self {
            Self::UnixEpochSeconds | Self::UnixEpochMilliseconds => AspectValue::UInt64(u64::MAX),
            Self::UtcTimestamp => AspectValue::Timestamp(CanonicalTimestamp {
                micros_since_unix_epoch: i64::MAX,
            }),
            Self::LocalDate => AspectValue::Date(CanonicalDate {
                days_from_unix_epoch: i32::MAX,
            }),
        }
    }

    /// Whether the timeline needs the Home's capability calendar.
    pub const fn requires_calendar(self) -> bool {
        matches!(self, Self::LocalDate)
    }
}

/// A value type a grant validity bound may be written in, naming the
/// timeline its bounds are judged on.
pub trait ApplicationCapabilityValidityValue {
    const TIMELINE: ApplicationCapabilityValidityTimeline;
}

impl ApplicationCapabilityValidityValue for CanonicalTimestamp {
    const TIMELINE: ApplicationCapabilityValidityTimeline =
        ApplicationCapabilityValidityTimeline::UtcTimestamp;
}

impl ApplicationCapabilityValidityValue for CanonicalDate {
    const TIMELINE: ApplicationCapabilityValidityTimeline =
        ApplicationCapabilityValidityTimeline::LocalDate;
}

/// A validity a declaration accepts as written: a window, or the result of a
/// window constructor that checks its fields.
pub trait ApplicationCapabilityValidityDeclaration {
    fn into_validity(
        self,
    ) -> Result<ApplicationCapabilityValidityDefinition, ApplicationSchemaDeclarationDenial>;
}

impl ApplicationCapabilityValidityDeclaration for ApplicationCapabilityValidityDefinition {
    fn into_validity(
        self,
    ) -> Result<ApplicationCapabilityValidityDefinition, ApplicationSchemaDeclarationDenial> {
        Ok(self)
    }
}

impl ApplicationCapabilityValidityDeclaration
    for Result<ApplicationCapabilityValidityDefinition, ApplicationSchemaDeclarationDenial>
{
    fn into_validity(
        self,
    ) -> Result<ApplicationCapabilityValidityDefinition, ApplicationSchemaDeclarationDenial> {
        self
    }
}

/// The inclusive window a grant is valid in. Either bound may be open: an
/// open lower bound admits every instant before the upper one, an open upper
/// bound never expires, and [`Self::unbounded`] is valid at every instant.
/// A delegable capability keeps both bounds, so a delegated child's window
/// is always proven inside its parent's.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityValidityDefinition {
    timeline: ApplicationCapabilityValidityTimeline,
    not_before: Option<ApplicationCapabilityFieldBinding>,
    not_after: Option<ApplicationCapabilityFieldBinding>,
}

impl ApplicationCapabilityValidityDefinition {
    /// Valid from `not_before` through `not_after`, both inclusive.
    pub fn new(
        timeline: ApplicationCapabilityValidityTimeline,
        not_before: ApplicationCapabilityFieldBinding,
        not_after: ApplicationCapabilityFieldBinding,
    ) -> Self {
        Self {
            timeline,
            not_before: Some(not_before),
            not_after: Some(not_after),
        }
    }

    /// Valid at every instant: the grant has no validity fields at all and
    /// stays current while it is active and its extent is live.
    pub fn unbounded() -> Self {
        Self {
            timeline: ApplicationCapabilityValidityTimeline::UtcTimestamp,
            not_before: None,
            not_after: None,
        }
    }

    /// Valid from `not_before` (inclusive) and never expiring. The field's
    /// type names the timeline: a UTC instant or a local calendar day.
    pub fn from<NotBefore>(
        not_before: NotBefore,
    ) -> Result<Self, ApplicationSchemaDeclarationDenial>
    where
        NotBefore: RecordSelection,
        NotBefore::Value: ApplicationCapabilityValidityValue,
    {
        Ok(Self {
            timeline: <NotBefore::Value as ApplicationCapabilityValidityValue>::TIMELINE,
            not_before: Some(ApplicationCapabilityFieldBinding::from_reference(
                not_before,
            )?),
            not_after: None,
        })
    }

    /// Valid at every instant through `not_after` (inclusive).
    pub fn until<NotAfter>(not_after: NotAfter) -> Result<Self, ApplicationSchemaDeclarationDenial>
    where
        NotAfter: RecordSelection,
        NotAfter::Value: ApplicationCapabilityValidityValue,
    {
        Ok(Self {
            timeline: <NotAfter::Value as ApplicationCapabilityValidityValue>::TIMELINE,
            not_before: None,
            not_after: Some(ApplicationCapabilityFieldBinding::from_reference(
                not_after,
            )?),
        })
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

    /// Rebuilds a window decoded from a portable record, accepting exactly
    /// the shapes the constructors produce: a closed window on any timeline,
    /// a one-sided window on a UTC or local-date timeline, or the unbounded
    /// window on the UTC timeline.
    pub fn from_portable_parts(
        timeline: ApplicationCapabilityValidityTimeline,
        not_before: Option<ApplicationCapabilityFieldBinding>,
        not_after: Option<ApplicationCapabilityFieldBinding>,
    ) -> Option<Self> {
        let canonical = match (&not_before, &not_after) {
            (Some(_), Some(_)) => true,
            (None, None) => timeline == ApplicationCapabilityValidityTimeline::UtcTimestamp,
            _ => !timeline.counts_unix_epoch_units(),
        };
        canonical.then_some(Self {
            timeline,
            not_before,
            not_after,
        })
    }

    pub const fn timeline(&self) -> ApplicationCapabilityValidityTimeline {
        self.timeline
    }

    /// The inclusive lower bound field, or `None` when the window is open
    /// below.
    pub fn not_before(&self) -> Option<&ApplicationCapabilityFieldBinding> {
        self.not_before.as_ref()
    }

    /// The inclusive upper bound field, or `None` when the grant never
    /// expires.
    pub fn not_after(&self) -> Option<&ApplicationCapabilityFieldBinding> {
        self.not_after.as_ref()
    }

    /// Both bounds, when the window is closed on both sides.
    pub fn closed_bounds(
        &self,
    ) -> Option<(
        &ApplicationCapabilityFieldBinding,
        &ApplicationCapabilityFieldBinding,
    )> {
        self.not_before.as_ref().zip(self.not_after.as_ref())
    }

    /// Whether both bounds are present.
    pub fn is_closed(&self) -> bool {
        self.closed_bounds().is_some()
    }

    /// The present bounds, lower first.
    pub fn bounds(&self) -> impl Iterator<Item = &ApplicationCapabilityFieldBinding> {
        self.not_before.iter().chain(self.not_after.iter())
    }

    /// Whether every present bound is a field of `entity` written on this
    /// timeline, and a closed window's two bounds share one value type.
    pub fn is_written_on(&self, entity: &str) -> bool {
        let family = self.timeline.scalar_family();
        self.bounds()
            .all(|bound| bound.entity() == entity && bound.scalar_family() == family)
            && self.closed_bounds().is_none_or(|(not_before, not_after)| {
                not_before.value_type() == not_after.value_type()
            })
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
