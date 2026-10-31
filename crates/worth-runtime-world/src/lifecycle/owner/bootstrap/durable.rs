//! The existing bootstrap phase retains one original destination across I/O.
use super::{preparation::PreparedRoot, *};
use crate::branch::{
    WorldDurableBootstrapOutcome as Outcome, WorldDurableBootstrapRefusal as Refusal,
};
use crate::durability::{
    ActualWorldPublication, DurableWorldHead, WorldDurableArtifact, WorldDurableMediumBinding,
    WorldDurablePublicationStop, WorldRecord,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::*;
use worth_proof::DurablyCommitted;

pub(crate) struct WorldDurableBootstrapCustody {
    binding: Arc<WorldDurableMediumBinding>,
    subject: WorldDurableArtifact,
    prepared: Mutex<Option<PreparedRoot>>,
    committed: Mutex<Option<DurablyCommitted<ActualWorldPublication>>>,
    driving: AtomicBool,
}
impl WorldDurableBootstrapCustody {
    pub(crate) fn is_driving(&self) -> bool {
        self.driving.load(Ordering::Acquire)
    }
    fn claim(self: &Arc<Self>) -> Option<BootstrapDriver> {
        self.driving
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .ok()
            .map(|_| BootstrapDriver(self.clone()))
    }
}
struct BootstrapDriver(Arc<WorldDurableBootstrapCustody>);
impl Drop for BootstrapDriver {
    fn drop(&mut self) {
        self.0.driving.store(false, Ordering::Release);
    }
}
struct PreparedRootLoan<'a> {
    custody: &'a WorldDurableBootstrapCustody,
    prepared: Option<PreparedRoot>,
}
impl Drop for PreparedRootLoan<'_> {
    fn drop(&mut self) {
        *self
            .custody
            .prepared
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = self.prepared.take();
    }
}
impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(crate) fn begin_durable_bootstrap(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        intent: RuntimeWorldBootstrapIntent,
    ) -> Outcome {
        let Some(binding) = self.state.durable.as_ref() else {
            return Outcome::Refused(Refusal::NotHomeBound);
        };
        if intent
            .cancellation()
            .is_some_and(|token| token.is_cancelled())
        {
            return Outcome::Refused(Refusal::Preparation(
                RuntimeWorldBootstrapNoEffectCause::Cancelled,
            ));
        }
        {
            let mut state = self
                .state
                .bootstrap
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            let close = self.state.close.lock().unwrap_or_else(|e| e.into_inner());
            if !matches!(*state, RuntimeWorldBootstrapState::Unperformed)
                || !self.owner_is_present()
                || close.state() != super::super::super::close::RuntimeWorldCloseState::Open
            {
                return Outcome::Refused(Refusal::Preparation(
                    RuntimeWorldBootstrapNoEffectCause::OwnerUnavailable,
                ));
            }
            *state = RuntimeWorldBootstrapState::InProgress;
        }
        let mut rollback = BootstrapRollback {
            owner: self,
            completed: false,
        };
        let native = match self
            .state
            .relational
            .basis_port()
            .export_recorded_current_component(execution, intent.relational_basis())
        {
            Ok(native) => native,
            Err(cause) => return Outcome::Refused(Refusal::Native(cause)),
        };
        let prepared = match self.prepare_root(intent) {
            Ok(prepared) => prepared,
            Err(cause) => return Outcome::Refused(Refusal::Preparation(cause)),
        };
        let owner = binding.binding().owner();
        let observed = &prepared.observation;
        let branch = match DurableProductBranchIdentity::from_recorded(
            owner,
            observed.branch_identity().name().as_str().to_owned(),
        ) {
            Ok(branch) => branch,
            Err(_) => {
                return Outcome::Refused(Refusal::Preparation(
                    RuntimeWorldBootstrapNoEffectCause::ForeignBasis,
                ))
            }
        };
        let record = WorldRecord::Branch {
            head: DurableWorldHead {
                branch,
                incarnation: DurableProductBranchIncarnation::from_recorded(
                    owner,
                    observed.lifecycle_incarnation().ordinal(),
                ),
                generation: DurableProductReferenceGeneration::from_recorded(
                    observed.reference_generation().get(),
                ),
                commit: DurableCompositeCommitIdentity::from_recorded(
                    owner,
                    observed.selected_commit().ordinal(),
                ),
            },
            bootstrap: DurableWorldBootstrapAttemptIdentity::from_recorded(
                owner,
                prepared.attempt.ordinal(),
            ),
            program_revision: binding.binding().program_revision(),
            relational: native,
        };
        let subject = match WorldDurableArtifact::from_record(record, execution) {
            Ok(subject) => subject,
            Err(cause) => return Outcome::Refused(Refusal::Medium(cause)),
        };
        let pending = Arc::new(WorldDurableBootstrapCustody {
            binding: binding.clone(),
            subject,
            prepared: Mutex::new(Some(prepared)),
            committed: Mutex::new(None),
            driving: AtomicBool::new(false),
        });
        *self
            .state
            .bootstrap
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = RuntimeWorldBootstrapState::Pending(pending);
        rollback.completed = true;
        self.advance_durable_bootstrap(execution)
    }
    pub(crate) fn advance_durable_bootstrap(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Outcome {
        let driver = {
            let state = self
                .state
                .bootstrap
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            let close = self.state.close.lock().unwrap_or_else(|e| e.into_inner());
            if !self.owner_is_present()
                || close.state() != super::super::super::close::RuntimeWorldCloseState::Open
            {
                return Outcome::RecoveryRequired(Refusal::Preparation(
                    RuntimeWorldBootstrapNoEffectCause::OwnerUnavailable,
                ));
            }
            let RuntimeWorldBootstrapState::Pending(pending) = &*state else {
                return Outcome::Refused(Refusal::NoPendingBootstrap);
            };
            let Some(driver) = pending.claim() else {
                return Outcome::Pending;
            };
            driver
        };
        let pending = &driver.0;
        let selected = pending
            .committed
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_some();
        if !selected {
            match pending.binding.publish(&pending.subject, execution) {
                Ok(witness) => {
                    *pending.committed.lock().unwrap_or_else(|e| e.into_inner()) = Some(witness)
                }
                Err(WorldDurablePublicationStop::Pending | WorldDurablePublicationStop::Busy) => {
                    return Outcome::Pending
                }
                Err(WorldDurablePublicationStop::EvidenceRequired) => {
                    return Outcome::EvidenceRequired
                }
                Err(WorldDurablePublicationStop::Unresolved) => return Outcome::Indeterminate,
                Err(WorldDurablePublicationStop::Refused(cause)) => {
                    let retired = std::mem::replace(
                        &mut *self
                            .state
                            .bootstrap
                            .lock()
                            .unwrap_or_else(|e| e.into_inner()),
                        RuntimeWorldBootstrapState::Unperformed,
                    );
                    drop(retired);
                    return Outcome::Refused(Refusal::Medium(cause));
                }
            }
        }
        // The actual move-only witness is retained before history finalization
        // or installation. This loan restores the SAME bundle on every unwind.
        let mut loan = PreparedRootLoan {
            custody: pending,
            prepared: pending
                .prepared
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .take(),
        };
        let prepared = loan
            .prepared
            .as_mut()
            .expect("one original bootstrap driver");
        prepared.retain_selected_history();
        match prepared.install() {
            Ok(performed) => {
                let retired = std::mem::replace(
                    &mut *self
                        .state
                        .bootstrap
                        .lock()
                        .unwrap_or_else(|e| e.into_inner()),
                    RuntimeWorldBootstrapState::Performed,
                );
                drop(retired);
                Outcome::Performed(performed)
            }
            Err(cause) => Outcome::RecoveryRequired(Refusal::Preparation(cause)),
        }
    }
}
