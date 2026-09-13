use super::state::UiRebindReservation;

mod detached;
mod progression;

pub(crate) use detached::UiDetachedRebindRecovery;

pub use progression::{
    UiRebindReconciliation, UiRebindReconciliationRequest, UiRebindRecoveryCompletionHandle,
    UiRebindRecoveryDenial, UiRebindRecoveryDenialCause, UiRebindRecoveryInternalDefect,
    UiRebindRecoveryInternalDefectKind, UiRebindRecoveryOutcome, UiRebindRecoveryReceipt,
    UiRebindRecoverySurfaceDenial,
};

pub struct UiRebindRecoveryHandle<'session> {
    plan: crate::runtime::rebind::UiRebindPlan,
    registration: UiRebindReservation,
    basis: UiRebindRecoveryBasis<'session>,
}

enum UiRebindRecoveryBasis<'session> {
    Initial(Box<crate::facade::WorthUiMountedApplicationReplacementIndeterminate<'session>>),
    Content(Box<crate::facade::entry::WorthUiMountedContentRebindIndeterminate<'session>>),
    Reconciliation {
        authority: crate::facade::entry::WorthUiRebindRecoveryAuthority<'session>,
        frame: crate::mounting::UiMountedIndeterminateFrame,
    },
}

impl<'session> UiRebindRecoveryHandle<'session> {
    pub(super) fn new(
        plan: crate::runtime::rebind::UiRebindPlan,
        registration: UiRebindReservation,
        inner: Box<crate::facade::WorthUiMountedApplicationReplacementIndeterminate<'session>>,
    ) -> Self {
        Self {
            plan,
            registration,
            basis: UiRebindRecoveryBasis::Initial(inner),
        }
    }

    pub(super) fn after_reconciliation(
        plan: crate::runtime::rebind::UiRebindPlan,
        registration: UiRebindReservation,
        authority: crate::facade::entry::WorthUiRebindRecoveryAuthority<'session>,
        frame: crate::mounting::UiMountedIndeterminateFrame,
    ) -> Self {
        Self {
            plan,
            registration,
            basis: UiRebindRecoveryBasis::Reconciliation { authority, frame },
        }
    }

    pub(super) fn content(
        plan: crate::runtime::rebind::UiRebindPlan,
        registration: UiRebindReservation,
        inner: Box<crate::facade::entry::WorthUiMountedContentRebindIndeterminate<'session>>,
    ) -> Self {
        Self {
            plan,
            registration,
            basis: UiRebindRecoveryBasis::Content(inner),
        }
    }

    pub const fn plan(&self) -> &crate::runtime::rebind::UiRebindPlan {
        &self.plan
    }

    pub fn frame(&self) -> &crate::mounting::UiMountedIndeterminateFrame {
        self.basis.frame()
    }

    pub fn begin_reconciliation(self) -> UiRebindReconciliation<'session> {
        let Self {
            plan,
            registration,
            basis,
        } = self;
        let affected_bindings = basis
            .frame()
            .report()
            .affected_bindings()
            .to_vec()
            .into_boxed_slice();
        UiRebindReconciliation::new(
            plan,
            registration,
            basis.into_authority(),
            affected_bindings,
        )
    }

    pub(crate) fn into_recovery_authority_for_shutdown(
        self,
    ) -> crate::facade::entry::WorthUiRebindRecoveryAuthority<'session> {
        let Self {
            plan,
            registration,
            basis,
        } = self;
        drop((plan, registration));
        basis.into_authority()
    }
}

impl<'session> UiRebindRecoveryBasis<'session> {
    fn frame(&self) -> &crate::mounting::UiMountedIndeterminateFrame {
        match self {
            Self::Initial(inner) => inner.frame(),
            Self::Content(inner) => inner.frame(),
            Self::Reconciliation { frame, .. } => frame,
        }
    }

    fn into_authority(self) -> crate::facade::entry::WorthUiRebindRecoveryAuthority<'session> {
        match self {
            Self::Initial(inner) => {
                crate::facade::entry::WorthUiRebindRecoveryAuthority::from_indeterminate(inner)
            }
            Self::Content(inner) => {
                crate::facade::entry::WorthUiRebindRecoveryAuthority::from_content_indeterminate(
                    inner,
                )
            }
            Self::Reconciliation { authority, frame } => {
                drop(frame);
                authority
            }
        }
    }
}
