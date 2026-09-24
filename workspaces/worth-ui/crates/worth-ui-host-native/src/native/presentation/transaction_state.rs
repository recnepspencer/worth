use worth_ui_host_contract::UiHostSurfacePresentationDenial;

use super::{
    port, UiNativePresentationFailure, UiNativePresentationPortFailure, UiNativeResourceClass,
    UiNativeResourceRegistry,
};

pub(crate) struct UiNativePresentationOwners {
    readback: crate::native::UiNativeResourceOwner,
    submission: crate::native::UiNativeResourceOwner,
    physical_work: crate::native::physical_work_signal::UiNativePhysicalPresentationIdentity,
    physical_token: crate::native::physical_work_signal::UiNativePhysicalSignalRequestToken,
}

pub(crate) struct UiNativePendingPresentation {
    external: Option<Box<dyn UiNativePendingExternalObligation>>,
    readback_owner: Option<crate::native::UiNativeResourceOwner>,
    submission_owner: Option<crate::native::UiNativeResourceOwner>,
    physical_work: Box<crate::native::physical_work_signal::UiNativePhysicalPresentationIdentity>,
    physical_token: Box<crate::native::physical_work_signal::UiNativePhysicalSignalRequestToken>,
    completion_identity: Option<u64>,
    cursor: Option<winit::window::CursorIcon>,
    settlement: Option<Box<super::UiNativePendingSurfaceSettlement>>,
    completion: UiNativePendingPresentationCompletion,
}

pub(crate) enum UiNativePendingPresentationCompletion {
    Pending,
    Presented(Box<port::UiNativePresentationPortObservation>),
    Superseded(Box<port::UiNativePresentationPortObservation>),
    Indeterminate,
}

pub(crate) trait UiNativePendingExternalObligation {
    fn poll_observation(
        &mut self,
        basis: crate::native::physical_work_signal::UiNativePhysicalSignalExternalBasis,
        device: Option<&wgpu::Device>,
    ) -> crate::native::physical_work_signal::UiNativePhysicalSignalExternalObservation;

    fn take_presented_observation(&mut self) -> Option<port::UiNativePresentationPortObservation> {
        None
    }

    #[cfg(feature = "certification-support")]
    fn take_duplicate_completed_observation(
        &mut self,
        _basis: crate::native::physical_work_signal::UiNativePhysicalSignalExternalBasis,
    ) -> Option<crate::native::physical_work_signal::UiNativePhysicalSignalExternalObservation>
    {
        None
    }
}

impl UiNativePendingPresentation {
    fn external(
        external: Box<dyn UiNativePendingExternalObligation>,
        readback_owner: crate::native::UiNativeResourceOwner,
        submission_owner: crate::native::UiNativeResourceOwner,
        physical_work: crate::native::physical_work_signal::UiNativePhysicalPresentationIdentity,
        physical_token: crate::native::physical_work_signal::UiNativePhysicalSignalRequestToken,
    ) -> Self {
        Self {
            external: Some(external),
            readback_owner: Some(readback_owner),
            submission_owner: Some(submission_owner),
            physical_work: Box::new(physical_work),
            physical_token: Box::new(physical_token),
            completion_identity: None,
            cursor: None,
            settlement: None,
            completion: UiNativePendingPresentationCompletion::Pending,
        }
    }

    pub(crate) fn with_settlement(
        mut self,
        settlement: super::UiNativePendingSurfaceSettlement,
    ) -> Self {
        debug_assert!(self.settlement.is_none());
        self.settlement = Some(Box::new(settlement));
        self
    }

    pub(crate) fn bind_completion_identity(
        &mut self,
        identity: u64,
        cursor: Option<winit::window::CursorIcon>,
    ) -> bool {
        if self.completion_identity.is_some() || identity == 0 {
            return false;
        }
        self.completion_identity = Some(identity);
        self.cursor = cursor;
        true
    }

    pub(crate) const fn prepared_cursor(&self) -> Option<winit::window::CursorIcon> {
        self.cursor
    }

    pub(crate) const fn completion_identity(&self) -> Option<u64> {
        self.completion_identity
    }

    pub(crate) fn take_completion(&mut self) -> UiNativePendingPresentationCompletion {
        std::mem::replace(
            &mut self.completion,
            UiNativePendingPresentationCompletion::Pending,
        )
    }

    pub(crate) const fn has_active_external(&self) -> bool {
        self.external.is_some()
    }

    pub(crate) fn take_settlement(&mut self) -> Option<super::UiNativePendingSurfaceSettlement> {
        self.settlement.take().map(|settlement| *settlement)
    }

    pub(crate) fn replace_settlement(
        &mut self,
        settlement: super::UiNativePendingSurfaceSettlement,
    ) {
        debug_assert!(self.settlement.is_none());
        self.settlement = Some(Box::new(settlement));
    }

