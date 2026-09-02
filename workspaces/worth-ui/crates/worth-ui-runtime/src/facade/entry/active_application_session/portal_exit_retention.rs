use std::collections::BTreeMap;

pub(in crate::facade::entry) struct UiPortalExitRetentionCoordinator {
    retentions:
        BTreeMap<crate::runtime::motion::UiMotionTrackIdentity, UiPortalMotionExitRetention>,
    pending: Option<UiPortalExitTerminalPending>,
}

pub(in crate::facade::entry) enum UiPortalExitTerminalPending {
    Retry(crate::runtime::motion::UiMotionTrackIdentity),
    InFlight {
        track: crate::runtime::motion::UiMotionTrackIdentity,
        completion: super::super::portal_dismissal::DetachedUiPortalDismissalInFlight,
    },
    Indeterminate {
        track: crate::runtime::motion::UiMotionTrackIdentity,
        recovery: super::super::portal_dismissal::DetachedUiPortalDismissalIndeterminate,
    },
    Reconstruction {
        track: crate::runtime::motion::UiMotionTrackIdentity,
        proposal: crate::runtime::session::UiIndeterminatePortalProposalTransaction,
        in_flight: crate::mounting::UiMountedPresentationInFlight,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::facade::entry) enum UiPortalExitTerminalPendingKind {
    InFlight,
    Indeterminate,
    Reconstruction,
}

pub(super) struct UiPortalMotionExitRetention {
    portal: crate::runtime::portal::UiPortalExitRetentionReceipt,
    motion: crate::runtime::motion::UiMotionExitRetentionReceipt,
    terminal: Option<crate::runtime::motion::UiMotionTerminalReceipt>,
}

impl UiPortalExitRetentionCoordinator {
    pub(super) fn new() -> Self {
        Self {
            retentions: BTreeMap::new(),
            pending: None,
        }
    }

    pub(super) fn install(
        &mut self,
        portal: crate::runtime::portal::UiPortalExitRetentionReceipt,
        motion: crate::runtime::motion::UiMotionExitRetentionReceipt,
    ) -> Result<(), ()> {
        let target = motion.target();
        if portal.portal().diagnostic_value() != target.owner_key()
            || portal.portal().owner().mounted_instance_identity() != target.mounted_instance()
        {
            return Err(());
        }
        if self.retentions.contains_key(&motion.track()) {
            return Err(());
        }
        self.retentions.insert(
            motion.track(),
            UiPortalMotionExitRetention {
                portal,
                motion,
                terminal: None,
            },
        );
        Ok(())
    }

    pub(super) fn observe_terminal(
        &mut self,
        terminal: crate::runtime::motion::UiMotionTerminalReceipt,
    ) -> Result<bool, ()> {
        let Some(exit) = terminal.exit_retention() else {
            return Ok(false);
        };
        let retention = self.retentions.get_mut(&terminal.track()).ok_or(())?;
        if retention.motion != exit || retention.terminal.is_some() {
            return Err(());
        }
        retention.terminal = Some(terminal);
        Ok(true)
    }

    pub(super) fn next_terminal(&self) -> Option<&UiPortalMotionExitRetention> {
        self.retentions
            .values()
            .find(|retention| retention.terminal.is_some())
    }

    pub(super) fn pending(&self) -> Option<&UiPortalExitTerminalPending> {
        self.pending.as_ref()
    }

    pub(in crate::facade::entry) fn pending_replacement_kind(
        &self,
    ) -> Option<UiPortalExitTerminalPendingKind> {
        match self.pending.as_ref()? {
            UiPortalExitTerminalPending::Retry(_) => None,
            UiPortalExitTerminalPending::InFlight { .. } => {
                Some(UiPortalExitTerminalPendingKind::InFlight)
            }
            UiPortalExitTerminalPending::Indeterminate { .. } => {
                Some(UiPortalExitTerminalPendingKind::Indeterminate)
            }
            UiPortalExitTerminalPending::Reconstruction { .. } => {
                Some(UiPortalExitTerminalPendingKind::Reconstruction)
            }
        }
    }

    pub(in crate::facade::entry) fn pending_track_is_coordinated(&self) -> bool {
        self.pending.as_ref().map_or(true, |pending| {
            self.retentions.contains_key(&pending.track())
        })
    }

    pub(super) fn take_pending(&mut self) -> Option<UiPortalExitTerminalPending> {
        self.pending.take()
    }

    pub(super) fn retain_pending(&mut self, pending: UiPortalExitTerminalPending) {
        assert!(self.pending.replace(pending).is_none());
    }

