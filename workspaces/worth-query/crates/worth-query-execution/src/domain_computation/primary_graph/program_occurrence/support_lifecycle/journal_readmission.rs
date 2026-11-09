//! Cold Query obligation admission before Home service.
use super::*;
use journal::*;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::CanonicalDigestId;

impl WorthQueryProgramSupportLifecycle {
    pub(in crate::domain_computation::primary_graph) fn readmit_journal(
        &self,
        medium: Arc<dyn QueryObligationsMedium>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<worth_proof::AdmittedHomeQueryObligations, QueryProgramSupportDenial> {
        use QueryProgramSupportDenial as Denial;
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
            let mut ledger = QueryProgramSupportLedger::empty(medium.home());
            while let Some(bytes) = medium.read_frame(
                request,
                ledger
                    .position()
                    .checked_add(1)
                    .ok_or(Denial::IdentityExhausted)?,
            )? {
                request.consult().map_err(Denial::Execution)?;
                ledger.readmit(QueryProgramSupportFrame::readmit(&bytes)?)?;
            }
            // Historical revisions remain DATA through D16 roster cutover.
            // Outstanding recovery rows still require their owning consumers.
            if let Some(row) = ledger.obligations().next() {
                return Err(match row.kind {
                    worth_foundational::facade::ProgramSupportObligationKind::OriginalInterpretation => Denial::OriginalInterpretationReadmissionRequired,
                    worth_foundational::facade::ProgramSupportObligationKind::BranchAdoption => Denial::AdoptionReadmissionRequired,
                });
            }
            let mut state = self.state();
            if state.entries.values().any(|entry| {
                entry.retained_interpretations != 0
                    || entry.mandatory_custody != 0
                    || entry.retirement_claimed
            }) {
                return Err(Denial::SupportUnavailable);
            }
            self.medium
                .set(Arc::clone(&medium))
                .map_err(|_| Denial::SupportUnavailable)?;
            state.journal = Some(SupportJournalState::from_ledger(ledger));
            medium
        };
        driver.finish_pending(medium.as_ref(), request)?;
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
}
