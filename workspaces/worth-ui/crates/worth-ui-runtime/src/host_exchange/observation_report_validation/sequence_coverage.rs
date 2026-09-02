use worth_ui_host_contract::{
    UiHostObservationBatch, UiHostObservationFamily, UiHostObservationLoss,
    UiHostObservationReport, UiHostObservationSequence, UiHostObservationSequenceRange,
};

use super::structural_admission::UiStructurallyAdmittedObservationBatch;
use super::{UiHostObservationBatchDisposition, UiHostObservationReportDenial};

pub(super) struct UiSequenceCoveredObservationBatch {
    batch: UiHostObservationBatch,
    disposition: UiHostObservationBatchDisposition,
    host_survivor: Option<UiHostObservationSequence>,
}

impl UiSequenceCoveredObservationBatch {
    pub(super) fn prove(
        admitted: UiStructurallyAdmittedObservationBatch,
    ) -> Result<Self, UiHostObservationReportDenial> {
        let range = admitted.core().sequences();
        if range.first().value() == 0 {
            return Err(UiHostObservationReportDenial::SequenceMustBeginAtOne);
        }
        let (loss, disposition, host_survivor) =
            classify_loss(admitted.core().loss(), admitted.reports(), range)?;
        prove_exact_coverage(admitted.reports(), range, loss)?;
        Ok(Self {
            batch: admitted.into_batch(),
            disposition,
            host_survivor,
        })
    }

    pub(super) const fn core(&self) -> worth_ui_host_contract::UiHostObservationCanonicalCore {
        self.batch.canonical_core()
    }

    pub(super) fn reports(&self) -> &[UiHostObservationReport] {
        self.batch.reports()
    }

    pub(super) const fn integrity(&self) -> worth_ui_host_contract::UiHostObservationIntegrity {
        self.batch.integrity()
    }

    pub(super) const fn disposition(&self) -> UiHostObservationBatchDisposition {
        self.disposition
    }

    pub(super) const fn host_survivor(&self) -> Option<UiHostObservationSequence> {
        self.host_survivor
    }
}

fn classify_loss(
    loss: UiHostObservationLoss,
    reports: &[UiHostObservationReport],
    range: UiHostObservationSequenceRange,
) -> Result<
    (
        Option<UiHostObservationSequenceRange>,
        UiHostObservationBatchDisposition,
        Option<UiHostObservationSequence>,
    ),
    UiHostObservationReportDenial,
> {
    match loss {
        UiHostObservationLoss::Complete => {
            Ok((None, UiHostObservationBatchDisposition::Complete, None))
        }
        UiHostObservationLoss::Coalesced {
            family,
            replaced,
            survivor,
        } => {
            require_loss_range(family, replaced, range)?;
            let survivor_sequence = checked_successor(replaced.last())?;
            let report = reports
                .iter()
                .find(|report| report.sequence() == survivor_sequence)
                .ok_or(UiHostObservationReportDenial::SequenceGap)?;
            if report.family() != family {
                return Err(UiHostObservationReportDenial::UnsupportedCoalescing(family));
            }
            if report.coalescing_identity() != Some(survivor) {
                return Err(UiHostObservationReportDenial::CoalescingIdentityMismatch);
            }
            Ok((
                Some(replaced),
                UiHostObservationBatchDisposition::Coalesced {
                    family,
                    replaced,
                    survivor,
                },
                Some(survivor_sequence),
            ))
        }
        UiHostObservationLoss::Overflow { family, affected } => {
            require_loss_range(family, affected, range)?;
            Ok((
                Some(affected),
                UiHostObservationBatchDisposition::Overflow { family, affected },
                None,
            ))
        }
    }
}

