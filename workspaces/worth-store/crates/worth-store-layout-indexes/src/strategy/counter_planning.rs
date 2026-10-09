use super::{AdmittedLayoutStrategy, LayoutStrategyFamily, StrategyCounterProfile};
use crate::access::budget::PlannedCounterEnvelope;
use crate::access::execution::AccessPathCounterSnapshot;
use crate::access::shape::AccessShapeDetail;

impl AdmittedLayoutStrategy {
    pub const fn declared_counter_profile(&self) -> StrategyCounterProfile {
        match self.planned_counter_envelope() {
            Some(envelope) => envelope.aggregate_profile(),
            None => self
                .invariants
                .suite()
                .counter_evidence()
                .aggregate_profile(),
        }
    }

    pub const fn planned_counter_envelope(&self) -> Option<PlannedCounterEnvelope> {
        if family_requires_shape_specific_lookup_envelope(self.declaration.family()) {
            None
        } else {
            self.declaration.planned_counter_envelope()
        }
    }

    pub const fn planned_counter_envelope_for(
        &self,
        detail: AccessShapeDetail,
    ) -> Option<PlannedCounterEnvelope> {
        planned_counter_envelope_for(self.declaration.family(), detail)
    }
}

pub(crate) const fn planned_counter_envelope_for(
    family: LayoutStrategyFamily,
    detail: AccessShapeDetail,
) -> Option<PlannedCounterEnvelope> {
    match family {
        LayoutStrategyFamily::BTreeRange => None,
        LayoutStrategyFamily::BaselineLsmWriteOptimized => match detail {
            AccessShapeDetail::PointLookup
            | AccessShapeDetail::Append(_)
            | AccessShapeDetail::CompactionRead(_)
            | AccessShapeDetail::RebuildRead(_) => declared_strategy_counter_envelope(family),
            _ => None,
        },
        _ => None,
    }
}

pub(super) const fn declared_strategy_counter_envelope(
    family: LayoutStrategyFamily,
) -> Option<PlannedCounterEnvelope> {
    match family {
        LayoutStrategyFamily::BTreeRange => None,
        LayoutStrategyFamily::BaselineLsmWriteOptimized => Some(PlannedCounterEnvelope::new(
            AccessPathCounterSnapshot::exact(1, 0, 0, 0, 0, 0, 3, 3, 0, 0, 0, 0, 0, 0, 0, 3, 0)
                .with_selected_plan_authority_allocation(),
            AccessPathCounterSnapshot::exact(
                0, 0, 0, 2, 2, 4, 0, 0, 0, 0, 0, 4, 16_384, 8_192, 2, 4, 2,
            ),
            AccessPathCounterSnapshot::exact(0, 0, 1, 0, 1, 2, 0, 0, 0, 0, 0, 1, 8_192, 0, 0, 2, 0),
        )),
        _ => None,
    }
}

pub(super) const fn planned_publication_counter_snapshot_for(
    family: LayoutStrategyFamily,
) -> AccessPathCounterSnapshot {
    match family {
        LayoutStrategyFamily::BTreeRange => zero_counter_snapshot(),
        LayoutStrategyFamily::BaselineLsmWriteOptimized => {
            declared_strategy_counter_envelope(family)
                .expect("LSM strategy declares planned counter envelope")
                .publication()
        }
        _ => zero_counter_snapshot(),
    }
}

pub(super) const fn planned_recovery_counter_snapshot_for(
    family: LayoutStrategyFamily,
) -> AccessPathCounterSnapshot {
    match family {
        LayoutStrategyFamily::BTreeRange => zero_counter_snapshot(),
        LayoutStrategyFamily::BaselineLsmWriteOptimized => {
            declared_strategy_counter_envelope(family)
                .expect("LSM strategy declares planned counter envelope")
                .recovery()
        }
        _ => zero_counter_snapshot(),
    }
}

pub(super) const fn family_requires_shape_specific_lookup_envelope(
    family: LayoutStrategyFamily,
) -> bool {
    matches!(family, LayoutStrategyFamily::BTreeRange)
}

const fn zero_counter_snapshot() -> AccessPathCounterSnapshot {
    AccessPathCounterSnapshot::exact(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)
}