    pub(super) fn remove(
        &mut self,
        track: crate::runtime::motion::UiMotionTrackIdentity,
    ) -> Option<UiPortalMotionExitRetention> {
        self.retentions.remove(&track)
    }

    pub(super) fn remove_displaced(
        &mut self,
        motion: crate::runtime::motion::UiMotionExitRetentionReceipt,
    ) -> Result<UiPortalMotionExitRetention, ()> {
        let retention = self.retentions.get(&motion.track()).ok_or(())?;
        if retention.motion != motion {
            return Err(());
        }
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.track() == motion.track())
        {
            // Same single predicate the displacement gate reads, so a retention
            // the gate admits is always removable here.
            match self.pending.take() {
                Some(pending)
                    if pending.physical_posture()
                        == UiPortalExitTerminalPhysicalPosture::RetryableBeforeEffect => {}
                Some(pending) => {
                    self.pending = Some(pending);
                    return Err(());
                }
                None => unreachable!("matched pending portal exit remains present"),
            }
        }
        Ok(self
            .retentions
            .remove(&motion.track())
            .expect("validated displaced portal exit remains retained"))
    }

    pub(super) fn remove_rebound_portal(
        &mut self,
        portal: crate::runtime::portal::UiPortalIdentity,
    ) -> Result<Option<crate::runtime::motion::UiMotionExitRetentionReceipt>, ()> {
        let motion = self.retentions.values().find_map(|retention| {
            (retention.portal.portal() == portal).then_some(retention.motion)
        });
        let Some(motion) = motion else {
            return Ok(None);
        };
        self.remove_displaced(motion)
            .map(|retention| Some(retention.motion()))
    }

    pub(in crate::facade::entry) fn has_portal(
        &self,
        portal: crate::runtime::portal::UiPortalIdentity,
    ) -> bool {
        self.retentions
            .values()
            .any(|retention| retention.portal.portal() == portal)
    }