fn require_loss_range(
    family: UiHostObservationFamily,
    loss: UiHostObservationSequenceRange,
    canonical: UiHostObservationSequenceRange,
) -> Result<(), UiHostObservationReportDenial> {
    if !family.permits_latest_value_coalescing() {
        return Err(UiHostObservationReportDenial::LosslessOverflow(family));
    }
    if !loss.is_ordered() || loss.first() < canonical.first() || loss.last() > canonical.last() {
        return Err(UiHostObservationReportDenial::MalformedBatch);
    }
    Ok(())
}

fn prove_exact_coverage(
    reports: &[UiHostObservationReport],
    range: UiHostObservationSequenceRange,
    loss: Option<UiHostObservationSequenceRange>,
) -> Result<(), UiHostObservationReportDenial> {
    let mut report_index = 0;
    let mut pending_loss = loss;
    let mut expected = range.first();
    loop {
        let report = reports.get(report_index);
        let report_start = report.map(UiHostObservationReport::sequence);
        let loss_start = pending_loss.map(UiHostObservationSequenceRange::first);
        let (start, end, consumed_loss) = match (report_start, loss_start) {
            (Some(report), Some(loss)) if report == loss => {
                return Err(UiHostObservationReportDenial::SequenceOverlap);
            }
            (Some(report), Some(loss)) if report < loss => (report, report, false),
            (_, Some(loss)) => {
                let interval = pending_loss.expect("loss start came from pending interval");
                (loss, interval.last(), true)
            }
            (Some(report), None) => (report, report, false),
            (None, None) => return Err(UiHostObservationReportDenial::SequenceGap),
        };
        if start < expected {
            return Err(UiHostObservationReportDenial::SequenceOverlap);
        }
        if start > expected {
            return Err(UiHostObservationReportDenial::SequenceGap);
        }
        if consumed_loss {
            pending_loss = None;
        } else {
            report_index += 1;
        }
        if end == range.last() {
            if report_index != reports.len() || pending_loss.is_some() {
                return Err(UiHostObservationReportDenial::SequenceOverlap);
            }
            return Ok(());
        }
        expected = checked_successor(end)?;
    }
}

