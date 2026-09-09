#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiPortalDismissalTrigger {
    Escape {
        semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    },
    OutsidePress {
        semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        viewport_point_bits: [u32; 2],
    },
    AcceptedSelection {
        semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    },
    AnchorLoss(super::UiPortalIdentity),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiPortalDismissalIgnoreReason {
    NoMatchingPortal,
    InsideTopmostPortal,
}

pub(crate) enum UiPortalDismissalPreparation {
    Ignored(UiPortalDismissalIgnoreReason),
    Prepared(UiPreparedPortalDismissal),
}

#[must_use = "dismissal changes no portal truth until its transition is published"]
pub(crate) struct UiPreparedPortalDismissal {
    transition: super::UiPreparedPortalServiceTransition,
    presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    #[cfg(test)]
    input_shielded: bool,
}

impl super::UiPortalRuntimeState {
    pub(crate) fn prepare_dismissal(
        &self,
        trigger: UiPortalDismissalTrigger,
        sampled_bounds: Option<worth_ui_host_contract::UiMountedCanonicalBox>,
        idempotency: crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity,
    ) -> Result<UiPortalDismissalPreparation, super::UiPortalServiceTransitionDenial> {
        let Some((portal, record)) = self.dismissal_target(trigger) else {
            return Ok(UiPortalDismissalPreparation::Ignored(
                UiPortalDismissalIgnoreReason::NoMatchingPortal,
            ));
        };
        let admitted_by_policy = match trigger {
            UiPortalDismissalTrigger::Escape { .. } => record.policy.dismisses_on_escape(),
            UiPortalDismissalTrigger::OutsidePress { .. } => {
                record.policy.dismisses_on_outside_press()
            }
            UiPortalDismissalTrigger::AcceptedSelection { .. } => {
                record.policy.dismisses_on_accepted_selection()
            }
            UiPortalDismissalTrigger::AnchorLoss(_) => record.policy.dismisses_on_anchor_loss(),
        };
        if !admitted_by_policy {
            return Ok(UiPortalDismissalPreparation::Ignored(
                UiPortalDismissalIgnoreReason::NoMatchingPortal,
            ));
        }
        if let UiPortalDismissalTrigger::OutsidePress {
            viewport_point_bits,
            ..
        } = trigger
        {
            let point = viewport_point_bits.map(f32::from_bits);
            let committed_bounds = record
                .placement
                .map(|placement| placement.prepared().bounds().mounted_box());
            if sampled_bounds
                .or(committed_bounds)
                .is_some_and(|bounds| contains(bounds, point))
            {
                return Ok(UiPortalDismissalPreparation::Ignored(
                    UiPortalDismissalIgnoreReason::InsideTopmostPortal,
                ));
            }
        }
        let cause = match trigger {
            UiPortalDismissalTrigger::Escape { .. } => super::UiPortalDismissalCause::Escape,
            UiPortalDismissalTrigger::OutsidePress { .. } => {
                super::UiPortalDismissalCause::OutsidePress
            }
            UiPortalDismissalTrigger::AcceptedSelection { .. } => {
                super::UiPortalDismissalCause::AcceptedSelection
            }
            UiPortalDismissalTrigger::AnchorLoss(_) => super::UiPortalDismissalCause::AnchorLoss,
        };
        #[cfg(test)]
        let input_shielded = record.placement.is_some_and(|placement| {
            placement.prepared().shielding() == super::UiPortalInputShielding::ModalSurface
        });
        let presentation = record
            .placement
            .expect("every active portal has committed placement")
            .prepared()
            .presentation();
        let transition = self.prepare(super::UiPortalServiceRequest::close(
            portal,
            idempotency,
            cause,
            record.semantic_surface,
        ))?;
        Ok(UiPortalDismissalPreparation::Prepared(
            UiPreparedPortalDismissal {
                transition,
                presentation,
                #[cfg(test)]
                input_shielded,
            },
        ))
    }

    pub(crate) fn dismissal_target_identity(
        &self,
        trigger: UiPortalDismissalTrigger,
    ) -> Option<super::UiPortalIdentity> {
        self.dismissal_target(trigger).map(|(portal, _)| portal)
    }

    fn dismissal_target(
        &self,
        trigger: UiPortalDismissalTrigger,
    ) -> Option<(super::UiPortalIdentity, &super::state::UiPortalRecord)> {
        if let UiPortalDismissalTrigger::AnchorLoss(anchor) = trigger {
            return self
                .records
                .get_key_value(&anchor)
                .filter(|(_, record)| {
                    matches!(
                        record.posture,
                        super::UiPortalLifecyclePosture::Open
                            | super::UiPortalLifecyclePosture::Visible
                    )
                })
                .map(|(portal, record)| (*portal, record));
        }
        let semantic_surface = match trigger {
            UiPortalDismissalTrigger::Escape { semantic_surface }
            | UiPortalDismissalTrigger::OutsidePress {
                semantic_surface, ..
            }
            | UiPortalDismissalTrigger::AcceptedSelection { semantic_surface } => semantic_surface,
            UiPortalDismissalTrigger::AnchorLoss(_) => unreachable!(),
        };
        let portal = self
            .surface_stacks
            .get(&semantic_surface)?
            .topmost_portal()?;
        let record = self.records.get(&portal)?;
        matches!(
            record.posture,
            super::UiPortalLifecyclePosture::Open
                | super::UiPortalLifecyclePosture::Visible
                | super::UiPortalLifecyclePosture::Closing
        )
        .then_some((portal, record))
    }

    pub(super) fn portal_descends_from(
        &self,
        mut portal: super::UiPortalIdentity,
        ancestor: super::UiPortalIdentity,
    ) -> bool {
        while let Some(parent) = self
            .records
            .get(&portal)
            .and_then(|record| record.placement)
            .and_then(|placement| placement.prepared().layer().parent())
        {
            if parent == ancestor {
                return true;
            }
            portal = parent;
        }
        false
    }
}

impl UiPreparedPortalDismissal {
    #[cfg(test)]
    pub(crate) const fn portal(&self) -> super::UiPortalIdentity {
        self.transition.portal()
    }

    #[cfg(test)]
    pub(crate) const fn input_shielded(&self) -> bool {
        self.input_shielded
    }

    pub(crate) const fn presentation(
        &self,
    ) -> worth_ui_host_contract::UiHostObservationPresentationBasis {
        self.presentation
    }

    pub(crate) fn into_transition(self) -> super::UiPreparedPortalServiceTransition {
        self.transition
    }
}

fn contains(bounds: worth_ui_host_contract::UiMountedCanonicalBox, point: [f32; 2]) -> bool {
    point[0] >= bounds.x()
        && point[1] >= bounds.y()
        && point[0] < bounds.x() + bounds.width()
        && point[1] < bounds.y() + bounds.height()
}
