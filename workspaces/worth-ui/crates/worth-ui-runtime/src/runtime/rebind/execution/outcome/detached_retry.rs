use super::UiRebindDenialReceipt;

pub(crate) struct UiDetachedRebindRetry {
    plan: crate::runtime::rebind::UiRebindPlan,
    registration: crate::runtime::rebind::UiRebindReservation,
    inner: UiDetachedRebindRetryInner,
}

enum UiDetachedRebindRetryInner {
    Changed(crate::facade::WorthUiDetachedPreparedMountedApplicationReplacement),
    Content(crate::facade::entry::WorthUiDetachedPreparedMountedContentRebind),
}

impl UiDetachedRebindRetry {
    pub(crate) fn is_theme_switch(&self) -> bool {
        match &self.inner {
            UiDetachedRebindRetryInner::Content(content) => content.is_theme_switch(),
            UiDetachedRebindRetryInner::Changed(_) => false,
        }
    }

    pub(crate) fn reconcile_theme_and_retry<'session>(
        mut self,
        session: &'session mut crate::facade::WorthUiActiveApplicationSession,
        replacements: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
        now_tick: u64,
    ) -> Result<
        crate::runtime::rebind::UiRebindOutcome<'session>,
        crate::runtime::rebind::UiRebindPreparationDenial,
    > {
        self.inner = match self.inner {
            UiDetachedRebindRetryInner::Content(content) => UiDetachedRebindRetryInner::Content(
                content.with_theme_reconciliation(replacements)?,
            ),
            UiDetachedRebindRetryInner::Changed(_) => {
                return Err(
                    crate::runtime::rebind::UiRebindPreparationDenial::CandidateBindingMismatch,
                )
            }
        };
        self.rebase_content_and_retry(session, now_tick)
    }

    pub(super) fn from_denial(
        mut denial: UiRebindDenialReceipt<'_>,
    ) -> Result<Self, UiRebindDenialReceipt<'_>> {
        let Some(prepared) = denial.retry.take() else {
            return Err(denial);
        };
        let super::super::preparation::UiPreparedRebind {
            plan,
            reservation,
            kind,
        } = *prepared;
        let inner = match kind {
            super::super::preparation::UiPreparedRebindKind::Changed(replacement) => {
                UiDetachedRebindRetryInner::Changed(replacement.detach())
            }
            super::super::preparation::UiPreparedRebindKind::Content(content) => {
                UiDetachedRebindRetryInner::Content(content.detach())
            }
            super::super::preparation::UiPreparedRebindKind::EvidenceOnly(prepared) => {
                denial.retry = Some(Box::new(super::super::preparation::UiPreparedRebind {
                    plan,
                    reservation,
                    kind: super::super::preparation::UiPreparedRebindKind::EvidenceOnly(prepared),
                }));
                return Err(denial);
            }
        };
        Ok(Self {
            plan,
            registration: reservation,
            inner,
        })
    }

    pub(crate) fn session_identity(
        &self,
    ) -> crate::facade::WorthUiActiveApplicationSessionIdentity {
        match &self.inner {
            UiDetachedRebindRetryInner::Changed(inner) => inner.session_identity(),
            UiDetachedRebindRetryInner::Content(inner) => inner.session_identity(),
        }
    }

    pub(crate) fn rebase_content_and_retry<'session>(
        self,
        session: &'session mut crate::facade::WorthUiActiveApplicationSession,
        now_tick: u64,
    ) -> Result<
        crate::runtime::rebind::UiRebindOutcome<'session>,
        crate::runtime::rebind::UiRebindPreparationDenial,
    > {
        let semantic_content = self.plan.content().clone();
        let kind = match self.inner {
            UiDetachedRebindRetryInner::Changed(inner) => {
                super::super::preparation::UiPreparedRebindKind::Changed(inner.attach(session))
            }
            UiDetachedRebindRetryInner::Content(inner) => {
                super::super::preparation::UiPreparedRebindKind::Content(
                    inner.rebase(session, semantic_content)?,
                )
            }
        };
        Ok(super::super::preparation::UiPreparedRebind {
            plan: self.plan,
            reservation: self.registration,
            kind,
        }
        .execute(now_tick))
    }
}
