use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Default)]
pub(crate) struct UiMountedPresentationLeaseGate {
    active: Rc<RefCell<Option<Weak<()>>>>,
}

pub(crate) struct UiMountedPresentationLease {
    pub(super) seal: Rc<()>,
    active: Weak<RefCell<Option<Weak<()>>>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiMountedPresentationLeaseDenial {
    AlreadyBound,
}

impl UiMountedPresentationLeaseGate {
    pub(crate) fn claim(
        &self,
    ) -> Result<UiMountedPresentationLease, UiMountedPresentationLeaseDenial> {
        let mut active = self.active.borrow_mut();
        if active.as_ref().is_some_and(|seal| seal.upgrade().is_some()) {
            return Err(UiMountedPresentationLeaseDenial::AlreadyBound);
        }
        let seal = Rc::new(());
        *active = Some(Rc::downgrade(&seal));
        Ok(UiMountedPresentationLease {
            seal,
            active: Rc::downgrade(&self.active),
        })
    }

    pub(crate) fn admits(
        &self,
        view: &worth_ui_host_contract::UiMountedFrameConsumptionView<'_>,
    ) -> bool {
        self.active_seal()
            .is_some_and(|active| view.issued_by_runtime(&active))
    }

    pub(crate) fn admits_token(
        &self,
        token: &worth_ui_host_contract::UiHostPresentationCompletionToken,
    ) -> bool {
        self.active_seal()
            .is_some_and(|active| token.issued_by_runtime(&active))
    }

    fn active_seal(&self) -> Option<Rc<()>> {
        self.active.borrow().as_ref().and_then(Weak::upgrade)
    }
}

impl UiMountedPresentationLease {
    pub(crate) fn admits_work(&self, work: &super::work::UiMountedPresentationWork) -> bool {
        work.issued_by(&self.seal)
    }

    pub(crate) fn mechanics_authority(&self) -> Rc<()> {
        Rc::clone(&self.seal)
    }
}

impl Drop for UiMountedPresentationLease {
    fn drop(&mut self) {
        let Some(active) = self.active.upgrade() else {
            return;
        };
        let mut active = active.borrow_mut();
        let matches = active
            .as_ref()
            .and_then(Weak::upgrade)
            .is_some_and(|current| Rc::ptr_eq(&current, &self.seal));
        if matches {
            *active = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inert_mechanics_view_and_completion_token_cannot_enter_runtime_authority() {
        let gate = UiMountedPresentationLeaseGate::default();
        let _runtime_lease = gate.claim().expect("runtime claims presentation authority");
        let projection = crate::certification_support::semantic_text_projection_for_certification(
            crate::certification_support::UiSemanticTextProjectionCertificationMutation::Exact,
        );
        let requirement = worth_ui_host_contract::UiMountedSurfaceBindingRequirement::new(
            projection.surface(),
            worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap(),
            projection.binding(),
            worth_ui_host_contract::WorthUiHostCapabilityObservationGeneration::new(7),
            11,
            worth_ui_host_contract::UiHostSurfacePresentationMode::RecordOnly,
        );
        let initial =
            crate::certification_support::initial_presentation_mechanics_for_certification(
                &projection,
                requirement,
            );
        let forged_authority = Rc::new(());
        let worth_ui_host_contract::UiHostProtocolNegotiation::Compatible(protocol) =
            worth_ui_host_contract::UiHostProtocolContract::current().negotiate()
        else {
            unreachable!("current protocol negotiates with itself")
        };
        let view = worth_ui_host_contract::UiMountedFrameConsumptionView::from_inert_mechanics(
            worth_ui_host_contract::UiMountedFrameConsumptionInput {
                authority: forged_authority,
                host_session_identity: 1,
                protocol,
                capability_generation: requirement.capability_generation(),
                capability_profile_digest: requirement.capability_profile_digest(),
                attempt:
                    worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound()
                        .unwrap(),
                deadline: worth_ui_host_contract::UiPresentationDeadline::at_tick(1),
                requirement,
                presentation_work: worth_ui_host_contract::UiMountedPresentationWorkView::Initial(
                    &initial,
                ),
                appearance_work: None,
                qualified_text: &NoQualifiedText,
                text_raster_work: None,
            },
        );
        assert!(!gate.admits(&view));
        assert!(!gate.admits_token(&view.issue_completion_token()));
    }

    struct NoQualifiedText;

    impl worth_ui_host_contract::UiMountedQualifiedTextResolver for NoQualifiedText {
        fn resolve(
            &self,
            _identity: worth_ui_host_contract::UiQualifiedTextLayoutIdentity,
        ) -> Option<worth_ui_host_contract::UiQualifiedTextLayoutView<'_>> {
            None
        }
    }
}
