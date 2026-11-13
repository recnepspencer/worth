//! Cleanup publication uses the existing serialized Query history owner.
use super::*;
use journal::*;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::ManagedCleanupObligationIdentity;

pub(in crate::domain_computation::primary_graph) struct ManagedCleanupPublication {
    history: Arc<QueryObligationHistory>,
    identity: ManagedCleanupObligationIdentity,
    pub(in crate::domain_computation::primary_graph) pending_denial: Option<QueryObligationDenial>,
}
impl ManagedCleanupPublication {
    pub(in crate::domain_computation::primary_graph) fn finish_retention(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryObligationDenial> {
        let medium = self
            .history
            .medium
            .get()
            .ok_or(QueryObligationDenial::SupportUnavailable)?;
        let settlement = self
            .history
            .lock_publication()?
            .finish_pending(medium.as_ref(), request);
        self.admit_retention_settlement(settlement)
    }
    fn admit_retention_settlement(
        &mut self,
        settlement: Result<(), QueryObligationDenial>,
    ) -> Result<(), QueryObligationDenial> {
        let retained = self
            .history
            .managed_cleanup_obligations()
            .iter()
            .any(|row| row.identity == self.identity);
        // A shared release/failure publication's no-effect cannot revoke an
        // already acknowledged Retain. Only that Retain's own no-effect,
        // with its row absent, permits replacement of this identity.
        match settlement {
            Err(QueryObligationDenial::PublicationNoEffect) if retained => {}
            other => other?,
        }
        if !retained {
            return Err(QueryObligationDenial::UnknownObligation);
        }
        self.pending_denial = None;
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph) fn identity(
        &self,
    ) -> ManagedCleanupObligationIdentity {
        self.identity
    }
    pub(in crate::domain_computation::primary_graph) fn record_failure(
        &self,
        request: &ExecutionRequest<'_, '_>,
        failure: &crate::domain_computation::WorthQueryMutationCleanupFailure,
    ) -> Result<(), QueryObligationDenial> {
        let medium = self
            .history
            .medium
            .get()
            .ok_or(QueryObligationDenial::SupportUnavailable)?;
        let driver = self.history.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        let rows = self.history.managed_cleanup_obligations();
        let row = rows
            .iter()
            .find(|row| row.identity == self.identity)
            .ok_or(QueryObligationDenial::UnknownObligation)?;
        if row.run != failure.run_identity() {
            return Err(QueryObligationDenial::ObligationSubjectMismatch);
        }
        if row.failure == failure.kind() {
            return Ok(());
        }
        driver.publish(
            medium.as_ref(),
            QueryManagedCleanupRecord::Failed {
                identity: self.identity,
                failure: failure.kind(),
            },
            request,
        )
    }
    pub(in crate::domain_computation::primary_graph) fn record_release(
        &self,
        request: &ExecutionRequest<'_, '_>,
        native: &crate::domain_computation::WorthQueryDirectRunCleanupReceipt,
    ) -> Result<(), QueryObligationDenial> {
        let inspection = native.inspection();
        if !inspection.resources_released()
            || inspection.provider_work().provider_retained_bytes() != 0
        {
            return Err(QueryObligationDenial::CleanupReleaseUnrecorded {
                identity: self.identity,
            });
        }
        let medium = self
            .history
            .medium
            .get()
            .ok_or(QueryObligationDenial::SupportUnavailable)?;
        let driver = self.history.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        {
            let state = self.history.state();
            let row = state
                .journal
                .as_ref()
                .and_then(|journal| {
                    journal
                        .ledger
                        .managed_cleanup()
                        .find(|row| row.identity == self.identity)
                })
                .ok_or(QueryObligationDenial::UnknownObligation)?;
            if row.run != inspection.run_identity() {
                return Err(QueryObligationDenial::ObligationSubjectMismatch);
            }
            if row.release.is_some() {
                return Ok(());
            }
        }
        driver.publish(
            medium.as_ref(),
            QueryManagedCleanupRecord::Released {
                identity: self.identity,
                release: QueryManagedCleanupRelease {
                    terminal: inspection.terminal(),
                    disposition: inspection.disposition(),
                },
            },
            request,
        )
    }
}
impl QueryObligationHistory {
    pub(in crate::domain_computation::primary_graph) fn retain_managed_cleanup(
        self: &Arc<Self>,
        request: &ExecutionRequest<'_, '_>,
        failure: &crate::domain_computation::WorthQueryMutationCleanupFailure,
        original: Option<QueryManagedCleanupOriginal>,
    ) -> Result<Option<ManagedCleanupPublication>, QueryObligationDenial> {
        let Some(medium) = self.medium.get() else {
            return Ok(None);
        };
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        let identity = {
            let mut state = self.state();
            let journal = state
                .journal
                .as_mut()
                .ok_or(QueryObligationDenial::SupportUnavailable)?;
            let position = journal
                .ledger
                .position()
                .checked_add(1)
                .and_then(std::num::NonZeroU64::new)
                .ok_or(QueryObligationDenial::IdentityExhausted)?;
            let identity = ManagedCleanupObligationIdentity::from_recorded(medium.home(), position);
            journal.prepare_pending(QueryManagedCleanupRecord::Retain(
                QueryManagedCleanupObligation {
                    identity,
                    run: failure.run_identity().to_owned(),
                    failure: failure.kind(),
                    original,
                    release: None,
                },
            ))?;
            identity
        };
        let pending_denial = driver.finish_pending(medium.as_ref(), request).err();
        if pending_denial == Some(QueryObligationDenial::PublicationNoEffect) {
            return Err(QueryObligationDenial::PublicationNoEffect);
        }
        Ok(Some(ManagedCleanupPublication {
            history: Arc::clone(self),
            identity,
            pending_denial,
        }))
    }

    pub(in crate::domain_computation::primary_graph) fn managed_cleanup_obligations(
        &self,
    ) -> Vec<QueryManagedCleanupObligation> {
        self.state()
            .journal
            .as_ref()
            .map(|journal| journal.ledger.managed_cleanup().cloned().collect())
            .unwrap_or_default()
    }

    pub(in crate::domain_computation::primary_graph) fn discharge_managed_cleanup(
        &self,
        request: &ExecutionRequest<'_, '_>,
        identity: ManagedCleanupObligationIdentity,
    ) -> Result<(), QueryObligationDenial> {
        let medium = self
            .medium
            .get()
            .ok_or(QueryObligationDenial::SupportUnavailable)?;
        self.lock_publication()?.publish(
            medium.as_ref(),
            QueryManagedCleanupRecord::Discharge { identity },
            request,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn preflight_managed_cleanup_close(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryObligationDenial> {
        let Some(medium) = self.medium.get() else {
            return Ok(());
        };
        self.lock_publication()?
            .finish_pending(medium.as_ref(), request)?;
        if let Some(row) = self
            .managed_cleanup_obligations()
            .into_iter()
            .find(|row| row.release.is_none())
        {
            return Err(QueryObligationDenial::CleanupReleaseUnrecorded {
                identity: row.identity,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
