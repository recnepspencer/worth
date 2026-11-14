//! Managed work shares the existing serialized Query publication/admission owner.
use super::*;
use journal::*;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::ManagedRunObligationIdentity;
impl QueryObligationHistory {
    pub(in crate::domain_computation) fn retain_managed_run(
        &self,
        request: &ExecutionRequest<'_, '_>,
        logical_run: &str,
        attempt: &str,
    ) -> Result<
        Option<(ManagedRunObligationIdentity, Option<QueryObligationDenial>)>,
        QueryObligationDenial,
    > {
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
            let identity = ManagedRunObligationIdentity::from_recorded(medium.home(), position);
            journal.prepare_pending(QueryManagedRunRecord::Retain(QueryManagedRunObligation {
                identity,
                logical_run: logical_run.to_owned(),
                original_attempt: attempt.to_owned(),
                attempt: attempt.to_owned(),
                posture: QueryManagedRunPosture::Active,
                original: None,
                recovery: None,
            }))?;
            identity
        };
        let pending = driver.finish_pending(medium.as_ref(), request).err();
        Ok(Some((identity, pending)))
    }
    pub(in crate::domain_computation) fn settle_managed_run_retention(
        &self,
        request: &ExecutionRequest<'_, '_>,
        identity: ManagedRunObligationIdentity,
    ) -> Result<bool, QueryObligationDenial> {
        let medium = self
            .medium
            .get()
            .ok_or(QueryObligationDenial::SupportUnavailable)?;
        let settlement = self
            .lock_publication()?
            .finish_pending(medium.as_ref(), request);
        let retained = self
            .managed_run_obligations()
            .iter()
            .any(|row| row.identity == identity);
        match settlement {
            Ok(()) | Err(QueryObligationDenial::PublicationNoEffect) => Ok(retained),
            Err(cause) => Err(cause),
        }
    }
    pub(in crate::domain_computation) fn managed_run_obligations(
        &self,
    ) -> Vec<QueryManagedRunObligation> {
        self.state()
            .journal
            .as_ref()
            .map(|journal| journal.ledger.managed_runs().cloned().collect())
            .unwrap_or_default()
    }
    pub(in crate::domain_computation) fn record_managed_run_handoff(
        &self,
        request: &ExecutionRequest<'_, '_>,
        identity: ManagedRunObligationIdentity,
        posture: QueryManagedRunPosture,
        attempt: &str,
    ) -> Result<(), QueryObligationDenial> {
        let medium = self
            .medium
            .get()
            .ok_or(QueryObligationDenial::SupportUnavailable)?;
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        let rows = self.managed_run_obligations();
        let Some(row) = rows.iter().find(|row| row.identity == identity) else {
            // The original local carrier can outlive acknowledged logical disposal.
            return Ok(());
        };
        if row.posture == posture && row.attempt == attempt {
            return Ok(());
        }
        driver.publish(
            medium.as_ref(),
            QueryManagedRunRecord::Handoff {
                identity,
                posture,
                attempt: attempt.to_owned(),
            },
            request,
        )
    }
    pub(in crate::domain_computation) fn associate_managed_run_original(
        &self,
        request: &ExecutionRequest<'_, '_>,
        identity: ManagedRunObligationIdentity,
        original: QueryManagedCleanupOriginal,
    ) -> Result<(), QueryObligationDenial> {
        let medium = self
            .medium
            .get()
            .ok_or(QueryObligationDenial::SupportUnavailable)?;
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        let rows = self.managed_run_obligations();
        let Some(row) = rows.iter().find(|row| row.identity == identity) else {
            return Ok(());
        };
        if row.original == Some(original) {
            return Ok(());
        }
        driver.publish(
            medium.as_ref(),
            QueryManagedRunRecord::Original { identity, original },
            request,
        )
    }
    pub(in crate::domain_computation) fn associate_managed_run_recovery(
        &self,
        request: &ExecutionRequest<'_, '_>,
        identity: ManagedRunObligationIdentity,
        cause: QueryManagedRunRecovery,
    ) -> Result<(), QueryObligationDenial> {
        let medium = self
            .medium
            .get()
            .ok_or(QueryObligationDenial::SupportUnavailable)?;
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        let rows = self.managed_run_obligations();
        let Some(row) = rows.iter().find(|row| row.identity == identity) else {
            return Ok(());
        };
        if row.recovery == Some(cause) {
            return Ok(());
        }
        driver.publish(
            medium.as_ref(),
            QueryManagedRunRecord::Recovery { identity, cause },
            request,
        )
    }
    pub(in crate::domain_computation::primary_graph) fn preflight_managed_run_close(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryObligationDenial> {
        let Some(medium) = self.medium.get() else {
            return Ok(());
        };
        self.lock_publication()?
            .finish_pending(medium.as_ref(), request)?;
        if let Some(row) = self
            .managed_run_obligations()
            .iter()
            .find(|row| !row.posture.resources_released())
        {
            return Err(QueryObligationDenial::ManagedRunPending {
                identity: row.identity,
            });
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph) fn discharge_managed_run(
        &self,
        request: &ExecutionRequest<'_, '_>,
        identity: ManagedRunObligationIdentity,
    ) -> Result<(), QueryObligationDenial> {
        let medium = self
            .medium
            .get()
            .ok_or(QueryObligationDenial::SupportUnavailable)?;
        if identity.home() != medium.home() {
            return Err(QueryObligationDenial::ForeignHome);
        }
        let driver = self.lock_publication()?;
        let known = self
            .managed_run_obligations()
            .iter()
            .any(|row| row.identity == identity);
        driver.finish_pending(medium.as_ref(), request)?;
        if !self
            .managed_run_obligations()
            .iter()
            .any(|row| row.identity == identity)
        {
            return if known {
                Ok(())
            } else {
                Err(QueryObligationDenial::UnknownObligation)
            };
        }
        driver.publish(
            medium.as_ref(),
            QueryManagedRunRecord::Discharge { identity },
            request,
        )
    }
}
