use super::{
    BridgeManagedClockAcceptedObservation, BridgeManagedClockBinding,
    BridgeManagedClockObservationOutcome, BridgeManagedClockObservationParts,
    BridgeManagedTemporalDenial, BridgeManagedTemporalDenialKind,
};
use crate::conditional_execution::execution_view::BridgeConditionalExecutionView;

enum ObservationAdmission {
    Advance,
    Duplicate,
    Stale,
    Reordered,
}

impl BridgeConditionalExecutionView<'_> {
    pub fn observe_managed_clock(
        &self,
        parts: BridgeManagedClockObservationParts<'_>,
    ) -> Result<BridgeManagedClockObservationOutcome, BridgeManagedTemporalDenial> {
        self.observe_clock(parts.binding, parts.sequence, || {
            if parts.binding.lease.sampler.is_some() {
                return Err(sampling_mode_mismatch());
            }
            validate_observation_source(&parts)?;
            Ok(parts.observed_coordinate)
        })
    }

    /// Observes the installed sampler without accepting caller-supplied time
    /// or source labels. A reboot requires closing and installing a new binding.
    pub fn observe_sampled_managed_clock(
        &self,
        binding: &BridgeManagedClockBinding,
        sequence: u64,
    ) -> Result<BridgeManagedClockObservationOutcome, BridgeManagedTemporalDenial> {
        self.observe_clock(binding, sequence, || {
            binding
                .lease
                .sampler
                .as_ref()
                .ok_or_else(sampling_mode_mismatch)?
                .coordinate()
        })
    }

    fn observe_clock(
        &self,
        binding: &BridgeManagedClockBinding,
        sequence: u64,
        observe_coordinate: impl FnOnce() -> Result<u64, BridgeManagedTemporalDenial>,
    ) -> Result<BridgeManagedClockObservationOutcome, BridgeManagedTemporalDenial> {
        let binding_identity = &binding.binding_identity;
        let cell = self.admit_managed_clock_lane(binding)?;
        // A host adapter must be called without holding a managed lane lock.
        let coordinate = observe_coordinate();
        let mut lane = super::lock_lane(&cell)?;
        super::lifecycle::require_clock_lane(&lane, binding)?;
        let observed_coordinate = coordinate?;
        let admission =
            classify_observation(lane.last_observation(), sequence, observed_coordinate);
        match admission {
            ObservationAdmission::Stale => Ok(BridgeManagedClockObservationOutcome::Stale),
            ObservationAdmission::Reordered => Ok(BridgeManagedClockObservationOutcome::Reordered),
            ObservationAdmission::Duplicate => {
                let due = lane.promote_due(binding_identity)?;
                Ok(BridgeManagedClockObservationOutcome::Duplicate(
                    BridgeManagedClockAcceptedObservation {
                        sequence,
                        observed_coordinate,
                        signal_advance_ordinal: None,
                        due,
                    },
                ))
            }
            ObservationAdmission::Advance => {
                let (signal_advance_ordinal, due) =
                    lane.advance_and_promote_due(binding_identity, observed_coordinate)?;
                lane.record_observation(sequence, observed_coordinate);
                Ok(BridgeManagedClockObservationOutcome::Accepted(
                    BridgeManagedClockAcceptedObservation {
                        sequence,
                        observed_coordinate,
                        signal_advance_ordinal: Some(signal_advance_ordinal),
                        due,
                    },
                ))
            }
        }
    }
}

fn sampling_mode_mismatch() -> BridgeManagedTemporalDenial {
    BridgeManagedTemporalDenial::new(
        BridgeManagedTemporalDenialKind::ForeignClockSource,
        "observation must use the installed clock source",
    )
}

fn validate_observation_source(
    parts: &BridgeManagedClockObservationParts<'_>,
) -> Result<(), BridgeManagedTemporalDenial> {
    if parts.binding.source_identity() != parts.source_identity {
        return Err(BridgeManagedTemporalDenial::new(
            BridgeManagedTemporalDenialKind::ForeignClockSource,
            "clock observation came from another installed source",
        ));
    }
    if parts.binding.timeline_identity() != parts.timeline_identity {
        return Err(BridgeManagedTemporalDenial::new(
            BridgeManagedTemporalDenialKind::ForeignClockTimeline,
            "clock observation came from another installed timeline",
        ));
    }
    Ok(())
}

fn classify_observation(
    last: Option<(u64, u64)>,
    sequence: u64,
    coordinate: u64,
) -> ObservationAdmission {
    let Some((last_sequence, last_coordinate)) = last else {
        return ObservationAdmission::Advance;
    };
    if sequence == last_sequence && coordinate == last_coordinate {
        ObservationAdmission::Duplicate
    } else if sequence <= last_sequence {
        ObservationAdmission::Stale
    } else if coordinate < last_coordinate {
        ObservationAdmission::Reordered
    } else {
        ObservationAdmission::Advance
    }
}
