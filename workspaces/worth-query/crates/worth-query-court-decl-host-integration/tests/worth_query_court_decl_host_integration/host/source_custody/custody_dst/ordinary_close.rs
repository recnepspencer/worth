//! Post-commit `PendingCloseCommitted` branch for `custody_dst::ordinary`.
use super::super::super::application_home::correction_material::numeric_home::{model, read_rows};
use super::ordinary::Observer;
use super::{Rng, Slot, TraceLog, TypedCustody};
use model::{edit::Edit, program::Action, schema::NumericSchema};
use worth_query_decl::facade::application_program::ApplicationProgramDefinition;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationRequest, WorthQueryOperationCloseDenial, WorthQuerySourceOnlyAdvance,
    },
    primary_graph::WorthQueryApplicationCommitReceipt,
    WorthQueryAnySourceCustodyStep, WorthQuerySourceCustodyStep,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn finish_pending_close<P: ApplicationProgramDefinition<NumericSchema> + 'static>(
    seed: u64,
    rng: &mut Rng,
    obs: &Observer<'_, P>,
    slot: Slot<P>,
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
                obs.slot("PendingAdvance", &s),
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
        obs.slot("PendingCloseCommitted", &pending_slot),
    );
    *expected_primary = obs.target_value;
    let Slot::Typed(pending_close) = pending_slot else {
        unreachable!()
    };

    let mut close_slot = if rng.coin() {
        let s = Slot::Erased(pending_close.erase());
        log.record("Erase", "", obs.slot("PendingCloseCommitted", &s));
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
                obs.slot("PendingCloseCommitted", &s),
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
                obs.slot("PendingCloseCommitted", &s),
            );
            s
        }
    };
    close_slot = match close_slot {
        Slot::Typed(c) => match c.resume(obs.home.runtime(), cancelled) {
            WorthQuerySourceCustodyStep::Pending { custody, .. } => {
                let s = Slot::Typed(custody);
                log.record(
                    "ClosePending",
                    "\"succeeds\":false",
                    obs.slot("PendingCloseCommitted", &s),
                );
                s
            }
            _ => panic!("seed {seed} cancelled close must stay pending"),
        },
        Slot::Erased(e) => match e.resume(obs.home.runtime(), cancelled) {
            WorthQueryAnySourceCustodyStep::Pending { custody, .. } => {
                let s = Slot::Erased(custody);
                log.record(
                    "ClosePending",
                    "\"succeeds\":false",
                    obs.slot("PendingCloseCommitted", &s),
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
        obs.terminal("TerminalCommitted", was_erased),
    );
    assert_eq!(
        read_rows(obs.home.runtime(), obs.branch)[0].primary,
        *expected_primary
    );
    receipt
}
