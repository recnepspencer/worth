use crate::facade::interaction::UiHostInteractionIngressOutcome;
use worth_ui_host_contract::UiHostObservationDrainDenial;

pub(crate) enum UiNativeObservationIngressSettlement {
    Drained(UiNativeObservationDrainReport),
    DrainDenied(UiHostObservationDrainDenial),
}

pub(crate) struct UiNativeObservationDrainReport {
    outcomes: Box<[UiHostInteractionIngressOutcome]>,
    dismissals: Box<[super::WorthUiAdmittedPortalDismissal]>,
    reachability: worth_ui_host_native::UiNativeInputReachability,
    applied_batches: usize,
    duplicate_batches: usize,
    quarantined_batches: usize,
    denied_batches: usize,
}

impl UiNativeObservationIngressSettlement {
    pub(crate) fn focus_publications(
        &self,
    ) -> impl Iterator<
        Item = &Result<
            (
                super::UiSemanticFocusPublicationReceipt,
                crate::mounting::UiMountedFramePublicationReceipt,
            ),
            super::UiFocusPlacementExecutionDenial,
        >,
    > {
        let outcomes = match self {
            Self::Drained(report) => report.outcomes.as_ref(),
            Self::DrainDenied(_) => &[],
        };
        outcomes
            .iter()
            .filter_map(|outcome| match outcome {
                UiHostInteractionIngressOutcome::Applied(receipt) => {
                    Some(receipt.focus_publications())
                }
                _ => None,
            })
            .flatten()
    }

    pub(crate) fn from_outcomes(
        outcomes: Box<[UiHostInteractionIngressOutcome]>,
        reachability: worth_ui_host_native::UiNativeInputReachability,
    ) -> Self {
        Self::with_dismissals(outcomes, Box::new([]), reachability)
    }

    pub(crate) fn with_dismissals(
        outcomes: Box<[UiHostInteractionIngressOutcome]>,
        dismissals: Box<[super::WorthUiAdmittedPortalDismissal]>,
        reachability: worth_ui_host_native::UiNativeInputReachability,
    ) -> Self {
        let mut report = UiNativeObservationDrainReport {
            outcomes,
            dismissals,
            reachability,
            applied_batches: 0,
            duplicate_batches: 0,
            quarantined_batches: 0,
            denied_batches: 0,
        };
        for outcome in &report.outcomes {
            match outcome {
                UiHostInteractionIngressOutcome::Applied(_) => report.applied_batches += 1,
                UiHostInteractionIngressOutcome::Duplicate(_) => report.duplicate_batches += 1,
                UiHostInteractionIngressOutcome::Quarantined(_) => report.quarantined_batches += 1,
                UiHostInteractionIngressOutcome::Denied(_) => report.denied_batches += 1,
            }
        }
        Self::Drained(report)
    }

    pub(crate) fn reachability(&self) -> worth_ui_host_native::UiNativeInputReachability {
        match self {
            Self::Drained(report) => report.reachability,
            Self::DrainDenied(_) => worth_ui_host_native::UiNativeInputReachability::default(),
        }
    }

    #[cfg(test)]
    pub(crate) fn into_outcomes(self) -> Box<[UiHostInteractionIngressOutcome]> {
        match self {
            Self::Drained(report) => report.outcomes,
            Self::DrainDenied(_) => Box::new([]),
        }
    }

    pub(crate) fn into_routing_parts(
        self,
    ) -> (
        Box<[UiHostInteractionIngressOutcome]>,
        Box<[super::WorthUiAdmittedPortalDismissal]>,
    ) {
        match self {
            Self::Drained(report) => (report.outcomes, report.dismissals),
            Self::DrainDenied(_) => (Box::new([]), Box::new([])),
        }
    }

    pub(crate) const fn drain_denial(&self) -> Option<UiHostObservationDrainDenial> {
        match self {
            Self::Drained(_) => None,
            Self::DrainDenied(denial) => Some(*denial),
        }
    }

    pub(crate) const fn counts(&self) -> (usize, usize, usize, usize) {
        match self {
            Self::Drained(report) => (
                report.applied_batches,
                report.duplicate_batches,
                report.quarantined_batches,
                report.denied_batches,
            ),
            Self::DrainDenied(_) => (0, 0, 0, 0),
        }
    }

    pub(crate) fn retained_batch_count(&self) -> usize {
        match self {
            Self::Drained(report) => report.outcomes.len(),
            Self::DrainDenied(_) => 0,
        }
    }
}
