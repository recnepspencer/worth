//! Configured correspondence with the one real owned-async cell timeline.
use super::*;
use worth_query_installation::facade::{
    WorthQueryClockSourceIdentity, WorthQueryClockTimelineIdentity, WorthQueryNamedClock,
    WorthQueryNamedClockFailure, WorthQueryNamedClockFailureKind, WorthQueryNamedClockSource,
};

pub(in crate::domain_computation::primary_graph::application_resource) struct ResourceClockReading {
    pub(in crate::domain_computation::primary_graph::application_resource) nanoseconds: u64,
}

pub(in crate::domain_computation::primary_graph::application_resource) trait ResourceClock:
    Send + Sync
{
    fn observe(&self) -> Result<ResourceClockReading, WorthQueryNamedClockFailure>;
}

pub(super) struct ClockCorrespondence {
    clock_type: TypeId,
    source_type: TypeId,
    instance: Arc<dyn Any + Send + Sync>,
    source: WorthQueryClockSourceIdentity,
    timeline: WorthQueryClockTimelineIdentity,
}

impl ClockCorrespondence {
    pub(super) fn of<Clock, Source>(source: &Arc<Source>) -> Self
    where
        Clock: WorthQueryNamedClock,
        Source: WorthQueryNamedClockSource<Clock>,
    {
        Self {
            clock_type: TypeId::of::<Clock>(),
            source_type: TypeId::of::<Source>(),
            instance: source.clone(),
            source: source.source_identity(),
            timeline: source.timeline_identity(),
        }
    }

    pub(super) fn matches(&self, other: &Self) -> bool {
        self.clock_type == other.clock_type
            && self.source_type == other.source_type
            && Arc::ptr_eq(&self.instance, &other.instance)
            && self.source == other.source
            && self.timeline == other.timeline
    }
}

pub(super) struct TypedResourceClock<Clock, Source> {
    source: Arc<Source>,
    admitted_source: WorthQueryClockSourceIdentity,
    admitted_timeline: WorthQueryClockTimelineIdentity,
    marker: PhantomData<fn() -> Clock>,
}

impl<Clock: WorthQueryNamedClock, Source: WorthQueryNamedClockSource<Clock>>
    TypedResourceClock<Clock, Source>
{
    pub(super) fn new(source: Arc<Source>) -> Self {
        Self {
            admitted_source: source.source_identity(),
            admitted_timeline: source.timeline_identity(),
            source,
            marker: PhantomData,
        }
    }
}

impl<Clock: WorthQueryNamedClock, Source: WorthQueryNamedClockSource<Clock>> ResourceClock
    for TypedResourceClock<Clock, Source>
{
    fn observe(&self) -> Result<ResourceClockReading, WorthQueryNamedClockFailure> {
        if self.source.source_identity() != self.admitted_source
            || self.source.timeline_identity() != self.admitted_timeline
        {
            return Err(WorthQueryNamedClockFailure::new(
                WorthQueryNamedClockFailureKind::ObservationFailed,
                "resource clock source/timeline changed after installation",
            ));
        }
        let reading = self.source.observe()?;
        Ok(ResourceClockReading {
            nanoseconds: reading.observed_time().nanoseconds(),
        })
    }
}
