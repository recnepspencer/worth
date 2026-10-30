use std::collections::VecDeque;

/// Bounded number of terminally closed portals whose exact closing request stays
/// recognizable. A duplicate close carrying the same idempotency identity must
/// still settle as `Idempotent`, but terminal portals may not accumulate in the
/// live table that placement, dismissal, descendant, and command-routing work
/// scans.
pub(super) const UI_PORTAL_CLOSED_REQUEST_CAPACITY: usize = 8;

/// The prior request a portal identity already settled, whether that portal is
/// still live or was terminally closed inside the retention window.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct UiPortalPriorRequest {
    pub(super) posture: super::super::UiPortalLifecyclePosture,
    pub(super) semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    pub(super) last_request: crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity,
    pub(super) dismissal: Option<super::super::UiPortalDismissalCause>,
    pub(super) placement: Option<super::super::UiCommittedPortalPlacement>,
}

/// One terminally closed portal retained only for duplicate-request recognition.
/// It carries no placement, no exit retention, and no layer participation, so it
/// can never re-enter dismissal targeting or descendant resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct UiPortalClosedRequestRecord {
    portal: super::super::UiPortalIdentity,
    semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    last_request: crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity,
    dismissal: super::super::UiPortalDismissalCause,
}

/// Bounded, insertion-ordered retention window over terminally closed portals.
#[derive(Debug)]
pub(super) struct UiPortalClosedRequestWindow {
    records: VecDeque<UiPortalClosedRequestRecord>,
}

impl UiPortalClosedRequestWindow {
    pub(super) const fn new() -> Self {
        Self {
            records: VecDeque::new(),
        }
    }

    pub(super) fn retain(
        &mut self,
        portal: super::super::UiPortalIdentity,
        semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        last_request: crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity,
        dismissal: super::super::UiPortalDismissalCause,
    ) {
        self.forget(portal);
        if self.records.len() == UI_PORTAL_CLOSED_REQUEST_CAPACITY {
            self.records.pop_front();
        }
        self.records.push_back(UiPortalClosedRequestRecord {
            portal,
            semantic_surface,
            last_request,
            dismissal,
        });
    }

    pub(super) fn prior_request(
        &self,
        portal: super::super::UiPortalIdentity,
    ) -> Option<UiPortalPriorRequest> {
        self.records
            .iter()
            .find(|record| record.portal == portal)
            .map(|record| UiPortalPriorRequest {
                posture: super::super::UiPortalLifecyclePosture::Closed,
                semantic_surface: record.semantic_surface,
                last_request: record.last_request,
                dismissal: Some(record.dismissal),
                placement: None,
            })
    }

    pub(super) fn forget(&mut self, portal: super::super::UiPortalIdentity) {
        self.records.retain(|record| record.portal != portal);
    }

    pub(super) fn len(&self) -> usize {
        self.records.len()
    }

    pub(super) fn clear(&mut self) {
        self.records.clear();
    }
}

/// Decides whether a prepared request repeats the exact prior request for the
/// same portal identity, and therefore which posture the transition stages.
pub(super) fn classify(
    prior: Option<UiPortalPriorRequest>,
    request: &super::super::UiPortalServiceRequest,
    placement: Option<super::super::UiPreparedPortalPlacement>,
) -> (
    super::super::UiPortalLifecyclePosture,
    super::super::UiPortalServiceDisposition,
) {
    if let Some(prior) = prior.filter(|prior| repeats_exact_request(*prior, request, placement))
        && settles_idempotently(prior.posture, request)
    {
        return (
            prior.posture,
            super::super::UiPortalServiceDisposition::Idempotent,
        );
    }
    match request.operation() {
        super::super::request::UiPortalServiceOperation::Open => (
            super::super::UiPortalLifecyclePosture::Open,
            super::super::UiPortalServiceDisposition::Opened,
        ),
        super::super::request::UiPortalServiceOperation::Close(_) => (
            super::super::UiPortalLifecyclePosture::Closing,
            super::super::UiPortalServiceDisposition::Closing,
        ),
    }
}

