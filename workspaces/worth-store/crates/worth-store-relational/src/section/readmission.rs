use super::RelationalStoreSection;
use worth_execution::ExecutionRequest;
use worth_relational::facade::durability::{
    RecoveryVerificationMode, RelationalDurableRecoverySource, RelationalRecoveryStop,
};
use worth_store::physical_runtime::AnchoredRecordMembership;

#[derive(Debug)]
pub enum RelationalSectionReadmissionDenial {
    MalformedSection,
    Execution(worth_execution::WorkCeilingDenial),
    Payload(worth_execution::ExecutionRequestArrayDenial),
    Journal(crate::RelationalStoreJournalRefusal),
    Transitions(crate::RelationalStoreTransitionJournalRefusal),
    Native(RelationalRecoveryStop),
}

impl RelationalStoreSection {
    /// Readmit the section chosen by the application's single manifest and
    /// the two uncovered tails. No independent checkpoint anchor is consulted.
    pub fn readmit(
        &mut self,
        selected: Option<(&[u8], &AnchoredRecordMembership)>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalSectionReadmissionDenial> {
        let mut admitted = None;
        let scope = execution.in_scope(|_| {
            admitted = Some(self.readmit_in_scope(selected, execution));
        });
        match (scope, admitted) {
            (Ok(()), Some(result)) => result,
            (
                Err(worth_execution::WorkCeilingDenial::Stopped(
                    worth_execution::MapKernelStop::NestedStopped,
                )),
                Some(Err(cause)),
            ) => Err(cause),
            (Err(cause), _) => Err(RelationalSectionReadmissionDenial::Execution(cause)),
            (Ok(()), None) => unreachable!("completed original-request native readmission"),
        }
    }

    fn readmit_in_scope(
        &mut self,
        selected: Option<(&[u8], &AnchoredRecordMembership)>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalSectionReadmissionDenial> {
        use RelationalSectionReadmissionDenial as Denial;
        let checkpoint = selected
            .map(|(bytes, membership)| {
                let checkpoint = super::codec::decode(bytes, execution)?;
                self.cold_source
                    .select_members(membership.version().generation(), membership.members());
                Ok(checkpoint)
            })
            .transpose()?;
        let covered = checkpoint.as_ref().map_or(0, |section| section.covered().0);
        let transitions = checkpoint
            .as_ref()
            .map_or(0, |section| section.covered_transitions().0);
        let tail = crate::recovered_history::journal_tail(&self.serving, covered)
            .map_err(Denial::Journal)?;
        let transitions = crate::recovered_transitions::transition_tail(&self.serving, transitions)
            .map_err(Denial::Transitions)?;
        let source = RelationalDurableRecoverySource::new(
            checkpoint,
            tail,
            transitions,
            self.cold_source.clone(),
        );
        let plan = self
            .runtime
            .durability()
            .recovery_plan_from_source(
                execution,
                source,
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .map_err(Denial::Native)?;
        self.runtime
            .durability_recovery()
            .recover(execution, plan)
            .map_err(Denial::Native)?;
        Ok(())
    }
}
