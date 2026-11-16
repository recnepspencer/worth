//! Actual acknowledged material expiry authorizes one terminal support release.
use super::*;
use crate::domain_computation::primary_graph::retained_correction::MaterialPublication;
use journal::*;
use worth_foundational::facade::ProgramSupportObligationKind;
impl QueryObligationHistory {
    pub(in crate::domain_computation::primary_graph) fn dispose_expired_interpretations(
        &self,
        material: &MaterialPublication<'_>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryCorrectionRetentionDenial>
    {
        let medium = self
            .medium
            .get()
            .ok_or(QueryObligationDenial::SupportUnavailable)?;
        material.admit_obligation_scope(medium.as_ref())?;
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        let expired = {
            let state = self.state();
            let ledger = &state
                .journal
                .as_ref()
                .ok_or(QueryObligationDenial::SupportUnavailable)?
                .ledger;
            ledger
                .obligations()
                .filter_map(|row| match row.kind {
                    ProgramSupportObligationKind::OriginalInterpretation
                        if row.source == row.target
                            && material.expired_membership(row.original, row.source) =>
                    {
                        Some(row.identity)
                    }
                    ProgramSupportObligationKind::OriginalInterpretation
                    | ProgramSupportObligationKind::BranchAdoption => None,
                })
                .collect::<Vec<_>>()
        };
        for identity in expired {
            driver.publish(
                medium.as_ref(),
                QueryProgramSupportRecord::Release { identity },
                request,
            )?;
        }
        Ok(())
    }
}
