use super::UiRebindReservation;

mod presentation;

pub struct UiRebindReconciliation<'session> {
    pub(super) plan: crate::runtime::rebind::UiRebindPlan,
    pub(super) registration: UiRebindReservation,
    pub(super) authority: crate::facade::entry::WorthUiRebindRecoveryAuthority<'session>,
    pub(super) affected_bindings: Box<[crate::mounting::UiSurfaceBindingGeneration]>,
}

pub struct UiRebindReconciliationRequest {
    replacements: Box<[crate::mounting::UiMountedSurfaceReconciliationBinding]>,
    deadline: worth_ui_host_contract::UiPresentationDeadline,
}

pub enum UiRebindRecoveryOutcome<'session> {
    Recovered(UiRebindRecoveryReceipt),
    RejectedBeforeEffects(Box<UiRebindRecoveryDenial<'session>>),
    InFlight(UiRebindRecoveryCompletionHandle<'session>),
    Indeterminate(super::UiRebindRecoveryHandle<'session>),
    InternalDefect(UiRebindRecoveryInternalDefect<'session>),
}

pub struct UiRebindRecoveryDenial<'session> {
    pub(super) cause: UiRebindRecoveryDenialCause,
    pub(super) reconciliation: UiRebindReconciliation<'session>,
    pub(super) request: UiRebindReconciliationRequest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiRebindRecoveryDenialCause {
    MountedIdentity(crate::mounting::UiMountedIdentityDenial),
    MountedRetention(crate::mounting::UiMountedFrameRetentionDenial),
    MountedPresentation(crate::mounting::UiMountedPresentationAdmissionDenial),
    HostRejectedBeforeEffects,
}

#[must_use = "recovery completion must be completed, disposed, or dropped through cancellation"]
pub struct UiRebindRecoveryCompletionHandle<'session> {
    pub(super) state: Option<Box<UiRebindRecoveryCompletionState<'session>>>,
}

pub(super) struct UiRebindRecoveryCompletionState<'session> {
    pub(super) reconciliation: UiRebindReconciliation<'session>,
    pub(super) in_flight: crate::mounting::UiMountedPresentationInFlight,
    pub(super) request: UiRebindReconciliationRequest,
}

pub struct UiRebindRecoveryReceipt {
    pub(super) plan: crate::runtime::rebind::UiRebindPlan,
    pub(super) mounted: crate::mounting::UiMountedFramePublicationReceipt,
    pub(super) affected_bindings: Box<[crate::mounting::UiSurfaceBindingGeneration]>,
}

pub struct UiRebindRecoveryInternalDefect<'session> {
    pub(super) kind: UiRebindRecoveryInternalDefectKind,
    pub(super) reconciliation: UiRebindReconciliation<'session>,
    pub(super) unexpected_publication: Option<crate::mounting::UiMountedFramePublicationReceipt>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiRebindRecoveryInternalDefectKind {
    UnexpectedPublicationPosture,
    CompletionAuthorityRejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiRebindRecoverySurfaceDenial {
    BindingNotAffected,
    MountedIdentity(crate::mounting::UiMountedIdentityDenial),
}

impl<'session> UiRebindReconciliation<'session> {
    pub(super) fn new(
        plan: crate::runtime::rebind::UiRebindPlan,
        registration: UiRebindReservation,
        authority: crate::facade::entry::WorthUiRebindRecoveryAuthority<'session>,
        affected_bindings: Box<[crate::mounting::UiSurfaceBindingGeneration]>,
    ) -> Self {
        Self {
            plan,
            registration,
            authority,
            affected_bindings,
        }
    }

    pub fn affected_bindings(&self) -> &[crate::mounting::UiSurfaceBindingGeneration] {
        &self.affected_bindings
    }

    pub fn rebind_surface(
        &mut self,
        affected: crate::mounting::UiSurfaceBindingGeneration,
        mode: worth_ui_host_contract::UiHostSurfacePresentationMode,
        profile: crate::mounting::UiSurfaceBindingProfile,
    ) -> Result<crate::mounting::UiSurfaceBindingIdentityView, UiRebindRecoverySurfaceDenial> {
        if !self.affected_bindings.contains(&affected) {
            return Err(UiRebindRecoverySurfaceDenial::BindingNotAffected);
        }
        self.authority
            .rebind_host_surface(affected, mode, profile)
            .map_err(UiRebindRecoverySurfaceDenial::MountedIdentity)
    }

    pub(crate) fn into_recovery_authority_for_shutdown(
        self,
    ) -> crate::facade::entry::WorthUiRebindRecoveryAuthority<'session> {
        let Self {
            plan,
            registration,
            authority,
            affected_bindings,
        } = self;
        drop((plan, registration, affected_bindings));
        authority
    }
}

impl UiRebindReconciliationRequest {
    pub fn new(
        replacements: Box<[crate::mounting::UiMountedSurfaceReconciliationBinding]>,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
    ) -> Self {
        Self {
            replacements,
            deadline,
        }
    }

    pub fn replacements(&self) -> &[crate::mounting::UiMountedSurfaceReconciliationBinding] {
        &self.replacements
    }

    pub const fn deadline(&self) -> worth_ui_host_contract::UiPresentationDeadline {
        self.deadline
    }
}

impl<'session> UiRebindRecoveryDenial<'session> {
    pub const fn cause(&self) -> UiRebindRecoveryDenialCause {
        self.cause
    }

    pub fn retry(self, now_tick: u64) -> UiRebindRecoveryOutcome<'session> {
        self.reconciliation.present_current(self.request, now_tick)
    }

    pub fn into_reconciliation(self) -> UiRebindReconciliation<'session> {
        self.reconciliation
    }
}

impl UiRebindRecoveryReceipt {
    pub(super) fn from_reconciled(
        plan: crate::runtime::rebind::UiRebindPlan,
        mounted: crate::mounting::UiMountedFramePublicationReceipt,
        affected_bindings: Box<[crate::mounting::UiSurfaceBindingGeneration]>,
    ) -> Self {
        Self {
            plan,
            mounted,
            affected_bindings,
        }
    }

    pub const fn plan(&self) -> &crate::runtime::rebind::UiRebindPlan {
        &self.plan
    }

    pub const fn mounted(&self) -> &crate::mounting::UiMountedFramePublicationReceipt {
        &self.mounted
    }

    pub fn affected_bindings(&self) -> &[crate::mounting::UiSurfaceBindingGeneration] {
        &self.affected_bindings
    }

    pub const fn predecessor_remains_current(&self) -> bool {
        true
    }
}

impl<'session> UiRebindRecoveryInternalDefect<'session> {
    pub const fn kind(&self) -> UiRebindRecoveryInternalDefectKind {
        self.kind
    }

    pub const fn publication_occurred(&self) -> bool {
        self.unexpected_publication.is_some()
    }

    pub(crate) fn into_recovery_authority_for_shutdown(
        self,
    ) -> crate::facade::entry::WorthUiRebindRecoveryAuthority<'session> {
        self.reconciliation.into_recovery_authority_for_shutdown()
    }
}
