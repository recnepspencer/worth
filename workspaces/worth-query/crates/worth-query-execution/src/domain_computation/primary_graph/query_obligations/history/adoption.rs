//! The existing serialized publisher owns adoption acquisition and discharge.
use super::*;
use journal::*;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::DurableCompositePublicationAttemptIdentity;
impl QueryObligationHistory {
    pub(in crate::domain_computation::primary_graph) fn retain_branch_adoption(
        &self,
        request: &ExecutionRequest<'_, '_>,
        row: QueryBranchAdoptionObligation,
    ) -> Result<(), QueryObligationDenial> {
        let Some(medium) = self.medium.get() else {
            return Ok(());
        };
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        if let Some(prior) = self
            .state()
            .journal
            .as_ref()
            .and_then(|journal| journal.ledger.adoption.branches.get(&row.attempt))
        {
            return if prior == &row {
                Ok(())
            } else {
                Err(QueryObligationDenial::ObligationSubjectMismatch)
            };
        }
        driver.publish(
            medium.as_ref(),
            QueryAdoptionRecord::RetainBranch(row),
            request,
        )
    }
    pub(in crate::domain_computation::primary_graph) fn release_branch_adoption(
        &self,
        request: &ExecutionRequest<'_, '_>,
        attempt: DurableCompositePublicationAttemptIdentity,
    ) -> Result<(), QueryObligationDenial> {
        let Some(medium) = self.medium.get() else {
            return Ok(());
        };
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        if !self.state().journal.as_ref().is_some_and(|journal| {
            journal.ledger.adoption.branches.contains_key(&attempt)
                && !journal.ledger.adoption.completed.contains(&attempt)
        }) {
            return Ok(());
        }
        driver.publish(
            medium.as_ref(),
            QueryAdoptionRecord::ReleaseBranch { attempt },
            request,
        )
    }
}

impl WorthQueryProgramSupportCustody {
    /// The acknowledged adoption record now owns these counts. Drop only ends
    /// the local reader and must not discharge durable logical custody.
    pub(in crate::domain_computation::primary_graph) fn transfer_to_durable(&mut self) {
        if !self.local {
            return;
        }
        QueryObligationHistory::release(&self.state, &self.source, SupportUse::Custody);
        if let Some(target) = &self.target {
            QueryObligationHistory::release(&self.state, target, SupportUse::Custody);
        }
        self.local = false;
    }
}
impl QueryObligationHistory {
    pub(in crate::domain_computation::primary_graph) fn readmit_adoption_custody(
        &self,
        row: &QueryBranchAdoptionObligation,
    ) -> Result<
        (
            ApplicationProgramRevision,
            ApplicationProgramRevision,
            WorthQueryProgramSupportCustody,
        ),
        QueryObligationDenial,
    > {
        let state = self.state();
        let journal = state
            .journal
            .as_ref()
            .ok_or(QueryObligationDenial::SupportUnavailable)?;
        if journal.ledger.adoption.branches.get(&row.attempt) != Some(row)
            || journal.ledger.adoption.completed.contains(&row.attempt)
        {
            return Err(QueryObligationDenial::UnknownObligation);
        }
        let source = *state
            .entries
            .keys()
            .find(|revision| revision.as_bytes() == row.source.bytes())
            .ok_or(QueryObligationDenial::UnrosteredProgram(row.source))?;
        let target = *state
            .entries
            .keys()
            .find(|revision| revision.as_bytes() == row.target.bytes())
            .ok_or(QueryObligationDenial::UnrosteredProgram(row.target))?;
        for revision in [&source, &target] {
            if !state
                .entries
                .get(revision)
                .is_some_and(|entry| entry.status != ProgramSupportStatus::Retired)
            {
                return Err(QueryObligationDenial::SupportUnavailable);
            }
        }
        Ok((
            source,
            target,
            WorthQueryProgramSupportCustody {
                state: Arc::clone(&self.state),
                source,
                target: (source != target).then_some(target),
                local: false,
            },
        ))
    }
}

impl QueryObligationHistory {
    pub(in crate::domain_computation::primary_graph) fn is_adoption_original(
        &self,
        effects: &worth_runtime_world::facade::WorldRecoveredUnpublishedEffects,
    ) -> Result<bool, QueryObligationDenial> {
        let actual = effects.attempt_identity();
        let Some(owner) = actual.owner_identity().durable() else {
            return Ok(false);
        };
        let attempt =
            DurableCompositePublicationAttemptIdentity::from_recorded(owner, actual.ordinal());
        let state = self.state();
        let Some(row) = state
            .journal
            .as_ref()
            .and_then(|journal| journal.ledger.adoption.branches.get(&attempt))
        else {
            return Ok(false);
        };
        let expected = effects.expected_head();
        let commit = expected.selected_commit();
        if commit.owner_identity().durable() != Some(row.expected.owner())
            || commit.ordinal() != row.expected.ordinal()
            || expected.branch_identity().name().as_str() != row.branch
            || expected.lifecycle_incarnation().ordinal() != row.lifecycle
            || expected.reference_generation().get().checked_add(1) != Some(row.generation)
        {
            return Err(QueryObligationDenial::ObligationSubjectMismatch);
        }
        Ok(true)
    }
}

impl QueryObligationHistory {
    pub(in crate::domain_computation::primary_graph) fn is_released_adoption_original(
        &self,
        original: &worth_runtime_world::facade::WorldReleasedOriginal,
    ) -> Result<bool, QueryObligationDenial> {
        let actual = original.attempt_identity();
        let Some(owner) = actual.owner_identity().durable() else {
            return Ok(false);
        };
        let attempt =
            DurableCompositePublicationAttemptIdentity::from_recorded(owner, actual.ordinal());
        let state = self.state();
        let Some(row) = state
            .journal
            .as_ref()
            .and_then(|journal| journal.ledger.adoption.branches.get(&attempt))
        else {
            return Ok(false);
        };
        if original.product_branch().name().as_str() != row.branch
            || original.product_incarnation().ordinal() != row.lifecycle
        {
            return Err(QueryObligationDenial::ObligationSubjectMismatch);
        }
        Ok(true)
    }
}

impl QueryObligationHistory {
    pub(in crate::domain_computation::primary_graph) fn branch_adoption(
        &self,
        attempt: DurableCompositePublicationAttemptIdentity,
    ) -> Option<QueryBranchAdoptionObligation> {
        let state = self.state();
        let fold = &state.journal.as_ref()?.ledger.adoption;
        (!fold.completed.contains(&attempt))
            .then(|| fold.branches.get(&attempt).cloned())
            .flatten()
    }
}

impl QueryObligationHistory {
    pub(in crate::domain_computation::primary_graph) fn owns_retained_adoption(
        &self,
        revision: &ApplicationProgramRevision,
        custody: &WorthQueryProgramSupportCustody,
    ) -> bool {
        Arc::ptr_eq(&self.state, &custody.state)
            && custody.source == *revision
            && self
                .state()
                .entries
                .get(revision)
                .is_some_and(|entry| entry.status != ProgramSupportStatus::Retired)
    }
}

mod stopped;
