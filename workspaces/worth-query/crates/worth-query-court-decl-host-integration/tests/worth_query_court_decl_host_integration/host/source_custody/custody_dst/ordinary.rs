//! Ordinary command seeded DST driver for Host source custody.
use super::super::super::application_home::correction_material::numeric_home::{model, read_rows};
use super::{cancelled_scope, Oracle, OtherAction, Rng, Slot, TypedCustody};
use model::{
    authentication::{authenticate_operator, request_scope},
    edit::{Edit, Edited, Value},
    other_edit::OtherEdit,
    program::Action,
    schema::NumericSchema,
};
use worth_query_decl::facade::application_program::ApplicationProgramDefinition;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    application_entry::{
        WorthQueryApplicationRequestExt, WorthQueryOperationCloseDenial,
        WorthQuerySourceOnlyAdvance,
    },
    primary_graph::WorthQueryApplicationCommitReceipt,
    product::WorthQueryProductBranch,
    WorthQueryAnySourceCustodyStep, WorthQueryApplicationHome, WorthQuerySourceCustodyStep,
    WorthQuerySourceCustodyWithdrawal,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn run_ordinary_seed<P: ApplicationProgramDefinition<NumericSchema> + 'static>(
    seed: u64,
    rng: &mut Rng,
    home: &WorthQueryApplicationHome<NumericSchema, P>,
    branch: WorthQueryProductBranch,
    clock: &WorthQueryAdmissionClock,
    oracle: &mut Oracle,
    expected_primary: &mut u64,
    committed_key: Option<&str>,
) -> Option<(String, WorthQueryApplicationCommitReceipt)> {
    let scope = request_scope();
    let principal = authenticate_operator(home.primary_graph().installed_schema(), &scope);
    let fresh = home.runtime().request(&principal, &scope).on_branch(branch);
    let cancelled_sc = cancelled_scope(clock);
    let cancelled = home
        .runtime()
        .request(&principal, &cancelled_sc)
        .on_branch(branch);

    let want_drift = committed_key.is_some() && rng.pick(6) == 0;
    let target_value = *expected_primary + 1 + (seed % 7);
    let key = if want_drift {
        committed_key.unwrap().to_owned()
    } else {
        format!("dst-ord-{seed}")
    };
    let described = fresh
        .mutate(Edit {
            key: "numeric".to_owned(),
            value: Value::Primary(target_value),
        })
        .without_source()
        .idempotency_owned(key.clone())
        .unwrap_or_else(|(_, _, denial)| panic!("seed {seed} capture: {denial:?}"));

    let start_erased = rng.coin();
    let mut slot = if start_erased {
        Slot::Erased(TypedCustody::<P>::Described(described).erase())
    } else {
        Slot::Typed(TypedCustody::<P>::Described(described))
    };
    oracle.phase = "Described";
    oracle.erased = start_erased;
    oracle.durable_original = false;
    oracle.exec_count = 0;
    oracle.committed_value = false;
    oracle.record("AcceptOrdinary", &format!("\"erased\":{start_erased}"));

    if !oracle.erased && rng.coin() {
        slot = match slot {
            Slot::Typed(c) => Slot::Erased(c.erase()),
            other => other,
        };
        oracle.erased = true;
        oracle.record("Erase", "");
    }
    if oracle.erased && rng.coin() {
        slot = match slot {
            Slot::Erased(e) => {
                let preserved = e
                    .try_into_typed::<OtherEdit, OtherAction>()
                    .err()
                    .expect("wrong-type downcast must return Err(self)");
                oracle.record("Downcast", "\"matched\":false");
                Slot::Erased(preserved)
            }
            other => other,
        };
    }
    if rng.coin() {
        slot = match slot {
            Slot::Typed(c) => match c.resume(home.runtime(), &cancelled) {
                WorthQuerySourceCustodyStep::Pending { custody, .. } => Slot::Typed(custody),
                _ => panic!("seed {seed} cancelled retention must stay pending"),
            },
            Slot::Erased(e) => match e.resume(home.runtime(), &cancelled) {
                WorthQueryAnySourceCustodyStep::Pending { custody, .. } => Slot::Erased(custody),
                _ => panic!("seed {seed} cancelled erased retention must stay pending"),
            },
        };
        oracle.record("ResumeRetentionRefused", "\"retainedOriginal\":false");
    }

    let mode = if want_drift { 3 } else { rng.pick(5) };
    if mode == 0 {
        match slot {
            Slot::Typed(c) => assert!(matches!(
                c.close_unexecuted(&fresh)
                    .unwrap_or_else(|(_, d)| panic!("seed {seed} described withdraw: {d:?}")),
                WorthQuerySourceCustodyWithdrawal::BeforeConstruction
            )),
            Slot::Erased(e) => assert!(matches!(
                e.close_unexecuted(&fresh).unwrap_or_else(|(_, d)| panic!(
                    "seed {seed} erased described withdraw: {d:?}"
                )),
                WorthQuerySourceCustodyWithdrawal::BeforeConstruction
            )),
        }
        oracle.phase = "TerminalWithdrawn";
        oracle.durable_original = false;
        oracle.record("WithdrawUnexecuted", "\"succeeds\":true");
        assert_eq!(
            read_rows(home.runtime(), branch)[0].primary,
            *expected_primary
        );
        return None;
    }

    if mode == 1 || mode == 2 || mode == 4 {
        let typed = match slot {
            Slot::Typed(c) => c,
            Slot::Erased(e) => {
                let c = e
                    .try_into_typed::<Edit, Action>()
                    .unwrap_or_else(|_| panic!("seed {seed} exact downcast"));
                oracle.erased = false;
                oracle.record("Downcast", "\"matched\":true");
                c
            }
        };
        let TypedCustody::Described(described) = typed else {
            unreachable!();
        };
        let retained = described
            .retain_durable_operation::<P, Action>(&fresh, home.runtime())
            .unwrap_or_else(|(_, denial)| panic!("seed {seed} retain: {denial:?}"));
        let retained = match TypedCustody::Retained(retained).resume(home.runtime(), &cancelled) {
            WorthQuerySourceCustodyStep::Pending { custody, .. } => custody,
            _ => panic!("seed {seed} cancelled advance must stay PendingAdvance"),
        };
        oracle.phase = "PendingAdvance";
        oracle.durable_original = true;
        oracle.record("ResumeRetainAndRefuse", "");
        slot = if rng.coin() {
            oracle.erased = true;
            oracle.record("Erase", "");
            Slot::Erased(retained.erase())
        } else {
            Slot::Typed(retained)
        };
        if rng.coin() {
            slot = match slot {
                Slot::Typed(c) => match c.resume(home.runtime(), &cancelled) {
                    WorthQuerySourceCustodyStep::Pending { custody, .. } => Slot::Typed(custody),
                    _ => panic!("seed {seed} repeat cancelled advance must stay pending"),
                },
                Slot::Erased(e) => match e.resume(home.runtime(), &cancelled) {
                    WorthQueryAnySourceCustodyStep::Pending { custody, .. } => {
                        Slot::Erased(custody)
                    }
                    _ => panic!("seed {seed} repeat cancelled erased advance must stay pending"),
                },
            };
            oracle.record("ResumeAdvanceRefused", "");
        }
    }

    if mode == 1 {
        slot = match slot {
            Slot::Typed(c) => {
                let (c, _) = c
                    .close_unexecuted(&cancelled)
                    .expect_err("cancelled withdrawal must preserve custody");
                oracle.record("WithdrawUnexecuted", "\"succeeds\":false");
                Slot::Typed(c)
            }
            Slot::Erased(e) => {
                let (e, _) = e
                    .close_unexecuted(&cancelled)
                    .expect_err("cancelled erased withdrawal must preserve custody");
                oracle.record("WithdrawUnexecuted", "\"succeeds\":false");
                Slot::Erased(e)
            }
        };
        match slot {
            Slot::Typed(c) => assert!(matches!(
                c.close_unexecuted(&fresh)
                    .unwrap_or_else(|(_, d)| panic!("seed {seed} retained withdraw: {d:?}")),
                WorthQuerySourceCustodyWithdrawal::Closed(_)
            )),
            Slot::Erased(e) => assert!(matches!(
                e.close_unexecuted(&fresh)
                    .unwrap_or_else(|(_, d)| panic!("seed {seed} erased retained withdraw: {d:?}")),
                WorthQuerySourceCustodyWithdrawal::Closed(_)
            )),
        }
        oracle.phase = "TerminalWithdrawn";
        oracle.durable_original = false;
        oracle.record("WithdrawUnexecuted", "\"succeeds\":true");
        assert_eq!(
            read_rows(home.runtime(), branch)[0].primary,
            *expected_primary
        );
        return None;
    }

    if mode == 4 {
        let typed = match slot {
            Slot::Typed(c) => c,
            Slot::Erased(e) => {
                let c = e
                    .try_into_typed::<Edit, Action>()
                    .unwrap_or_else(|_| panic!("seed {seed} downcast before completed"));
                oracle.erased = false;
                oracle.record("Downcast", "\"matched\":true");
                c
            }
        };
        let TypedCustody::Retained(op) = typed else {
            unreachable!();
        };
        let WorthQuerySourceOnlyAdvance::Completed(completed) = op.advance(&fresh) else {
            panic!("seed {seed} advance must reach Completed");
        };
        let pending_close = TypedCustody::Completed(completed);
        oracle.phase = "PendingCloseCommitted";
        oracle.durable_original = true;
        oracle.exec_count = 1;
        oracle.committed_value = true;
        oracle.record("ResumeCommitPendingClose", "");
        *expected_primary = target_value;

        let mut close_slot = if rng.coin() {
            oracle.erased = true;
            oracle.record("Erase", "");
            Slot::Erased(pending_close.erase())
        } else {
            Slot::Typed(pending_close)
        };
        close_slot = match close_slot {
            Slot::Typed(c) => {
                let (c, denial) = c
                    .close_unexecuted(&fresh)
                    .expect_err("PendingCloseCommitted must refuse WithdrawUnexecuted");
                assert!(matches!(
                    denial,
                    WorthQueryOperationCloseDenial::NativeObligationRetained
                ));
                oracle.record("WithdrawUnexecuted", "\"succeeds\":false");
                Slot::Typed(c)
            }
            Slot::Erased(e) => {
                let (e, denial) = e
                    .close_unexecuted(&fresh)
                    .expect_err("erased PendingCloseCommitted must refuse WithdrawUnexecuted");
                assert!(matches!(
                    denial,
                    WorthQueryOperationCloseDenial::NativeObligationRetained
                ));
                oracle.record("WithdrawUnexecuted", "\"succeeds\":false");
                Slot::Erased(e)
            }
        };
        close_slot = match close_slot {
            Slot::Typed(c) => match c.resume(home.runtime(), &cancelled) {
                WorthQuerySourceCustodyStep::Pending { custody, .. } => {
                    oracle.record("ClosePending", "\"succeeds\":false");
                    Slot::Typed(custody)
                }
                _ => panic!("seed {seed} cancelled close must stay pending"),
            },
            Slot::Erased(e) => match e.resume(home.runtime(), &cancelled) {
                WorthQueryAnySourceCustodyStep::Pending { custody, .. } => {
                    oracle.record("ClosePending", "\"succeeds\":false");
                    Slot::Erased(custody)
                }
                _ => panic!("seed {seed} cancelled erased close must stay pending"),
            },
        };
        let receipt = match close_slot {
            Slot::Typed(c) => match c.resume(home.runtime(), &fresh) {
                WorthQuerySourceCustodyStep::Committed { evidence, .. } => {
                    evidence.live_receipt().cloned().unwrap()
                }
                _ => panic!("seed {seed} fresh close must commit"),
            },
            Slot::Erased(e) => match e.resume(home.runtime(), &fresh) {
                WorthQueryAnySourceCustodyStep::Committed { evidence, .. } => {
                    evidence.live_receipt().cloned().unwrap()
                }
                _ => panic!("seed {seed} fresh erased close must commit"),
            },
        };
        oracle.phase = "TerminalCommitted";
        oracle.durable_original = false;
        oracle.record("ClosePending", "\"succeeds\":true");
        assert_eq!(
            read_rows(home.runtime(), branch)[0].primary,
            *expected_primary
        );
        return Some((key, receipt));
    }

    let receipt = match slot {
        Slot::Typed(c) => match c.resume(home.runtime(), &fresh) {
            WorthQuerySourceCustodyStep::Committed { evidence, .. } => {
                assert!(!want_drift);
                Some(evidence.live_receipt().cloned().unwrap())
            }
            WorthQuerySourceCustodyStep::Declined { .. } => {
                assert!(want_drift);
                None
            }
            _ => panic!("seed {seed} fresh resume must settle"),
        },
        Slot::Erased(e) => match e.resume(home.runtime(), &fresh) {
            WorthQueryAnySourceCustodyStep::Committed { evidence, .. } => {
                assert!(!want_drift);
                Some(
                    evidence
                        .into_evidence::<Edited>()
                        .unwrap_or_else(|_| panic!("seed {seed} Edited downcast"))
                        .live_receipt()
                        .cloned()
                        .unwrap(),
                )
            }
            WorthQueryAnySourceCustodyStep::Declined { .. } => {
                assert!(want_drift);
                None
            }
            _ => panic!("seed {seed} fresh erased resume must settle"),
        },
    };
    if let Some(receipt) = receipt {
        oracle.phase = "TerminalCommitted";
        oracle.durable_original = false;
        oracle.exec_count = 1;
        oracle.committed_value = true;
        oracle.record("ResumeCommitClosed", "");
        *expected_primary = target_value;
        assert_eq!(
            read_rows(home.runtime(), branch)[0].primary,
            *expected_primary
        );
        Some((key, receipt))
    } else {
        oracle.phase = "TerminalDenied";
        oracle.durable_original = false;
        oracle.record("ResumeDeclineClosed", "");
        assert_eq!(
            read_rows(home.runtime(), branch)[0].primary,
            *expected_primary
        );
        None
    }
}
