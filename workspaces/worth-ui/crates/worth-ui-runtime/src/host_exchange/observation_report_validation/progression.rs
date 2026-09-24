use worth_ui_host_contract::{UiHostObservationSequence, UiHostObservationSequenceRange};

use super::UiHostObservationReportDenial;

impl super::UiHostObservationReportValidation {
    /// Whether `incoming` starts where the host stream continues. After a
    /// denied batch the host moves on without resending it, while a caller
    /// holding that batch may retry its position with a corrected basis, so
    /// either continuation is next. The cursor itself moves only on admission.
    pub(super) fn validate_sequence_progression(
        &self,
        incoming: UiHostObservationSequenceRange,
    ) -> Result<(), UiHostObservationReportDenial> {
        let retried = next_after(self.last_sequence, incoming);
        match self.denied_sequence {
            Some(denied) if retried.is_err() => next_after(Some(denied), incoming).or(retried),
            _ => retried,
        }
    }

    pub(super) fn advance_sequence(&mut self, last: UiHostObservationSequence) {
        self.last_sequence = Some(last);
        self.denied_sequence = None;
    }

    /// Records that this session's batch ending at `last` was denied at the
    /// next position, so the host may continue after it.
    pub(super) fn pass_denied_sequence(&mut self, last: UiHostObservationSequence) {
        self.denied_sequence = Some(last);
    }
}

fn next_after(
    previous: Option<UiHostObservationSequence>,
    incoming: UiHostObservationSequenceRange,
) -> Result<(), UiHostObservationReportDenial> {
    let expected = match previous {
        Some(sequence) => sequence
            .value()
            .checked_add(1)
            .ok_or(UiHostObservationReportDenial::SequenceExhausted)?,
        None => 1,
    };
    if incoming.first().value() < expected {
        return Err(UiHostObservationReportDenial::SequenceReordered);
    }
    if incoming.first().value() > expected {
        return Err(UiHostObservationReportDenial::SequenceGap);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_partition_cannot_progress_beyond_the_sequence_domain() {
        assert_eq!(
            next_after(
                Some(UiHostObservationSequence::new(u64::MAX)),
                UiHostObservationSequenceRange::new(
                    UiHostObservationSequence::new(u64::MAX),
                    UiHostObservationSequence::new(u64::MAX),
                ),
            ),
            Err(UiHostObservationReportDenial::SequenceExhausted)
        );
    }
}
