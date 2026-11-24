//! Ordinary command seeded DST driver deriving every trace field from real observations.
use super::super::super::application_home::correction_material::numeric_home::{
    head_ordinal, model, read_rows,
};
use super::{
    cancelled_scope, ordinary_close, ordinary_close::Observer, OtherAction, Rng, Slot, TraceLog,
    TypedCustody,
};
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
    application_entry::WorthQueryApplicationRequestExt,
    primary_graph::WorthQueryApplicationCommitReceipt, product::WorthQueryProductBranch,
    WorthQueryAnySourceCustodyStep, WorthQueryApplicationHome, WorthQuerySourceCustodyStep,
    WorthQuerySourceCustodyWithdrawal,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn run_ordinary_seed<P: ApplicationProgramDefinition<NumericSchema> + 'static>(
    seed: u64,
    rng: &mut Rng,
    home: &WorthQueryApplicationHome<NumericSchema, P>,
    branch: WorthQueryProductBranch,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    clock: &WorthQueryAdmissionClock,
    log: &mut TraceLog,
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
    let obs = Observer {
        home,
        branch,
        execution,
        start_ordinal: head_ordinal(home.runtime(), branch),
        target_value,
    };
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

    let mut tracked_original = None;
    let start_erased = rng.coin();
    let mut slot = if start_erased {
        Slot::Erased(TypedCustody::<P>::Described(described).erase())
    } else {
        Slot::Typed(TypedCustody::<P>::Described(described))
    };
    log.record(
        "AcceptOrdinary",
        &format!(
            "\"erased\":{start_erased},\"durableMode\":{}",
            home.is_directory()
        ),
        obs.slot(&slot, &mut tracked_original, &fresh),
    );

    if !slot.is_erased() && rng.coin() {
        slot = match slot {
            Slot::Typed(c) => Slot::Erased(c.erase()),
            other => other,
        };
        log.record("Erase", "", obs.slot(&slot, &mut tracked_original, &fresh));
    }
    if slot.is_erased() && rng.coin() {
        if let Slot::Erased(e) = slot {
            let preserved = e
                .try_into_typed::<OtherEdit, OtherAction>()
                .err()
                .expect("wrong-type downcast must return Err(self)");
            slot = Slot::Erased(preserved);
            log.record(
                "Downcast",
                "\"matched\":false",
                obs.slot(&slot, &mut tracked_original, &fresh),
            );
        }
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
        let retained_orig = slot.has_durable_original();
        log.record(
            "ResumeRetentionRefused",
            &format!("\"retainedOriginal\":{retained_orig}"),
            obs.slot(&slot, &mut tracked_original, &fresh),
        );
    }

    let mode = if want_drift {
        3
    } else if home.is_directory() {
        4
    } else {
        rng.pick(5)
    };
    if mode == 0 {
        let was_erased = slot.is_erased();
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
        log.record(
            "WithdrawUnexecuted",
            "\"succeeds\":true",
            obs.terminal("TerminalWithdrawn", was_erased, tracked_original, &fresh),
        );
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
                let s = Slot::Typed(c);
                log.record(
                    "Downcast",
                    "\"matched\":true",
                    obs.slot(&s, &mut tracked_original, &fresh),
                );
                let Slot::Typed(c) = s else { unreachable!() };
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
        let typed_slot = Slot::Typed(retained);
        log.record(
            "ResumeRetainAndRefuse",
            "",
            obs.slot(&typed_slot, &mut tracked_original, &fresh),
        );
        let Slot::Typed(retained) = typed_slot else {
            unreachable!()
        };
        slot = if rng.coin() {
            let s = Slot::Erased(retained.erase());
            log.record("Erase", "", obs.slot(&s, &mut tracked_original, &fresh));
            s
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
            log.record(
                "ResumeAdvanceRefused",
                "",
                obs.slot(&slot, &mut tracked_original, &fresh),
            );
        }
    }

    if mode == 1 {
        slot = match slot {
            Slot::Typed(c) => {
                let (c, _) = c
                    .close_unexecuted(&cancelled)
                    .expect_err("cancelled withdrawal must preserve custody");
                let s = Slot::Typed(c);
                log.record(
                    "WithdrawUnexecuted",
                    "\"succeeds\":false",
                    obs.slot(&s, &mut tracked_original, &fresh),
                );
                s
            }
            Slot::Erased(e) => {
                let (e, _) = e
                    .close_unexecuted(&cancelled)
                    .expect_err("cancelled erased withdrawal must preserve custody");
                let s = Slot::Erased(e);
                log.record(
                    "WithdrawUnexecuted",
                    "\"succeeds\":false",
                    obs.slot(&s, &mut tracked_original, &fresh),
                );
                s
            }
        };
        let was_erased = slot.is_erased();
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
        log.record(
            "WithdrawUnexecuted",
            "\"succeeds\":true",
            obs.terminal("TerminalWithdrawn", was_erased, tracked_original, &fresh),
        );
        assert_eq!(
            read_rows(home.runtime(), branch)[0].primary,
            *expected_primary
        );
        return None;
    }

    if mode == 4 {
        let receipt = ordinary_close::finish_pending_close(
            seed,
            rng,
            &obs,
            slot,
            &mut tracked_original,
            &fresh,
            &cancelled,
            log,
            expected_primary,
        );
        return Some((key, receipt));
    }

    let was_erased = slot.is_erased();
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
        log.record(
            "ResumeCommitClosed",
            "",
            obs.terminal("TerminalCommitted", was_erased, tracked_original, &fresh),
        );
        *expected_primary = target_value;
        assert_eq!(
            read_rows(home.runtime(), branch)[0].primary,
            *expected_primary
        );
        Some((key, receipt))
    } else {
        log.record(
            "ResumeDeclineClosed",
            "",
            obs.terminal("TerminalDenied", was_erased, tracked_original, &fresh),
        );
        assert_eq!(
            read_rows(home.runtime(), branch)[0].primary,
            *expected_primary
        );
        None
    }
}
