use std::sync::Arc;

use worth_proof::NonEmpty;

use super::RecordPublicationDirector;
use crate::physical_runtime::{
    CompletedPhysicalMutationFact, DataSettledPhysicalMutationMembers,
    IndeterminatePhysicalMutation, PhysicalCurrentRootAdvanceOutcome, PhysicalDataDispatchOutcome,
    PhysicalDataSettlementOutcome, PhysicalDurabilityGroupBasis, PhysicalMutationAttempt,
    PhysicalMutationIndeterminateStage, PhysicalMutationProgressPhase,
    PhysicalMutationProvenNoEffectCause, PhysicalMutationTerminalFact,
    PhysicalPreSealCancellationOutcome, PhysicalRootNamespaceDurabilityOutcome,
    PhysicalRootPublicationPreparationOutcome, PhysicalRootReplacementOutcome,
    PhysicalWalGroupAppendOutcome, PhysicalWalGroupBarrierOutcome, PreparedPhysicalMutation,
    RootNamespaceDurablePhysicalMutationMembers, RootPublicationPreparedPhysicalMutationMembers,
    RootReplacedPhysicalMutationMembers, SealedPhysicalDurabilityGroupMembers,
    WalDurablePhysicalMutation,
};

impl RecordPublicationDirector {
    pub(in crate::physical_runtime) fn execute_managed_mutation(
        &self,
        prepared: PreparedPhysicalMutation,
        attempt: &PhysicalMutationAttempt,
    ) -> PhysicalMutationTerminalFact {
        match self.progress_managed_mutation(prepared, attempt) {
            Ok(completed) => PhysicalMutationTerminalFact::Completed(completed),
            Err(terminal) => terminal,
        }
    }

    fn progress_managed_mutation(
        &self,
        prepared: PreparedPhysicalMutation,
        attempt: &PhysicalMutationAttempt,
    ) -> Result<Arc<CompletedPhysicalMutationFact>, PhysicalMutationTerminalFact> {
        let appended = self.append_managed_wal(prepared, attempt)?;
        let (basis, durable) = self.synchronize_managed_wal(appended, attempt)?;
        let settled = self.settle_managed_data(basis, durable, attempt)?;
        let prepared_root = self.prepare_managed_root(settled, attempt)?;
        let replaced_root = self.replace_managed_root(prepared_root, attempt)?;
        let durable_root = self.synchronize_managed_root(replaced_root, attempt)?;
        self.advance_managed_root(durable_root, attempt)
    }

