//! Correction custody seeded DST driver deriving every trace field from real observations.
use super::super::super::application_home::correction_material::numeric_home::{
    head_ordinal, limits, model, read_rows,
};
use super::{
    cancelled_scope, ledger_has_original, phase_label, ErasedCustody, ObservedState, OtherAction,
    Rng, TraceLog,
};
use model::{
    authentication::{authenticate_operator, request_scope},
    correction::RecordedEdit,
    schema::NumericSchema,
};
use worth_query_decl::facade::application_program::ApplicationProgramDefinition;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    application_entry::{WorthQueryApplicationRequestExt, WorthQueryOperationCloseDenial},
    primary_graph::WorthQueryApplicationCommitReceipt,
    product::WorthQueryProductBranch,
    WorthQueryAnyCorrectionCustodyOutcome, WorthQueryAnySourceCustodyStep,
    WorthQueryApplicationHome, WorthQueryCorrectionCustodyOutcome, WorthQuerySourceCustody,
    WorthQuerySourceCustodyPending, WorthQuerySourceCustodyStep,
};

fn observe<P: ApplicationProgramDefinition<NumericSchema> + 'static>(
    home: &WorthQueryApplicationHome<NumericSchema, P>,
    branch: WorthQueryProductBranch,
    start_ordinal: u64,
    prior_primary: u64,
    phase: &'static str,
    erased: bool,
    durable_original: bool,
) -> ObservedState {
    let head = head_ordinal(home.runtime(), branch);
    let exec_count = u8::try_from(head.saturating_sub(start_ordinal)).unwrap();
    let row = read_rows(home.runtime(), branch)[0].primary;
    ObservedState {
        phase,
        erased,
        durable_mode: home.is_directory(),
        durable_original,
        exec_count,
        committed_value: exec_count > 0 && row == prior_primary,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn run_correction_seed<P: ApplicationProgramDefinition<NumericSchema> + 'static>(
    seed: u64,
    rng: &mut Rng,
    home: &WorthQueryApplicationHome<NumericSchema, P>,
    branch: WorthQueryProductBranch,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    clock: &WorthQueryAdmissionClock,
    log: &mut TraceLog,
    expected_primary: &mut u64,
    prior_primary: u64,
    receipt: &WorthQueryApplicationCommitReceipt,
) {
    let scope = request_scope();
    let principal = authenticate_operator(home.primary_graph().installed_schema(), &scope);
    let fresh = home.runtime().request(&principal, &scope).on_branch(branch);
    let cancelled_sc = cancelled_scope(clock);
    let cancelled = home
        .runtime()
        .request(&principal, &cancelled_sc)
        .on_branch(branch);
    let start_ordinal = head_ordinal(home.runtime(), branch);
    let durable_mode = home.is_directory();
    let selected = fresh
        .corrections()
        .select(receipt, limits())
        .expect("select committed edit for correction");
    let corr_key = format!("dst-corr-{seed}");
    let prepared = fresh
        .corrections()
        .reverse(&selected)
        .idempotency(&corr_key)
        .limits(limits())
        .prepare_operation::<RecordedEdit, P, OtherAction>(home.runtime())
        .expect("prepare reversal");

    let erased = rng.coin();
    let cancel_close = rng.coin() || durable_mode;
    let close_req = if cancel_close { &cancelled } else { &fresh };

    if erased {
        match ErasedCustody::<P>::accept_correction(prepared, close_req) {
            WorthQueryAnyCorrectionCustodyOutcome::Custody(
                WorthQueryAnySourceCustodyStep::Committed { .. },
            ) => {
                assert!(!cancel_close);
                let in_ledger = ledger_has_original(home, execution, None);
                log.record(
                    "AcceptCorrection",
                    &format!(
                        "\"erased\":true,\"durableMode\":{durable_mode},\"outcome\":\"CommittedClosed\""
                    ),
                    observe(
                        home,
                        branch,
                        start_ordinal,
                        prior_primary,
                        "TerminalCommitted",
                        true,
                        in_ledger,
                    ),
                );
            }
            WorthQueryAnyCorrectionCustodyOutcome::Custody(
                WorthQueryAnySourceCustodyStep::Pending { custody, cause },
            ) => {
                assert!(cancel_close);
                assert!(matches!(
                    cause,
                    WorthQuerySourceCustodyPending::Close(
                        WorthQueryOperationCloseDenial::Execution(_)
                    )
                ));
                let orig_id = custody.durable_original_identity();
                log.record(
                    "AcceptCorrection",
                    &format!(
                        "\"erased\":true,\"durableMode\":{durable_mode},\"outcome\":\"PendingClose\""
                    ),
                    observe(
                        home,
                        branch,
                        start_ordinal,
                        prior_primary,
                        phase_label(custody.phase()),
                        true,
                        custody.has_durable_original(),
                    ),
                );
                let (custody, denial) = custody
                    .close_unexecuted(&fresh)
                    .expect_err("committed correction must refuse close_unexecuted");
                assert!(matches!(
                    denial,
                    WorthQueryOperationCloseDenial::NativeObligationRetained
                ));
                log.record(
                    "WithdrawUnexecuted",
                    "\"succeeds\":false",
                    observe(
                        home,
                        branch,
                        start_ordinal,
                        prior_primary,
                        phase_label(custody.phase()),
                        true,
                        custody.has_durable_original(),
                    ),
                );
                assert!(matches!(
                    custody.resume(home.runtime(), &fresh),
                    WorthQueryAnySourceCustodyStep::Committed { .. }
                ));
                let in_ledger = ledger_has_original(home, execution, orig_id);
                log.record(
                    "ClosePending",
                    "\"succeeds\":true",
                    observe(
                        home,
                        branch,
                        start_ordinal,
                        prior_primary,
                        "TerminalCommitted",
                        true,
                        in_ledger,
                    ),
                );
            }
            _ => panic!("seed {seed} erased correction must commit or pend close"),
        }
    } else {
        match WorthQuerySourceCustody::accept_correction(prepared, close_req) {
            WorthQueryCorrectionCustodyOutcome::Custody(
                WorthQuerySourceCustodyStep::Committed { .. },
            ) => {
                assert!(!cancel_close);
                let in_ledger = ledger_has_original(home, execution, None);
                log.record(
                    "AcceptCorrection",
                    &format!(
                        "\"erased\":false,\"durableMode\":{durable_mode},\"outcome\":\"CommittedClosed\""
                    ),
                    observe(
                        home,
                        branch,
                        start_ordinal,
                        prior_primary,
                        "TerminalCommitted",
                        false,
                        in_ledger,
                    ),
                );
            }
            WorthQueryCorrectionCustodyOutcome::Custody(WorthQuerySourceCustodyStep::Pending {
                custody,
                cause,
            }) => {
                assert!(cancel_close);
                assert!(matches!(
                    cause,
                    WorthQuerySourceCustodyPending::Close(
                        WorthQueryOperationCloseDenial::Execution(_)
                    )
                ));
                let orig_id = custody.durable_original_identity();
                log.record(
                    "AcceptCorrection",
                    &format!(
                        "\"erased\":false,\"durableMode\":{durable_mode},\"outcome\":\"PendingClose\""
                    ),
                    observe(
                        home,
                        branch,
                        start_ordinal,
                        prior_primary,
                        phase_label(custody.phase()),
                        false,
                        custody.has_durable_original(),
                    ),
                );
                let (custody, denial) = custody
                    .close_unexecuted(&fresh)
                    .expect_err("typed committed correction must refuse close_unexecuted");
                assert!(matches!(
                    denial,
                    WorthQueryOperationCloseDenial::NativeObligationRetained
                ));
                log.record(
                    "WithdrawUnexecuted",
                    "\"succeeds\":false",
                    observe(
                        home,
                        branch,
                        start_ordinal,
                        prior_primary,
                        phase_label(custody.phase()),
                        false,
                        custody.has_durable_original(),
                    ),
                );
                assert!(matches!(
                    custody.resume(home.runtime(), &fresh),
                    WorthQuerySourceCustodyStep::Committed { .. }
                ));
                let in_ledger = ledger_has_original(home, execution, orig_id);
                log.record(
                    "ClosePending",
                    "\"succeeds\":true",
                    observe(
                        home,
                        branch,
                        start_ordinal,
                        prior_primary,
                        "TerminalCommitted",
                        false,
                        in_ledger,
                    ),
                );
            }
            _ => panic!("seed {seed} typed correction must commit or pend close"),
        }
    }
    *expected_primary = prior_primary;
    assert_eq!(
        read_rows(home.runtime(), branch)[0].primary,
        *expected_primary
    );
}
