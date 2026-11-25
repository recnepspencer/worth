//! Post-commit `PendingCloseCommitted` branch and live observer for `custody_dst::ordinary`.
use super::super::super::application_home::correction_material::numeric_home::{
    head_ordinal, model, read_rows,
};
use super::{ledger_has_original, ObservedState, Rng, Slot, TraceLog, TypedCustody};
use model::{edit::Edit, program::Action, schema::NumericSchema};
use worth_foundational::facade::ProgramSupportObligationIdentity;
use worth_query_decl::facade::application_program::ApplicationProgramDefinition;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationRequest, WorthQueryOperationCloseDenial, WorthQuerySourceOnlyAdvance,
    },
    primary_graph::WorthQueryApplicationCommitReceipt,
    product::WorthQueryProductBranch,
    WorthQueryAnySourceCustodyStep, WorthQueryApplicationHome, WorthQuerySourceCustodyPending,
    WorthQuerySourceCustodyStep,
};

pub(super) struct Observer<'a, 'e, P: ApplicationProgramDefinition<NumericSchema> + 'static> {
    pub(super) home: &'a WorthQueryApplicationHome<NumericSchema, P>,
    pub(super) branch: WorthQueryProductBranch,
    pub(super) execution: &'a worth_execution::ExecutionRequest<'e, 'e>,
    pub(super) start_ordinal: u64,
    pub(super) target_value: u64,
}

