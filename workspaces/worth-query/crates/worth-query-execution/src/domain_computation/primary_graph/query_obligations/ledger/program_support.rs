//! Program support status and counted original-meaning membership transitions.
use super::*;
impl QueryObligationLedger {
    pub(super) fn apply_support(&mut self, record: QueryProgramSupportRecord) {
        match record {
            QueryProgramSupportRecord::Install { revision } => {
                self.statuses.insert(revision, ProgramSupportStatus::Active);
            }
            QueryProgramSupportRecord::Status {
                revision, after, ..
            } => {
                self.statuses.insert(revision, after);
            }
            QueryProgramSupportRecord::Acquire(row) => {
                self.obligations.insert(row.identity, row);
            }
            QueryProgramSupportRecord::Release { identity } => {
                self.obligations.remove(&identity);
            }
        }
    }
    pub(super) fn validate_support(
        &self,
        record: &QueryProgramSupportRecord,
        position: u64,
    ) -> Result<(), QueryObligationDenial> {
        use QueryObligationDenial as Denial;
        match record {
            QueryProgramSupportRecord::Install { revision } => {
                if self.statuses.contains_key(revision) {
                    return Err(Denial::DuplicateProgram);
                }
            }
            QueryProgramSupportRecord::Status {
                revision,
                before,
                after,
            } => {
                if self.status(revision) != Some(*before) {
                    return Err(Denial::StatusMismatch);
                }
                match (before, after) {
                    (ProgramSupportStatus::Active, ProgramSupportStatus::Retiring)
                    | (ProgramSupportStatus::Retiring, ProgramSupportStatus::Active) => {}
                    (ProgramSupportStatus::Retiring, ProgramSupportStatus::Retired) => {
                        if self.counts(revision) != (0, 0) {
                            return Err(Denial::SupportUnavailable);
                        }
                    }
                    (
                        ProgramSupportStatus::Active,
                        ProgramSupportStatus::Active | ProgramSupportStatus::Retired,
                    )
                    | (ProgramSupportStatus::Retiring, ProgramSupportStatus::Retiring)
                    | (
                        ProgramSupportStatus::Retired,
                        ProgramSupportStatus::Active
                        | ProgramSupportStatus::Retiring
                        | ProgramSupportStatus::Retired,
                    ) => return Err(Denial::IllegalStatusTransition),
                }
            }
            QueryProgramSupportRecord::Acquire(row) => {
                if row.identity.home() != self.home
                    || row.identity.acquire_position().get() != position
                {
                    return Err(Denial::ObligationIdentityMismatch);
                }
                if self.obligations.contains_key(&row.identity) {
                    return Err(Denial::DuplicateObligation);
                }
                if row.original.owner().home() != self.home {
                    return Err(Denial::ForeignHome);
                }
                if let Some(existing) = self
                    .obligations
                    .values()
                    .find(|known| known.kind == row.kind && known.original == row.original)
                {
                    return Err(
                        if existing.source == row.source && existing.target == row.target {
                            Denial::DuplicateObligation
                        } else {
                            Denial::ObligationSubjectMismatch
                        },
                    );
                }
                match row.kind {
                    ProgramSupportObligationKind::OriginalInterpretation => {
                        if row.source != row.target {
                            return Err(Denial::ObligationSubjectMismatch);
                        }
                    }
                    ProgramSupportObligationKind::BranchAdoption => {}
                }
                for revision in [&row.source, &row.target] {
                    match (row.kind, self.status(revision)) {
                        (
                            ProgramSupportObligationKind::OriginalInterpretation
                            | ProgramSupportObligationKind::BranchAdoption,
                            Some(ProgramSupportStatus::Active),
                        )
                        | (
                            ProgramSupportObligationKind::OriginalInterpretation,
                            Some(ProgramSupportStatus::Retiring),
                        ) => {}
                        (
                            ProgramSupportObligationKind::BranchAdoption,
                            Some(ProgramSupportStatus::Retiring),
                        )
                        | (
                            ProgramSupportObligationKind::OriginalInterpretation
                            | ProgramSupportObligationKind::BranchAdoption,
                            Some(ProgramSupportStatus::Retired) | None,
                        ) => return Err(Denial::SupportUnavailable),
                    }
                }
            }
            QueryProgramSupportRecord::Release { identity } => {
                if identity.home() != self.home {
                    return Err(Denial::ForeignHome);
                }
                if !self.obligations.contains_key(identity) {
                    return Err(Denial::UnknownObligation);
                }
            }
        }
        Ok(())
    }
}