    fn append_managed_wal(
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
            Err((members, _)) => {
                return Err(self.pre_effect_terminal(
                    one(members),
                    attempt,
                    PhysicalMutationProvenNoEffectCause::AdmissionDeniedBeforeGroupSeal,
                ))
            }
        };
        match self.wal.append_prepared_group(planned) {
            PhysicalWalGroupAppendOutcome::Appended(appended) => {
                attempt.commit_settlement();
                drop(effect_cutover);
                self.mutations.reach_checkpoint(
                    crate::physical_runtime::durability::PhysicalMutationCheckpoint::AfterGroupSeal,
                );
                Ok(appended)
            }
            PhysicalWalGroupAppendOutcome::NotAdmitted { members, .. } => Err(self
                .pre_effect_terminal(
                    one(members),
                    attempt,
                    PhysicalMutationProvenNoEffectCause::AdmissionDeniedBeforeGroupSeal,
                )),
            PhysicalWalGroupAppendOutcome::AdmissionRejected(rejected) => Err(self
                .pre_effect_terminal(
                    one(rejected.into_members()),
                    attempt,
                    PhysicalMutationProvenNoEffectCause::AdmissionDeniedBeforeGroupSeal,
                )),
            PhysicalWalGroupAppendOutcome::NotStarted(_)
            | PhysicalWalGroupAppendOutcome::PartiallyAppended(_)
            | PhysicalWalGroupAppendOutcome::Indeterminate(_) => {
                attempt.commit_settlement();
                drop(effect_cutover);
                Err(indeterminate(
                    attempt,
                    PhysicalMutationIndeterminateStage::WalAppend,
                    0,
                ))
            }
        }
    }

    fn pre_seal_denial(
        &self,
        attempt: &PhysicalMutationAttempt,
    ) -> Option<PhysicalMutationProvenNoEffectCause> {
        if attempt.cancellation_requested() {
            return Some(PhysicalMutationProvenNoEffectCause::CancelledBeforeGroupSeal);
        }
        match self.deadline_elapsed(attempt) {
            Ok(true) => Some(PhysicalMutationProvenNoEffectCause::DeadlineElapsedBeforeGroupSeal),
            Err(()) => Some(PhysicalMutationProvenNoEffectCause::WorkerUnavailableBeforeGroupSeal),
            Ok(false) => None,
        }
    }

    fn synchronize_managed_wal(
        &self,
        appended: SealedPhysicalDurabilityGroupMembers,
        attempt: &PhysicalMutationAttempt,
    ) -> Result<
        (PhysicalDurabilityGroupBasis, WalDurablePhysicalMutation),
        PhysicalMutationTerminalFact,
    > {
        attempt.enter(PhysicalMutationProgressPhase::WalDurabilityBarrier);
        let durable = match self.wal_barrier.synchronize_appended_group(appended) {
            PhysicalWalGroupBarrierOutcome::Durable(durable) => durable,
            PhysicalWalGroupBarrierOutcome::BarrierNotStarted { .. }
            | PhysicalWalGroupBarrierOutcome::Indeterminate(_) => {
                return Err(indeterminate(
                    attempt,
                    PhysicalMutationIndeterminateStage::WalDurabilityBarrier,
                    1,
                ))
            }
        };
        let basis = durable.basis();
        let durable = one(durable.into_members());
        self.mutations.reach_checkpoint(
            crate::physical_runtime::durability::PhysicalMutationCheckpoint::AfterWalDurability,
        );
        Ok((basis, durable))
    }

    fn settle_managed_data(
        &self,
        basis: PhysicalDurabilityGroupBasis,
        durable: WalDurablePhysicalMutation,
        attempt: &PhysicalMutationAttempt,
    ) -> Result<DataSettledPhysicalMutationMembers, PhysicalMutationTerminalFact> {
        attempt.enter(PhysicalMutationProgressPhase::DataDispatch);
        let dispatched = match self.dispatch_wal_durable_data(durable) {
            PhysicalDataDispatchOutcome::Dispatched(dispatched) => dispatched,
            PhysicalDataDispatchOutcome::RetryableAfterCleanup(_)
            | PhysicalDataDispatchOutcome::NotStarted { .. }
            | PhysicalDataDispatchOutcome::Indeterminate(_) => {
                return Err(indeterminate(
                    attempt,
                    PhysicalMutationIndeterminateStage::DataDispatch,
                    1,
                ))
            }
        };
        attempt.enter(PhysicalMutationProgressPhase::DataSettlement);
        self.mutations.reach_checkpoint(
            crate::physical_runtime::durability::PhysicalMutationCheckpoint::DuringDataSettlement,
        );
        let settled = match dispatched.settle_exact_effects() {
            PhysicalDataSettlementOutcome::Settled(settled) => settled,
            PhysicalDataSettlementOutcome::InspectionRequired { .. } => {
                return Err(indeterminate(
                    attempt,
                    PhysicalMutationIndeterminateStage::DataSettlement,
                    1,
                ))
            }
        };
        let settled = match crate::physical_runtime::DataSettledPhysicalMutationMembers::admit(
            basis,
            NonEmpty::new(settled, Vec::new()),
        ) {
            Ok(settled) => settled,
            Err(_) => {
                return Err(indeterminate(
                    attempt,
                    PhysicalMutationIndeterminateStage::DataSettlement,
                    1,
                ))
            }
        };
        self.mutations.reach_checkpoint(
            crate::physical_runtime::durability::PhysicalMutationCheckpoint::AfterDataSettlement,
        );
        Ok(settled)
    }

    fn prepare_managed_root(
        &self,
        settled: DataSettledPhysicalMutationMembers,
        attempt: &PhysicalMutationAttempt,
    ) -> Result<RootPublicationPreparedPhysicalMutationMembers, PhysicalMutationTerminalFact> {
        attempt.enter(PhysicalMutationProgressPhase::RootPreparation);
        let prepared_root = match PhysicalRootPublicationPreparationOutcome::from_result(
            self.prepare_settled_root_publication(settled),
        ) {
            PhysicalRootPublicationPreparationOutcome::Prepared(prepared) => prepared,
            PhysicalRootPublicationPreparationOutcome::NotStarted(_)
            | PhysicalRootPublicationPreparationOutcome::InspectionRequired(_) => {
                return Err(indeterminate(
                    attempt,
                    PhysicalMutationIndeterminateStage::RootPreparation,
                    1,
                ))
            }
        };
        self.mutations.reach_checkpoint(
            crate::physical_runtime::durability::PhysicalMutationCheckpoint::DuringRootPublication,
        );
        Ok(prepared_root)
    }

    fn replace_managed_root(
        &self,
        prepared_root: RootPublicationPreparedPhysicalMutationMembers,
        attempt: &PhysicalMutationAttempt,
    ) -> Result<RootReplacedPhysicalMutationMembers, PhysicalMutationTerminalFact> {
        attempt.enter(PhysicalMutationProgressPhase::RootReplacement);
        match self.replace_prepared_root(prepared_root) {
            PhysicalRootReplacementOutcome::Replaced(replaced) => Ok(replaced),
            PhysicalRootReplacementOutcome::NotStarted(_)
            | PhysicalRootReplacementOutcome::InspectionRequired(_) => Err(indeterminate(
                attempt,
                PhysicalMutationIndeterminateStage::RootReplacement,
                1,
            )),
        }
    }

    fn synchronize_managed_root(
        &self,
        replaced: RootReplacedPhysicalMutationMembers,
        attempt: &PhysicalMutationAttempt,
    ) -> Result<RootNamespaceDurablePhysicalMutationMembers, PhysicalMutationTerminalFact> {
        attempt.enter(PhysicalMutationProgressPhase::RootNamespaceDurability);
        match self.synchronize_replaced_root_namespace(replaced) {
            PhysicalRootNamespaceDurabilityOutcome::Durable(durable) => Ok(durable),
            PhysicalRootNamespaceDurabilityOutcome::NotStarted(_)
            | PhysicalRootNamespaceDurabilityOutcome::InspectionRequired(_) => Err(indeterminate(
                attempt,
                PhysicalMutationIndeterminateStage::RootNamespaceDurability,
                2,
            )),
        }
    }

    fn advance_managed_root(
        &self,
        namespace_durable: RootNamespaceDurablePhysicalMutationMembers,
        attempt: &PhysicalMutationAttempt,
    ) -> Result<Arc<CompletedPhysicalMutationFact>, PhysicalMutationTerminalFact> {
        attempt.enter(PhysicalMutationProgressPhase::CurrentRootAdvance);
        let completed = match self.advance_namespace_durable_root(namespace_durable) {
            PhysicalCurrentRootAdvanceOutcome::Advanced(completed) => completed,
            PhysicalCurrentRootAdvanceOutcome::InspectionRequired(_) => {
                return Err(indeterminate(
                    attempt,
                    PhysicalMutationIndeterminateStage::CurrentRootAdvance,
                    3,
                ))
            }
        };
        let member = completed
            .settled_members()
            .first()
            .expect("one-member managed group completes one exact member");
        debug_assert_eq!(member.mutation_identity(), attempt.identity());
        Ok(CompletedPhysicalMutationFact::from_root_member(
            member,
            attempt.fingerprint(),
            completed.current_root().generation(),
        ))
    }

    fn deadline_elapsed(&self, attempt: &PhysicalMutationAttempt) -> Result<bool, ()> {
        self.runtime
            .upgrade()
            .ok_or(())?
            .signal
            .clock_observation()
            .map(|clock| attempt.deadline().signal_deadline().get() <= clock.current_tick())
            .map_err(|_| ())
    }

    fn pre_effect_terminal(
        &self,
        prepared: PreparedPhysicalMutation,
        attempt: &PhysicalMutationAttempt,
        cause: PhysicalMutationProvenNoEffectCause,
    ) -> PhysicalMutationTerminalFact {
        match self.settle_prepared_before_group_seal(prepared, cause) {
            PhysicalPreSealCancellationOutcome::ProvenNoEffect(terminal) => {
                PhysicalMutationTerminalFact::ProvenNoEffect(terminal)
            }
            PhysicalPreSealCancellationOutcome::NotCancelled { .. } => {
                indeterminate(attempt, PhysicalMutationIndeterminateStage::WalAppend, 0)
            }
        }
    }
}

fn one<T>(members: NonEmpty<T>) -> T {
    let mut members = members.into_vec().into_iter();
    let member = members
        .next()
        .expect("a managed physical mutation group is nonempty");
    debug_assert!(members.next().is_none());
    member
}

fn indeterminate(
    attempt: &PhysicalMutationAttempt,
    stage: PhysicalMutationIndeterminateStage,
    completed_effects: usize,
) -> PhysicalMutationTerminalFact {
    PhysicalMutationTerminalFact::Indeterminate(IndeterminatePhysicalMutation::possible_effect(
        attempt.identity(),
        attempt.idempotency_identity(),
        attempt.fingerprint(),
        stage,
        completed_effects,
    ))
}
