//! Cold Query obligation admission before Home service.
use super::*;
use crate::domain_computation::primary_graph::retained_correction::{
    MaterialPublication, WorthQueryCorrectionRetentionDenial,
};
use journal::*;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    CanonicalDigestId, ProgramSupportObligationIdentity, ProgramSupportObligationKind,
};
use worth_runtime_world::facade::WorldProgramActivationHistory;

impl QueryObligationHistory {
    pub(in crate::domain_computation::primary_graph) fn bind_live_receivers(
        self: &Arc<Self>,
        provider: &crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider,
        execution: &crate::domain_computation::WorthQueryExecutionRuntime,
    ) -> Result<(), QueryObligationDenial> {
        provider.bind_original_meaning_support(self)?;
        execution.managed_run_custody.bind(Arc::clone(self))
    }

    pub(in crate::domain_computation::primary_graph) fn readmit_journal(
        &self,
        medium: Arc<dyn QueryObligationsMedium>,
        originals: &WorldProgramActivationHistory,
        material: &MaterialPublication<'_>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<worth_proof::AdmittedHomeQueryObligations, WorthQueryCorrectionRetentionDenial>
    {
        use QueryObligationDenial as Denial;
        material.admit_obligation_scope(medium.as_ref())?;
        let driver = self.lock_publication()?;
        let medium = if let Some(installed) = self.medium.get() {
            installed
                .authority()
                .admit(medium.authority())
                .map_err(|cause| match cause {
                    worth_proof::HomeIncarnationDenial::ForeignHome => Denial::ForeignHome,
                    worth_proof::HomeIncarnationDenial::StaleIncarnation => {
                        Denial::StaleIncarnation
                    }
                })?;
            Arc::clone(installed)
        } else {
            let mut ledger = QueryObligationLedger::empty(medium.home());
            while let Some(bytes) = medium.read_frame(
                request,
                ledger
                    .position()
                    .checked_add(1)
                    .ok_or(Denial::IdentityExhausted)?,
            )? {
                request.consult().map_err(Denial::Execution)?;
                ledger.readmit(QueryObligationFrame::readmit(&bytes)?)?;
            }
            // Joined originals retain their durable counts. Adoption's owning
            // consumer remains a separate admission requirement.
            self.join_original_interpretations(&ledger, originals, material)?;
            let mut state = self.state();
            if state.entries.values().any(|entry| {
                entry.retained_interpretations != 0
                    || entry.mandatory_custody != 0
                    || entry.retirement_claimed
            }) {
                return Err(Denial::SupportUnavailable.into());
            }
            self.medium
                .set(Arc::clone(&medium))
                .map_err(|_| Denial::SupportUnavailable)?;
            state.journal = Some(QueryObligationJournalState::from_ledger(ledger));
            medium
        };
        driver.finish_pending(medium.as_ref(), request)?;
        // The exclusive Home open has admitted the complete protected World
        // prefix. A pre-effect acquire with no World open has no owner effect;
        // terminal original fates no longer carry adoption custody.
        let finished_adoptions = {
            let state = self.state();
            let fold = &state
                .journal
                .as_ref()
                .expect("admitted journal")
                .ledger
                .adoption;
            fold.branches
                .keys()
                .copied()
                .filter(|attempt| !fold.completed.contains(attempt))
                .filter(|attempt| {
                    originals.original_state(*attempt).is_none_or(|original| {
                        original.fate() != worth_foundational::facade::WorldOriginalFate::Open
                    })
                })
                .collect::<Vec<_>>()
        };
        for attempt in finished_adoptions {
            driver.publish(
                medium.as_ref(),
                QueryAdoptionRecord::ReleaseBranch { attempt },
                request,
            )?;
        }
        let released = {
            let state = self.state();
            let ledger = &state.journal.as_ref().expect("admitted journal").ledger;
            Self::join_original_interpretations_with_roster(
                ledger,
                &state.entries,
                originals,
                material,
            )?
        };
        for identity in released {
            driver.publish(
                medium.as_ref(),
                QueryProgramSupportRecord::Release { identity },
                request,
            )?;
        }
        let missing = {
            let state = self.state();
            let ledger = &state.journal.as_ref().expect("decoded journal").ledger;
            state
                .entries
                .keys()
                .map(|revision| CanonicalDigestId::new(*revision.as_bytes()))
                .filter(|revision| ledger.status(revision).is_none())
                .collect::<Vec<_>>()
        };
        for revision in missing {
            driver.publish(
                medium.as_ref(),
                QueryProgramSupportRecord::Install { revision },
                request,
            )?;
        }
        let mut state = self.state();
        let statuses = {
            let ledger = &state
                .journal
                .as_ref()
                .expect("original admitted journal")
                .ledger;
            state
                .entries
                .keys()
                .map(|revision| {
                    (
                        *revision,
                        ledger
                            .status(&CanonicalDigestId::new(*revision.as_bytes()))
                            .expect("installed status"),
                    )
                })
                .collect::<Vec<_>>()
        };
        for (revision, status) in statuses {
            state
                .entries
                .get_mut(&revision)
                .expect("same roster")
                .status = status;
        }
        let ledger = &state
            .journal
            .as_ref()
            .expect("original admitted journal")
            .ledger;
        Ok(medium.authority().admit_query_obligation_prefix(
            ledger.position(),
            ledger.digest(),
            ledger.payload_digest(),
        ))
    }
    /// Finish cold cleanup's acknowledged records before the Home service
    /// witness escapes. This is the same publisher and admitted ledger.
    pub(in crate::domain_computation::primary_graph) fn admit_current_prefix(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<worth_proof::AdmittedHomeQueryObligations, QueryObligationDenial> {
        let medium = self
            .medium
            .get()
            .ok_or(QueryObligationDenial::SupportUnavailable)?;
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        let state = self.state();
        let ledger = &state
            .journal
            .as_ref()
            .ok_or(QueryObligationDenial::SupportUnavailable)?
            .ledger;
        Ok(medium.authority().admit_query_obligation_prefix(
            ledger.position(),
            ledger.digest(),
            ledger.payload_digest(),
        ))
    }
    fn join_original_interpretations(
        &self,
        ledger: &QueryObligationLedger,
        originals: &WorldProgramActivationHistory,
        material: &MaterialPublication<'_>,
    ) -> Result<Vec<ProgramSupportObligationIdentity>, QueryObligationDenial> {
        Self::join_original_interpretations_with_roster(
            ledger,
            &self.state().entries,
            originals,
            material,
        )
    }
    fn join_original_interpretations_with_roster(
        ledger: &QueryObligationLedger,
        entries: &BTreeMap<ApplicationProgramRevision, SupportEntryState>,
        originals: &WorldProgramActivationHistory,
        material: &MaterialPublication<'_>,
    ) -> Result<Vec<ProgramSupportObligationIdentity>, QueryObligationDenial> {
        for row in ledger
            .adoption
            .branches
            .values()
            .filter(|row| !ledger.adoption.completed.contains(&row.attempt))
        {
            for revision in [row.source, row.target] {
                if !entries
                    .keys()
                    .any(|known| known.as_bytes() == revision.bytes())
                {
                    return Err(QueryObligationDenial::UnrosteredProgram(revision));
                }
                if !matches!(
                    ledger.status(&revision),
                    Some(ProgramSupportStatus::Active | ProgramSupportStatus::Retiring)
                ) {
                    return Err(QueryObligationDenial::SupportUnavailable);
                }
            }
        }
        for row in ledger.adoption.stopped.values() {
            for revision in std::iter::once(row.target)
                .chain(row.remaining.iter().map(|(_, _, source)| *source))
            {
                if !entries
                    .keys()
                    .any(|known| known.as_bytes() == revision.bytes())
                {
                    return Err(QueryObligationDenial::UnrosteredProgram(revision));
                }
                if !matches!(
                    ledger.status(&revision),
                    Some(ProgramSupportStatus::Active | ProgramSupportStatus::Retiring)
                ) {
                    return Err(QueryObligationDenial::SupportUnavailable);
                }
            }
        }
        for row in ledger.output.sources.values() {
            let original = originals
                .recorded_entry(row.commit)
                .ok_or(QueryObligationDenial::ObligationSubjectMismatch)?;
            if original.branch().name() != row.branch
                || original.incarnation().ordinal() != row.lifecycle
                || original.generation().get() != row.generation
                || &original.recorded_revision() != row.revision.bytes()
            {
                return Err(QueryObligationDenial::ObligationSubjectMismatch);
            }
        }
        let revisions = ledger
            .output
            .sources
            .values()
            .map(|row| row.revision)
            .chain(
                ledger
                    .output
                    .demands
                    .values()
                    .filter(|row| !ledger.output.completed.contains(&row.acquire_position))
                    .map(|row| row.revision),
            );
        for revision in revisions {
            if !entries
                .keys()
                .any(|known| known.as_bytes() == revision.bytes())
            {
                return Err(QueryObligationDenial::UnrosteredProgram(revision));
            }
            if !matches!(
                ledger.status(&revision),
                Some(ProgramSupportStatus::Active | ProgramSupportStatus::Retiring)
            ) {
                return Err(QueryObligationDenial::SupportUnavailable);
            }
        }
        let mut terminal = ledger
            .released_original_interpretations(originals)?
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
        for row in ledger.obligations() {
            match row.kind {
                ProgramSupportObligationKind::BranchAdoption => {
                    return Err(QueryObligationDenial::AdoptionReadmissionRequired)
                }
                ProgramSupportObligationKind::OriginalInterpretation => {
                    if row.source == row.target
                        && material.expired_membership(row.original, row.source)
                    {
                        terminal.insert(row.identity);
                    }
                    if !terminal.contains(&row.identity)
                        && !entries
                            .keys()
                            .any(|revision| revision.as_bytes() == row.source.bytes())
                    {
                        return Err(QueryObligationDenial::UnrosteredProgram(row.source));
                    }
                }
            }
        }
        Ok(terminal.into_iter().collect())
    }
}
