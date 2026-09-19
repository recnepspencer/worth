/// The rebind owner retains its plan and recovery reservation across native
/// physical progress; the shell retains the corresponding mounted frame phase.
pub(crate) struct UiDetachedRebindRecovery {
    session_identity: crate::facade::WorthUiActiveApplicationSessionIdentity,
    plan: crate::runtime::rebind::UiRebindPlan,
    registration: super::UiRebindReservation,
    affected_bindings: Box<[crate::mounting::UiSurfaceBindingGeneration]>,
}

impl super::UiRebindRecoveryHandle<'_> {
    pub(crate) fn detach_for_native(
        self,
    ) -> (
        UiDetachedRebindRecovery,
        crate::mounting::UiMountedIndeterminateFrame,
    ) {
        let Self {
            plan,
            registration,
            basis,
        } = self;
        let (session, frame) = match basis {
            super::UiRebindRecoveryBasis::Initial(inner) => inner.into_recovery_parts(),
            super::UiRebindRecoveryBasis::Content(inner) => inner.into_parts(),
            super::UiRebindRecoveryBasis::Reconciliation { authority, frame } => {
                (authority.into_session(), frame)
            }
        };
        let affected_bindings = frame
            .report()
            .affected_bindings()
            .to_vec()
            .into_boxed_slice();
        (
            UiDetachedRebindRecovery {
                session_identity: session.session_identity(),
                plan,
                registration,
                affected_bindings,
            },
            frame,
        )
    }
}

impl UiDetachedRebindRecovery {
    pub(crate) fn from_indeterminate_posture(
        session_identity: crate::facade::WorthUiActiveApplicationSessionIdentity,
        plan: crate::runtime::rebind::UiRebindPlan,
        registration: super::UiRebindReservation,
        frame: &crate::mounting::UiMountedIndeterminateFrame,
    ) -> Self {
        Self {
            session_identity,
            plan,
            registration,
            affected_bindings: frame
                .report()
                .affected_bindings()
                .to_vec()
                .into_boxed_slice(),
        }
    }

    pub(crate) fn session_identity(
        &self,
    ) -> crate::facade::WorthUiActiveApplicationSessionIdentity {
        self.session_identity
    }

    pub(crate) fn settle(
        self,
        mounted: crate::mounting::UiMountedFramePublicationReceipt,
    ) -> super::UiRebindRecoveryReceipt {
        let Self {
            plan,
            registration,
            affected_bindings,
            ..
        } = self;
        drop(registration);
        super::UiRebindRecoveryReceipt::from_reconciled(plan, mounted, affected_bindings)
    }
}