    pub(crate) fn inherit_predecessor_settlement(
        &mut self,
        predecessor: super::UiNativePendingSurfaceSettlement,
        cursor: Option<winit::window::CursorIcon>,
    ) -> Result<(), Box<super::UiNativePendingSurfaceSettlement>> {
        let Some(successor) = self.settlement.as_mut() else {
            return Err(Box::new(predecessor));
        };
        successor.inherit_predecessor(predecessor)?;
        self.cursor = self.cursor.or(cursor);
        Ok(())
    }

    pub(crate) const fn has_settlement(&self) -> bool {
        self.settlement.is_some()
    }

    pub(crate) fn mark_presented(
        &mut self,
        observation: port::UiNativePresentationPortObservation,
    ) {
        self.completion = UiNativePendingPresentationCompletion::Presented(Box::new(observation));
    }

    pub(crate) fn mark_superseded(
        &mut self,
        observation: port::UiNativePresentationPortObservation,
    ) {
        self.completion = UiNativePendingPresentationCompletion::Superseded(Box::new(observation));
    }

    pub(crate) fn mark_indeterminate(&mut self) {
        self.completion = UiNativePendingPresentationCompletion::Indeterminate;
    }

    pub(crate) fn consume_completion_identity(&mut self) {
        self.completion_identity = None;
    }

    pub(crate) const fn physical_work(
        &self,
    ) -> crate::native::physical_work_signal::UiNativePhysicalPresentationIdentity {
        *self.physical_work
    }

    pub(crate) const fn physical_basis(
        &self,
    ) -> crate::native::physical_work_signal::UiNativePhysicalPresentationBasis {
        self.physical_work.basis()
    }

    pub(crate) const fn physical_token(
        &self,
    ) -> crate::native::physical_work_signal::UiNativePhysicalSignalRequestToken {
        *self.physical_token
    }

    pub(crate) fn refresh_physical_token(
        &mut self,
        token: crate::native::physical_work_signal::UiNativePhysicalSignalRequestToken,
    ) -> bool {
        if token.work()
            != crate::native::physical_work_signal::UiNativePhysicalSignalWork::Presentation(
                *self.physical_work,
            )
        {
            return false;
        }
        *self.physical_token = token;
        true
    }

    pub(crate) fn poll_observation(
        &mut self,
        device: Option<&wgpu::Device>,
    ) -> crate::native::physical_work_signal::UiNativePhysicalSignalExternalObservation {
        self.external
            .as_mut()
            .expect("active physical presentation retains its external obligation")
            .poll_observation(self.physical_token.external_basis(), device)
    }

    pub(crate) fn take_presented_observation(
        &mut self,
    ) -> Option<port::UiNativePresentationPortObservation> {
        self.external
            .as_mut()
            .and_then(|external| external.take_presented_observation())
    }

    #[cfg(feature = "certification-support")]
    pub(crate) fn qualify_external_observation(
        &mut self,
        effects_indeterminate: bool,
        duplicate_completed: bool,
    ) {
        if !effects_indeterminate && !duplicate_completed {
            return;
        }
        let external = self
            .external
            .take()
            .expect("qualification decorates one retained external obligation");
        self.external = Some(Box::new(super::UiNativeQualifiedExternalObligation::new(
            external,
            effects_indeterminate,
            duplicate_completed,
        )));
    }

    #[cfg(feature = "certification-support")]
    pub(crate) fn take_duplicate_completed_observation(
        &mut self,
    ) -> Option<crate::native::physical_work_signal::UiNativePhysicalSignalExternalObservation>
    {
        self.external.as_mut().and_then(|external| {
            external.take_duplicate_completed_observation(self.physical_token.external_basis())
        })
    }

    pub(crate) fn release_external(&mut self, resources: &mut UiNativeResourceRegistry) {
        drop(self.external.take());
        if let Some(readback_owner) = self.readback_owner.take() {
            resources
                .release(readback_owner)
                .expect("settled readback owner must remain exact");
        }
        if let Some(submission_owner) = self.submission_owner.take() {
            resources
                .release(submission_owner)
                .expect("settled submission owner must remain exact");
        }
    }

    /// Stands a settled presentation's finished external work back up for a
    /// recovery admitted after its settlement.
    pub(crate) fn await_settled_recovery(&mut self) {
        debug_assert!(self.external.is_none());
        self.external = Some(Box::new(
            super::settled_external_obligation::UiNativeSettledExternalObligation,
        ));
    }

    pub(crate) fn release(mut self, resources: &mut UiNativeResourceRegistry) {
        self.release_external(resources);
    }
}

