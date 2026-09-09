use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

use super::{IntervalAnchor, IntervalCondition, MissedTickPolicy, TemporalCondition};

impl RetainedStorageMeasurement for TemporalCondition {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::After(_)
            | Self::AtOrAfter(_)
            | Self::Debounce(_)
            | Self::Throttle(_)
            | Self::StaleAfter(_) => {}
            Self::Interval(IntervalCondition {
                period: _,
                anchor,
                missed_tick_policy,
                clock_domain: _,
            }) => {
                match anchor {
                    IntervalAnchor::Registration
                    | IntervalAnchor::FirstEvaluation
                    | IntervalAnchor::ExplicitTick(_) => {}
                }
                match missed_tick_policy {
                    MissedTickPolicy::CollapseToOne
                    | MissedTickPolicy::CatchUpAll
                    | MissedTickPolicy::SkipToLatest => {}
                }
            }
        }
        Ok(Charge::ZERO)
    }
}
