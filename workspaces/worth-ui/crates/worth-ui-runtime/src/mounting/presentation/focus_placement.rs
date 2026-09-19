#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiFocusHostPlacementSettlementDenial {
    NoCurrentSemanticFocus,
    TargetMismatch,
    StaleRequest,
    IndeterminateRequiresReconciliation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiMountedFocusPlacementDenial {
    IdentityExhausted,
    Request(worth_ui_host_contract::UiHostFocusPlacementRequestDenial),
    Settlement(UiFocusHostPlacementSettlementDenial),
}

#[derive(Clone, Copy)]
pub(crate) struct UiMountedFocusPlacementRequestBasis {
    pub(crate) protocol: worth_ui_host_contract::UiHostProtocolAgreement,
    pub(crate) host_session: u64,
    pub(crate) host_surface: worth_ui_host_contract::UiHostSurfaceIdentity,
    pub(crate) binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    pub(crate) presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    pub(crate) target: worth_ui_host_contract::UiHostFocusPlacementTarget,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiFocusHostPlacementReconciliationDenial {
    NoIndeterminatePlacement,
    RequestMismatch,
    ProtocolMismatch,
    HostSessionMismatch,
    SurfaceMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiFocusHostPlacementReconciliationOutcome {
    RequestedTargetObserved,
    RequestedTargetAbsent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiFocusHostPlacementReconciliationReceipt {
    observation: worth_ui_host_contract::UiHostFocusPlacementObservation,
    outcome: UiFocusHostPlacementReconciliationOutcome,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiFocusHostPlacementShutdownReport {
    abandoned_indeterminate_request:
        Option<worth_ui_host_contract::UiHostFocusPlacementRequestIdentity>,
}

pub(super) struct UiMountedFocusPlacementState {
    persistence: crate::runtime::UiServiceStatePersistencePosture,
    next_identity: u64,
    last: Option<worth_ui_host_contract::UiHostFocusPlacementAcknowledgement>,
    reconciliation: Option<UiFocusHostPlacementReconciliationReceipt>,
}

impl Default for UiMountedFocusPlacementState {
    fn default() -> Self {
        Self {
            persistence: crate::runtime::UiServiceStatePersistencePosture::Effecting,
            next_identity: 1,
            last: None,
            reconciliation: None,
        }
    }
}

impl super::UiMountedPresentationCoordinator {
    pub(crate) fn place_semantic_focus(
        &mut self,
        basis: UiMountedFocusPlacementRequestBasis,
        supported: bool,
        host: crate::facade::UiHostEffectPort<'_>,
    ) -> Result<
        worth_ui_host_contract::UiHostFocusPlacementAcknowledgement,
        UiMountedFocusPlacementDenial,
    > {
        self.focus_placement.place(basis, supported, host)
    }

    pub(crate) fn reconcile_focus_placement(
        &mut self,
        observation: worth_ui_host_contract::UiHostFocusPlacementObservation,
    ) -> Result<UiFocusHostPlacementReconciliationReceipt, UiFocusHostPlacementReconciliationDenial>
    {
        self.focus_placement.reconcile(observation)
    }

    pub(crate) fn shutdown_focus_placement(&mut self) -> UiFocusHostPlacementShutdownReport {
        self.focus_placement.shutdown()
    }
}

impl UiMountedFocusPlacementState {
    fn place(
        &mut self,
        basis: UiMountedFocusPlacementRequestBasis,
        supported: bool,
        host: crate::facade::UiHostEffectPort<'_>,
    ) -> Result<
        worth_ui_host_contract::UiHostFocusPlacementAcknowledgement,
        UiMountedFocusPlacementDenial,
    > {
        self.require_available()
            .map_err(UiMountedFocusPlacementDenial::Settlement)?;
        let identity =
            worth_ui_host_contract::UiHostFocusPlacementRequestIdentity::new(self.next_identity)
                .ok_or(UiMountedFocusPlacementDenial::IdentityExhausted)?;
        let successor_identity = self
            .next_identity
            .checked_add(1)
            .ok_or(UiMountedFocusPlacementDenial::IdentityExhausted)?;
        let request = worth_ui_host_contract::UiHostFocusPlacementRequest::new(
            worth_ui_host_contract::UiHostFocusPlacementRequestInput {
                identity,
                protocol: basis.protocol,
                host_session: basis.host_session,
                host_surface: basis.host_surface,
                binding: basis.binding,
                presentation: basis.presentation,
                target: basis.target,
            },
        )
        .map_err(UiMountedFocusPlacementDenial::Request)?;
        self.next_identity = successor_identity;
        let acknowledgement = if supported {
            host.adapter()
                .place_semantic_focus(host.authority(), request)
        } else {
            worth_ui_host_contract::UiHostFocusPlacementAcknowledgement::settled(
                request,
                worth_ui_host_contract::UiHostFocusPlacementDisposition::RejectedBeforeEffect(
                    worth_ui_host_contract::UiHostFocusPlacementRejection::Unsupported,
                ),
            )
        };
        self.settle(acknowledgement, Some(basis.target))
            .map_err(UiMountedFocusPlacementDenial::Settlement)?;
        Ok(acknowledgement)
    }

    fn settle(
        &mut self,
        acknowledgement: worth_ui_host_contract::UiHostFocusPlacementAcknowledgement,
        current: Option<worth_ui_host_contract::UiHostFocusPlacementTarget>,
    ) -> Result<(), UiFocusHostPlacementSettlementDenial> {
        self.require_available()?;
        let current =
            current.ok_or(UiFocusHostPlacementSettlementDenial::NoCurrentSemanticFocus)?;
        let request = acknowledgement.request();
        if request.target() != current {
            return Err(UiFocusHostPlacementSettlementDenial::TargetMismatch);
        }
        if self
            .last
            .is_some_and(|last| last.request().identity() >= request.identity())
        {
            return Err(UiFocusHostPlacementSettlementDenial::StaleRequest);
        }
        self.last = Some(acknowledgement);
        self.reconciliation = None;
        Ok(())
    }

    fn require_available(&self) -> Result<(), UiFocusHostPlacementSettlementDenial> {
        let unresolved = self.last.is_some_and(|last| {
            last.disposition()
                == worth_ui_host_contract::UiHostFocusPlacementDisposition::Indeterminate
                && self
                    .reconciliation
                    .map(|receipt| receipt.observation.request())
                    != Some(last.request().identity())
        });
        if unresolved {
            Err(UiFocusHostPlacementSettlementDenial::IndeterminateRequiresReconciliation)
        } else {
            Ok(())
        }
    }

    fn reconcile(
        &mut self,
        observation: worth_ui_host_contract::UiHostFocusPlacementObservation,
    ) -> Result<UiFocusHostPlacementReconciliationReceipt, UiFocusHostPlacementReconciliationDenial>
    {
        let acknowledgement = self
            .last
            .filter(|last| {
                last.disposition()
                    == worth_ui_host_contract::UiHostFocusPlacementDisposition::Indeterminate
            })
            .ok_or(UiFocusHostPlacementReconciliationDenial::NoIndeterminatePlacement)?;
        let request = acknowledgement.request();
        if observation.request() != request.identity() {
            return Err(UiFocusHostPlacementReconciliationDenial::RequestMismatch);
        }
        if observation.protocol() != request.protocol() {
            return Err(UiFocusHostPlacementReconciliationDenial::ProtocolMismatch);
        }
        if observation.host_session() != request.host_session() {
            return Err(UiFocusHostPlacementReconciliationDenial::HostSessionMismatch);
        }
        if observation.host_surface() != request.host_surface() {
            return Err(UiFocusHostPlacementReconciliationDenial::SurfaceMismatch);
        }
        let outcome = if observation.observed_target() == Some(request.target()) {
            UiFocusHostPlacementReconciliationOutcome::RequestedTargetObserved
        } else {
            UiFocusHostPlacementReconciliationOutcome::RequestedTargetAbsent
        };
        let receipt = UiFocusHostPlacementReconciliationReceipt {
            observation,
            outcome,
        };
        self.reconciliation = Some(receipt);
        Ok(receipt)
    }

    fn shutdown(&mut self) -> UiFocusHostPlacementShutdownReport {
        debug_assert_eq!(
            self.persistence,
            crate::runtime::UiServiceStatePersistencePosture::Effecting
        );
        let abandoned_indeterminate_request = self
            .require_available()
            .err()
            .and_then(|_| self.last.map(|last| last.request().identity()));
        self.last = None;
        self.reconciliation = None;
        UiFocusHostPlacementShutdownReport {
            abandoned_indeterminate_request,
        }
    }
}

impl UiFocusHostPlacementReconciliationReceipt {
    pub const fn observation(self) -> worth_ui_host_contract::UiHostFocusPlacementObservation {
        self.observation
    }

    pub const fn outcome(self) -> UiFocusHostPlacementReconciliationOutcome {
        self.outcome
    }
}

impl UiFocusHostPlacementShutdownReport {
    pub const fn abandoned_indeterminate_request(
        self,
    ) -> Option<worth_ui_host_contract::UiHostFocusPlacementRequestIdentity> {
        self.abandoned_indeterminate_request
    }
}