pub(crate) fn reserve_presentation_owners(
    resources: &mut UiNativeResourceRegistry,
    physical_signal: &mut crate::native::physical_work_signal::UiNativePhysicalSignalOwner,
    basis: crate::native::physical_work_signal::UiNativePhysicalPresentationBasis,
) -> Result<UiNativePresentationOwners, UiNativePresentationFailure> {
    let mut owners = resources
        .reserve(&[
            UiNativeResourceClass::ReadbackBuffer,
            UiNativeResourceClass::PendingSubmission,
        ])
        .map_err(|_| {
            UiNativePresentationFailure::BeforeEffects(
                UiHostSurfacePresentationDenial::CapacityExceeded,
            )
        })?;
    let readback = owners.remove(0);
    let submission = owners.remove(0);
    let physical_work = match physical_signal.admit_presentation(basis) {
        Ok(work) => work,
        Err(()) => {
            resources
                .release(readback)
                .expect("unused readback reservation must release exactly");
            resources
                .release(submission)
                .expect("unused submission reservation must release exactly");
            return Err(UiNativePresentationFailure::BeforeEffects(
                UiHostSurfacePresentationDenial::CapacityExceeded,
            ));
        }
    };
    let physical_token = physical_signal
        .take_initial_presentation(physical_work)
        .expect("new physical presentation work must issue one exact wake");
    Ok(UiNativePresentationOwners {
        readback,
        submission,
        physical_work,
        physical_token,
    })
}

pub(crate) fn settle_port_result(
    resources: &mut UiNativeResourceRegistry,
    physical_signal: &mut crate::native::physical_work_signal::UiNativePhysicalSignalOwner,
    owners: UiNativePresentationOwners,
    result: Result<port::UiNativePresentationPortObservation, UiNativePresentationPortFailure>,
) -> Result<port::UiNativePresentationPortObservation, UiNativePresentationFailure> {
    match result {
        Ok(observation) => {
            let settled = physical_signal.reconcile(owners.physical_token.observe(
                crate::native::physical_work_signal::UiNativePhysicalSignalStatus::Completed,
            ));
            assert!(matches!(
                settled,
                crate::native::physical_work_signal::UiNativePhysicalSignalSettlement::Completed
            ));
            release_reserved(resources, owners);
            Ok(observation)
        }
        Err(UiNativePresentationPortFailure::Surface(surface_failure)) => {
            let settled = physical_signal.reconcile(owners.physical_token.observe(
                crate::native::physical_work_signal::UiNativePhysicalSignalStatus::RejectedBeforeEffects,
            ));
            assert!(matches!(
                settled,
                crate::native::physical_work_signal::UiNativePhysicalSignalSettlement::Rejected
            ));
            release_reserved(resources, owners);
            match crate::native::lifecycle::UiNativeLifecycleOrchestrator::classify_surface_failure(
                surface_failure,
            ) {
                super::UiNativeSurfaceFailureDisposition::ReconstructionRequired(recovery) => {
                    Err(UiNativePresentationFailure::RecoveryRequired {
                        denial: UiHostSurfacePresentationDenial::ReconstructionRequired,
                        cause: recovery.cause(),
                    })
                }
                super::UiNativeSurfaceFailureDisposition::RetryAfterTimeout => {
                    Err(UiNativePresentationFailure::BeforeEffects(
                        UiHostSurfacePresentationDenial::ExternalTimeout,
                    ))
                }
                super::UiNativeSurfaceFailureDisposition::WaitForVisibility => {
                    Err(UiNativePresentationFailure::BeforeEffects(
                        UiHostSurfacePresentationDenial::SurfaceOccluded,
                    ))
                }
                super::UiNativeSurfaceFailureDisposition::ValidationRejected => {
                    Err(UiNativePresentationFailure::BeforeEffects(
                        UiHostSurfacePresentationDenial::ExternalValidationFailed,
                    ))
                }
            }
        }
        Err(UiNativePresentationPortFailure::ReadbackUnsettled(external)) => {
            let settled = physical_signal.reconcile(owners.physical_token.observe(
                crate::native::physical_work_signal::UiNativePhysicalSignalStatus::Pending,
            ));
            assert!(matches!(
                settled,
                crate::native::physical_work_signal::UiNativePhysicalSignalSettlement::Pending
            ));
            Err(UiNativePresentationFailure::Pending(
                UiNativePendingPresentation::external(
                    external,
                    owners.readback,
                    owners.submission,
                    owners.physical_work,
                    owners.physical_token,
                ),
            ))
        }
    }
}

fn release_reserved(resources: &mut UiNativeResourceRegistry, owners: UiNativePresentationOwners) {
    resources
        .release(owners.readback)
        .expect("readback reservation must remain exact");
    resources
        .release(owners.submission)
        .expect("submission reservation must remain exact");
}