fn checked_successor(
    sequence: UiHostObservationSequence,
) -> Result<UiHostObservationSequence, UiHostObservationReportDenial> {
    sequence
        .value()
        .checked_add(1)
        .map(UiHostObservationSequence::new)
        .ok_or(UiHostObservationReportDenial::SequenceExhausted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverage_rejects_a_hole_outside_the_declared_loss() {
        let range = sequence_range(1, 4);
        let reports = [report(1), report(4)];
        assert_eq!(
            prove_exact_coverage(&reports, range, Some(sequence_range(2, 2))),
            Err(UiHostObservationReportDenial::SequenceGap)
        );
    }

    #[test]
    fn coverage_rejects_a_report_overlapping_the_declared_loss() {
        let range = sequence_range(1, 3);
        let reports = [report(1), report(2), report(3)];
        assert_eq!(
            prove_exact_coverage(&reports, range, Some(sequence_range(2, 2))),
            Err(UiHostObservationReportDenial::SequenceOverlap)
        );
    }

    #[test]
    fn one_declared_interval_exactly_covers_prefix_middle_suffix_or_whole_range() {
        let range = sequence_range(1, 4);
        for (reports, loss) in [
            (vec![report(3), report(4)], sequence_range(1, 2)),
            (vec![report(1), report(4)], sequence_range(2, 3)),
            (vec![report(1), report(2)], sequence_range(3, 4)),
            (Vec::new(), sequence_range(1, 4)),
        ] {
            assert_eq!(prove_exact_coverage(&reports, range, Some(loss)), Ok(()));
        }
    }

    #[test]
    fn one_declared_interval_cannot_conceal_a_second_hole() {
        let range = sequence_range(1, 5);
        let reports = [report(1), report(3), report(5)];
        assert_eq!(
            prove_exact_coverage(&reports, range, Some(sequence_range(2, 2))),
            Err(UiHostObservationReportDenial::SequenceGap)
        );
    }

    #[test]
    fn range_ending_at_max_is_covered_without_wrapping() {
        let range = sequence_range(u64::MAX, u64::MAX);
        assert_eq!(
            prove_exact_coverage(&[report(u64::MAX)], range, None),
            Ok(())
        );
    }

    #[test]
    fn coalescing_requires_the_declared_adjacent_survivor_identity() {
        let range = sequence_range(1, 3);
        let reports = [pointer_report(3, 7, 2, 1)];
        let loss = UiHostObservationLoss::Coalesced {
            family: UiHostObservationFamily::PointerMotion,
            replaced: sequence_range(1, 2),
            survivor: worth_ui_host_contract::UiHostObservationCoalescingIdentity::PointerMotion {
                pointer: worth_ui_host_contract::UiHostPointerIdentity::new(8),
                capture_epoch: worth_ui_host_contract::UiHostPointerCaptureEpoch::new(2),
                pressed_buttons: worth_ui_host_contract::UiHostPressedPointerButtons::from_buttons(
                    [worth_ui_host_contract::UiHostPointerButton::Primary],
                ),
                device_kind: Some(worth_ui_host_contract::UiHostPointerDeviceKind::Mouse),
            },
        };
        assert_eq!(
            classify_loss(loss, &reports, range),
            Err(UiHostObservationReportDenial::CoalescingIdentityMismatch)
        );
    }

    #[test]
    fn a_lossless_family_cannot_claim_overflow() {
        assert!(UiHostObservationFamily::ScrollDelta.requires_lossless_delivery());
        assert_eq!(
            require_loss_range(
                UiHostObservationFamily::WindowFocus,
                sequence_range(1, 1),
                sequence_range(1, 2),
            ),
            Err(UiHostObservationReportDenial::LosslessOverflow(
                UiHostObservationFamily::WindowFocus
            ))
        );
        assert_eq!(
            require_loss_range(
                UiHostObservationFamily::ScrollDelta,
                sequence_range(1, 1),
                sequence_range(1, 2),
            ),
            Err(UiHostObservationReportDenial::LosslessOverflow(
                UiHostObservationFamily::ScrollDelta
            ))
        );
    }

    fn report(sequence: u64) -> UiHostObservationReport {
        UiHostObservationReport::new(
            UiHostObservationSequence::new(sequence),
            worth_ui_host_contract::UiHostObservationTimeBasis::HostMonotonicMillis(sequence),
            worth_ui_host_contract::UiHostObservationPayload::WindowFocus {
                surface: worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap(),
                focused: true,
            },
        )
    }

    fn pointer_report(
        sequence: u64,
        pointer: u64,
        capture_epoch: u64,
        pressed_buttons: u64,
    ) -> UiHostObservationReport {
        UiHostObservationReport::new(
            UiHostObservationSequence::new(sequence),
            worth_ui_host_contract::UiHostObservationTimeBasis::HostMonotonicMillis(sequence),
            worth_ui_host_contract::UiHostObservationPayload::PointerMotion {
                pointer: worth_ui_host_contract::UiHostPointerIdentity::new(pointer),
                capture_epoch: worth_ui_host_contract::UiHostPointerCaptureEpoch::new(
                    capture_epoch,
                ),
                pressed_buttons: if pressed_buttons == 0 {
                    worth_ui_host_contract::UiHostPressedPointerButtons::NONE
                } else {
                    worth_ui_host_contract::UiHostPressedPointerButtons::from_buttons([
                        worth_ui_host_contract::UiHostPointerButton::Primary,
                    ])
                },
                position: worth_ui_host_contract::UiHostSurfacePosition::viewport_logical(0, 0),
            },
        )
    }

    fn sequence_range(first: u64, last: u64) -> UiHostObservationSequenceRange {
        UiHostObservationSequenceRange::new(
            UiHostObservationSequence::new(first),
            UiHostObservationSequence::new(last),
        )
    }
}