fn repeats_exact_request(
    prior: UiPortalPriorRequest,
    request: &super::super::UiPortalServiceRequest,
    placement: Option<super::super::UiPreparedPortalPlacement>,
) -> bool {
    if prior.last_request != request.idempotency()
        || prior.semantic_surface != request.semantic_surface()
    {
        return false;
    }
    match request.operation() {
        super::super::request::UiPortalServiceOperation::Open => {
            prior.dismissal.is_none()
                && prior
                    .placement
                    .map(super::super::UiCommittedPortalPlacement::prepared)
                    == placement
        }
        super::super::request::UiPortalServiceOperation::Close(cause) => {
            prior.dismissal == Some(cause)
        }
    }
}

const fn settles_idempotently(
    posture: super::super::UiPortalLifecyclePosture,
    request: &super::super::UiPortalServiceRequest,
) -> bool {
    matches!(
        (posture, request.operation()),
        (
            super::super::UiPortalLifecyclePosture::Open
                | super::super::UiPortalLifecyclePosture::Visible,
            super::super::request::UiPortalServiceOperation::Open
        ) | (
            super::super::UiPortalLifecyclePosture::Closing
                | super::super::UiPortalLifecyclePosture::Closed,
            super::super::request::UiPortalServiceOperation::Close(_)
        )
    )
}

#[cfg(test)]
mod tests {
    use super::{UiPortalClosedRequestWindow, UI_PORTAL_CLOSED_REQUEST_CAPACITY};

    fn portal(value: u64) -> super::super::super::UiPortalIdentity {
        super::super::super::UiPortalIdentity::for_owner(
            super::super::super::UiPortalOwnerIdentity::for_test(value, value),
        )
    }

    fn surface() -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound()
            .expect("closed-window fixture surface")
    }

    fn idempotency(
        value: u64,
    ) -> crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity {
        crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity::issued(1, value)
    }

    #[test]
    fn window_evicts_the_oldest_beyond_its_declared_capacity() {
        let mut window = UiPortalClosedRequestWindow::new();
        let portals = (1..=u64::try_from(UI_PORTAL_CLOSED_REQUEST_CAPACITY).unwrap() + 4)
            .map(|value| {
                let portal = portal(value);
                window.retain(
                    portal,
                    surface(),
                    idempotency(value),
                    super::super::super::UiPortalDismissalCause::Escape,
                );
                portal
            })
            .collect::<Vec<_>>();

        assert_eq!(window.len(), UI_PORTAL_CLOSED_REQUEST_CAPACITY);
        assert_eq!(window.prior_request(portals[0]), None);
        assert!(window.prior_request(portals[portals.len() - 1]).is_some());
    }

    #[test]
    fn retaining_one_portal_twice_replaces_rather_than_accumulates() {
        let mut window = UiPortalClosedRequestWindow::new();
        let portal = portal(7);
        window.retain(
            portal,
            surface(),
            idempotency(1),
            super::super::super::UiPortalDismissalCause::Escape,
        );
        window.retain(
            portal,
            surface(),
            idempotency(2),
            super::super::super::UiPortalDismissalCause::OutsidePress,
        );

        assert_eq!(window.len(), 1);
        let retained = window.prior_request(portal).expect("one retained request");
        assert_eq!(retained.last_request, idempotency(2));
        assert_eq!(
            retained.dismissal,
            Some(super::super::super::UiPortalDismissalCause::OutsidePress)
        );
        assert_eq!(retained.placement, None);
    }

    #[test]
    fn forget_and_clear_release_the_window() {
        let mut window = UiPortalClosedRequestWindow::new();
        let portal = portal(11);
        window.retain(
            portal,
            surface(),
            idempotency(1),
            super::super::super::UiPortalDismissalCause::Escape,
        );
        window.forget(portal);
        assert_eq!(window.len(), 0);

        window.retain(
            portal,
            surface(),
            idempotency(2),
            super::super::super::UiPortalDismissalCause::Escape,
        );
        window.clear();
        assert_eq!(window.len(), 0);
    }
}
