use worth_proof::NonEmpty;

use super::{indeterminate, one};
use crate::physical_runtime::record_serving::publication::director::RecordPublicationDirector;
use crate::physical_runtime::{
    PhysicalMutationAttempt, PhysicalMutationIndeterminateStage,
    PhysicalMutationPreSealAdmissionDetail as Detail, PhysicalMutationProgressPhase,
    PhysicalMutationProvenNoEffectCause as Cause, PhysicalMutationTerminalFact,
    PhysicalWalGroupAppendFailureCause, PhysicalWalGroupAppendOutcome, PreparedPhysicalMutation,
    RecordAppendDenial, SealedPhysicalDurabilityGroupMembers,
};

impl RecordPublicationDirector {
    pub(super) fn append_managed_wal(
        &self,
        prepared: PreparedPhysicalMutation,
        attempt: &PhysicalMutationAttempt,
    ) -> Result<SealedPhysicalDurabilityGroupMembers, PhysicalMutationTerminalFact> {
        let effect_cutover = attempt.effect_cutover();
        if let Some(cause) = self.pre_seal_denial(attempt) {
            return Err(self.pre_effect_terminal(prepared, attempt, cause));
        }
        attempt.enter(PhysicalMutationProgressPhase::WalAppend);
        let planned = match self.plan_prepared_group_for_wal(NonEmpty::new(prepared, Vec::new())) {
            Ok(planned) => planned,
            Err((members, denial)) => {
                let prepared = one(members);
                let cause = if self.rewrite_source_changed(&prepared) {
                    Cause::SourceChanged
                } else if denial == RecordAppendDenial::RetentionPressure {
                    Cause::RetentionPressure
                } else {
                    Cause::AdmissionDeniedBeforeGroupSeal
                };
                return Err(self.pre_effect_terminal_with_admission_detail(
                    prepared,
                    attempt,
                    cause,
                    Some(Detail::RecordPlanning(denial)),
                ));
            }
        };
        self.mutations.reach_checkpoint(
            crate::physical_runtime::durability::PhysicalMutationCheckpoint::BeforeWalAppend,
        );
        match self.wal.append_prepared_group(planned) {
            PhysicalWalGroupAppendOutcome::Appended(appended) => {
                attempt.commit_settlement();
                drop(effect_cutover);
                self.mutations.reach_checkpoint(
                    crate::physical_runtime::durability::PhysicalMutationCheckpoint::AfterGroupSeal,
                );
                Ok(appended)
            }
            PhysicalWalGroupAppendOutcome::NotAdmitted { members, cause } => {
                let broad = if cause == PhysicalWalGroupAppendFailureCause::PublicationGrowth {
                    Cause::RetentionPressure
                } else {
                    Cause::AdmissionDeniedBeforeGroupSeal
                };
                Err(self.pre_effect_terminal_with_admission_detail(
                    one(members),
                    attempt,
                    broad,
                    Some(Detail::WalAdmission(cause)),
                ))
            }
            PhysicalWalGroupAppendOutcome::AdmissionRejected(rejected) => {
                let detail = Detail::GroupAdmission(rejected.cause());
                Err(self.pre_effect_terminal_with_admission_detail(
                    one(rejected.into_members()),
                    attempt,
                    Cause::AdmissionDeniedBeforeGroupSeal,
                    Some(detail),
                ))
            }
            PhysicalWalGroupAppendOutcome::NotStarted(_) => {
                attempt.commit_settlement();
                drop(effect_cutover);
                Err(indeterminate(
                    attempt,
                    PhysicalMutationIndeterminateStage::WalAppend,
                    0,
                ))
            }
            PhysicalWalGroupAppendOutcome::PartiallyAppended(_)
            | PhysicalWalGroupAppendOutcome::Indeterminate(_) => {
                attempt.commit_settlement();
                drop(effect_cutover);
                Err(indeterminate(
                    attempt,
                    PhysicalMutationIndeterminateStage::WalAppend,
                    1,
                ))
            }
        }
    }
}