    pub(in crate::facade::entry) fn retentions_for_portals(
        &self,
        portals: &[crate::runtime::portal::UiPortalIdentity],
    ) -> Box<
        [(
            crate::runtime::portal::UiPortalIdentity,
            crate::runtime::motion::UiMotionExitRetentionReceipt,
        )],
    > {
        self.retentions
            .values()
            .filter_map(|retention| {
                let portal = retention.portal.portal();
                portals
                    .contains(&portal)
                    .then_some((portal, retention.motion))
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    pub(super) fn clear_for_shutdown(&mut self) -> usize {
        assert!(self.pending.is_none());
        let retained = self.retentions.len();
        self.retentions.clear();
        retained
    }

    pub(super) fn get(
        &self,
        track: crate::runtime::motion::UiMotionTrackIdentity,
    ) -> Option<&UiPortalMotionExitRetention> {
        self.retentions.get(&track)
    }

    pub(super) fn len(&self) -> usize {
        self.retentions.len()
    }

    pub(super) fn resource_counts(&self) -> (usize, usize) {
        (self.retentions.len(), usize::from(self.pending.is_some()))
    }

    pub(super) fn has_terminal_work(&self) -> bool {
        self.pending.is_some() || self.next_terminal().is_some()
    }

    /// True when a retained exit for this Motion target has issued physical work
    /// that has not settled. Displacing that retention would strand the pending
    /// terminal, so a new transition for the same target must be denied before
    /// effect rather than committed and then discovered as unremovable.
    pub(super) fn physical_settlement_pending_for(
        &self,
        target: crate::runtime::motion::UiMotionTargetIdentity,
    ) -> bool {
        let Some(pending) = self.pending.as_ref().filter(|pending| {
            pending.physical_posture()
                == UiPortalExitTerminalPhysicalPosture::AwaitingPhysicalSettlement
        }) else {
            return false;
        };
        self.retentions
            .get(&pending.track())
            .is_some_and(|retention| retention.motion.target() == target)
    }

    pub(super) fn awaits_physical_progress(&self) -> bool {
        self.pending.as_ref().is_some_and(|pending| {
            pending.physical_posture()
                == UiPortalExitTerminalPhysicalPosture::AwaitingPhysicalSettlement
        })
    }
}

/// Whether a pending portal exit terminal still owns issued physical work. This
/// single classification governs displacement admission, displacement removal,
/// and host wake readiness so they cannot drift apart.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum UiPortalExitTerminalPhysicalPosture {
    RetryableBeforeEffect,
    AwaitingPhysicalSettlement,
}

impl UiPortalExitTerminalPending {
    pub(super) const fn physical_posture(&self) -> UiPortalExitTerminalPhysicalPosture {
        match self {
            Self::Retry(_) => UiPortalExitTerminalPhysicalPosture::RetryableBeforeEffect,
            Self::InFlight { .. } | Self::Indeterminate { .. } | Self::Reconstruction { .. } => {
                UiPortalExitTerminalPhysicalPosture::AwaitingPhysicalSettlement
            }
        }
    }

    pub(super) const fn track(&self) -> crate::runtime::motion::UiMotionTrackIdentity {
        match self {
            Self::Retry(track)
            | Self::InFlight { track, .. }
            | Self::Indeterminate { track, .. }
            | Self::Reconstruction { track, .. } => *track,
        }
    }

    pub(super) fn matches_native_physical(
        &self,
        class: worth_ui_host_native::UiNativePhysicalProgressClass,
        presentation: Option<worth_ui_host_native::UiNativePhysicalPresentationCorrelation>,
    ) -> bool {
        match self {
            Self::InFlight { completion, .. } => {
                completion.matches_native_physical(class, presentation)
            }
            Self::Indeterminate { .. } => matches!(
                class,
                worth_ui_host_native::UiNativePhysicalProgressClass::PresentationRecovery
                    | worth_ui_host_native::UiNativePhysicalProgressClass::Presentation
            ),
            Self::Reconstruction { in_flight, .. } => {
                let progress_class = match class {
                    worth_ui_host_native::UiNativePhysicalProgressClass::Presentation => {
                        worth_ui_host_contract::UiHostPresentationProgressClass::PhysicalSurface
                    }
                    worth_ui_host_native::UiNativePhysicalProgressClass::TextAtlas => {
                        worth_ui_host_contract::UiHostPresentationProgressClass::TextAtlas
                    }
                    worth_ui_host_native::UiNativePhysicalProgressClass::PresentationRecovery => {
                        return false;
                    }
                };
                in_flight.awaits_progress_class(progress_class)
                    && presentation.is_none_or(|presentation| {
                        presentation.attempt() == in_flight.attempt()
                            && in_flight
                                .pending_bindings()
                                .any(|binding| binding == presentation.binding())
                    })
            }
            Self::Retry(_) => false,
        }
    }
}

impl UiPortalMotionExitRetention {
    pub(super) const fn portal(&self) -> crate::runtime::portal::UiPortalExitRetentionReceipt {
        self.portal
    }

    pub(super) const fn motion(&self) -> crate::runtime::motion::UiMotionExitRetentionReceipt {
        self.motion
    }
}

#[cfg(test)]
mod tests {
    use super::{
        UiPortalExitRetentionCoordinator, UiPortalExitTerminalPending,
        UiPortalExitTerminalPhysicalPosture,
    };

    /// The classification is one exhaustive match, so a new pending variant
    /// cannot be added without deciding whether it owns physical work. That
    /// exhaustiveness is what keeps the displacement gate and the displacement
    /// removal from drifting apart.
    #[test]
    fn only_a_retryable_pending_admits_exit_displacement() {
        let track = crate::runtime::motion::UiMotionTrackIdentity::for_test(1);

        assert_eq!(
            UiPortalExitTerminalPending::Retry(track).physical_posture(),
            UiPortalExitTerminalPhysicalPosture::RetryableBeforeEffect
        );
    }

    #[test]
    fn an_idle_coordinator_blocks_no_target_and_awaits_no_physical_work() {
        let coordinator = UiPortalExitRetentionCoordinator::new();
        let target = crate::runtime::motion::UiMotionTargetIdentity::from_family_owner(
            worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound()
                .expect("fixture semantic surface"),
            worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound()
                .expect("fixture mounted instance"),
            7,
        );

        assert!(!coordinator.physical_settlement_pending_for(target));
        assert!(!coordinator.awaits_physical_progress());
        assert_eq!(coordinator.resource_counts(), (0, 0));
    }

    #[test]
    fn a_retryable_pending_awaits_no_physical_progress() {
        let mut coordinator = UiPortalExitRetentionCoordinator::new();
        coordinator.retain_pending(UiPortalExitTerminalPending::Retry(
            crate::runtime::motion::UiMotionTrackIdentity::for_test(2),
        ));

        assert!(!coordinator.awaits_physical_progress());
        assert_eq!(coordinator.resource_counts(), (0, 1));
    }
}