impl<'a, 'e, P: ApplicationProgramDefinition<NumericSchema> + 'static> Observer<'a, 'e, P> {
    pub(super) fn slot(
        &self,
        slot: &Slot<P>,
        tracked_original: &mut Option<ProgramSupportObligationIdentity>,
        _fresh: &WorthQueryApplicationRequest<'_, '_, '_, NumericSchema>,
    ) -> ObservedState {
        if let Some(id) = slot.durable_original_identity() {
            *tracked_original = Some(id);
        }
        let owner_original = slot.has_durable_original();
        let durable_original = if tracked_original.is_some() {
            let in_ledger = ledger_has_original(self.home, self.execution, *tracked_original);
            assert_eq!(
                owner_original, in_ledger,
                "live custody durable_original must match the Home durable original ledger"
            );
            in_ledger
        } else {
            owner_original
        };
        let head = head_ordinal(self.home.runtime(), self.branch);
        let exec_count = u8::try_from(head.saturating_sub(self.start_ordinal)).unwrap();
        let row = read_rows(self.home.runtime(), self.branch)[0].primary;
        ObservedState {
            phase: slot.phase_label(),
            erased: slot.is_erased(),
            durable_mode: self.home.is_directory(),
            durable_original,
            exec_count,
            committed_value: exec_count > 0 && row == self.target_value,
        }
    }

    pub(super) fn terminal(
        &self,
        phase: &'static str,
        erased: bool,
        tracked_original: Option<ProgramSupportObligationIdentity>,
        _fresh: &WorthQueryApplicationRequest<'_, '_, '_, NumericSchema>,
    ) -> ObservedState {
        let durable_original = ledger_has_original(self.home, self.execution, tracked_original);
        let head = head_ordinal(self.home.runtime(), self.branch);
        let exec_count = u8::try_from(head.saturating_sub(self.start_ordinal)).unwrap();
        let row = read_rows(self.home.runtime(), self.branch)[0].primary;
        ObservedState {
            phase,
            erased,
            durable_mode: self.home.is_directory(),
            durable_original,
            exec_count,
            committed_value: exec_count > 0 && row == self.target_value,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn finish_pending_close<P: ApplicationProgramDefinition<NumericSchema> + 'static>(
    seed: u64,
    rng: &mut Rng,
    obs: &Observer<'_, '_, P>,
    slot: Slot<P>,
    tracked_original: &mut Option<ProgramSupportObligationIdentity>,
    fresh: &WorthQueryApplicationRequest<'_, '_, '_, NumericSchema>,
    cancelled: &WorthQueryApplicationRequest<'_, '_, '_, NumericSchema>,
    log: &mut TraceLog,
    expected_primary: &mut u64,
) -> WorthQueryApplicationCommitReceipt {
    let typed = match slot {
        Slot::Typed(c) => c,
        Slot::Erased(e) => {
            let c = e
                .try_into_typed::<Edit, Action>()
                .unwrap_or_else(|_| panic!("seed {seed} downcast before completed"));
            let s = Slot::Typed(c);
            log.record(
                "Downcast",
                "\"matched\":true",
                obs.slot(&s, tracked_original, fresh),
            );
            let Slot::Typed(c) = s else { unreachable!() };
            c
        }
    };
    let TypedCustody::Retained(op) = typed else {
        unreachable!();
    };
    let WorthQuerySourceOnlyAdvance::Completed(completed) = op.advance(fresh) else {
        panic!("seed {seed} advance must reach Completed");
    };
    let pending_slot = Slot::Typed(TypedCustody::Completed(completed));
    log.record(
        "ResumeCommitPendingClose",
        "",
        obs.slot(&pending_slot, tracked_original, fresh),
    );
    *expected_primary = obs.target_value;
    let Slot::Typed(pending_close) = pending_slot else {
        unreachable!()
    };

    let mut close_slot = if rng.coin() {
        let s = Slot::Erased(pending_close.erase());
        log.record("Erase", "", obs.slot(&s, tracked_original, fresh));
        s
    } else {
        Slot::Typed(pending_close)
    };
    close_slot = match close_slot {
        Slot::Typed(c) => {
            let (c, denial) = c
                .close_unexecuted(fresh)
                .expect_err("PendingCloseCommitted must refuse WithdrawUnexecuted");
            assert!(matches!(
                denial,
                WorthQueryOperationCloseDenial::NativeObligationRetained
            ));
            let s = Slot::Typed(c);
            log.record(
                "WithdrawUnexecuted",
                "\"succeeds\":false",
                obs.slot(&s, tracked_original, fresh),
            );
            s
        }
        Slot::Erased(e) => {
            let (e, denial) = e
                .close_unexecuted(fresh)
                .expect_err("erased PendingCloseCommitted must refuse WithdrawUnexecuted");
            assert!(matches!(
                denial,
                WorthQueryOperationCloseDenial::NativeObligationRetained
            ));
            let s = Slot::Erased(e);
            log.record(
                "WithdrawUnexecuted",
                "\"succeeds\":false",
                obs.slot(&s, tracked_original, fresh),
            );
            s
        }
    };
    close_slot = match close_slot {
        Slot::Typed(c) => match c.resume(obs.home.runtime(), cancelled) {
            WorthQuerySourceCustodyStep::Pending { custody, cause } => {
                assert!(matches!(
                    cause,
                    WorthQuerySourceCustodyPending::Close(
                        WorthQueryOperationCloseDenial::Execution(_)
                    )
                ));
                let s = Slot::Typed(custody);
                log.record(
                    "ClosePending",
                    "\"succeeds\":false",
                    obs.slot(&s, tracked_original, fresh),
                );
                s
            }
            _ => panic!("seed {seed} cancelled close must stay pending"),
        },
        Slot::Erased(e) => match e.resume(obs.home.runtime(), cancelled) {
            WorthQueryAnySourceCustodyStep::Pending { custody, cause } => {
                assert!(matches!(
                    cause,
                    WorthQuerySourceCustodyPending::Close(
                        WorthQueryOperationCloseDenial::Execution(_)
                    )
                ));
                let s = Slot::Erased(custody);
                log.record(
                    "ClosePending",
                    "\"succeeds\":false",
                    obs.slot(&s, tracked_original, fresh),
                );
                s
            }
            _ => panic!("seed {seed} cancelled erased close must stay pending"),
        },
    };
    let was_erased = close_slot.is_erased();
    let receipt = match close_slot {
        Slot::Typed(c) => match c.resume(obs.home.runtime(), fresh) {
            WorthQuerySourceCustodyStep::Committed { evidence, .. } => {
                evidence.live_receipt().cloned().unwrap()
            }
            _ => panic!("seed {seed} fresh close must commit"),
        },
        Slot::Erased(e) => match e.resume(obs.home.runtime(), fresh) {
            WorthQueryAnySourceCustodyStep::Committed { evidence, .. } => {
                evidence.live_receipt().cloned().unwrap()
            }
            _ => panic!("seed {seed} fresh erased close must commit"),
        },
    };
    log.record(
        "ClosePending",
        "\"succeeds\":true",
        obs.terminal("TerminalCommitted", was_erased, *tracked_original, fresh),
    );
    assert_eq!(
        read_rows(obs.home.runtime(), obs.branch)[0].primary,
        *expected_primary
    );
    receipt
}
