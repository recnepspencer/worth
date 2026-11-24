//! Correction custody seeded DST driver for Host source custody.
use super::super::super::application_home::correction_material::numeric_home::{
    limits, model, read_rows,
};
use super::{cancelled_scope, ErasedCustody, Oracle, OtherAction, Rng};
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
    WorthQuerySourceCustodyStep,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn run_correction_seed<P: ApplicationProgramDefinition<NumericSchema> + 'static>(
    seed: u64,
    rng: &mut Rng,
    home: &WorthQueryApplicationHome<NumericSchema, P>,
    branch: WorthQueryProductBranch,
    clock: &WorthQueryAdmissionClock,
    oracle: &mut Oracle,
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
    let cancel_close = rng.coin();
    let close_req = if cancel_close { &cancelled } else { &fresh };

    if erased {
        match ErasedCustody::<P>::accept_correction(prepared, close_req) {
            WorthQueryAnyCorrectionCustodyOutcome::Custody(
                WorthQueryAnySourceCustodyStep::Committed { .. },
            ) => {
                assert!(!cancel_close);
                oracle.phase = "TerminalCommitted";
                oracle.erased = true;
                oracle.durable_original = false;
                oracle.exec_count = 1;
                oracle.committed_value = true;
                oracle.record(
                    "AcceptCorrection",
                    "\"erased\":true,\"outcome\":\"CommittedClosed\"",
                );
            }
            WorthQueryAnyCorrectionCustodyOutcome::Custody(
                WorthQueryAnySourceCustodyStep::Pending { custody, .. },
            ) => {
                assert!(cancel_close);
                oracle.phase = "PendingCloseCommitted";
                oracle.erased = true;
                oracle.durable_original = true;
                oracle.exec_count = 1;
                oracle.committed_value = true;
                oracle.record(
                    "AcceptCorrection",
                    "\"erased\":true,\"outcome\":\"PendingClose\"",
                );
                let (custody, denial) = custody
                    .close_unexecuted(&fresh)
                    .expect_err("committed correction must refuse close_unexecuted");
                assert!(matches!(
                    denial,
                    WorthQueryOperationCloseDenial::NativeObligationRetained
                ));
                oracle.record("WithdrawUnexecuted", "\"succeeds\":false");
                assert!(matches!(
                    custody.resume(home.runtime(), &fresh),
                    WorthQueryAnySourceCustodyStep::Committed { .. }
                ));
                oracle.phase = "TerminalCommitted";
                oracle.durable_original = false;
                oracle.record("ClosePending", "\"succeeds\":true");
            }
            _ => panic!("seed {seed} erased correction must commit or pend close"),
        }
    } else {
        match WorthQuerySourceCustody::accept_correction(prepared, close_req) {
            WorthQueryCorrectionCustodyOutcome::Custody(
                WorthQuerySourceCustodyStep::Committed { .. },
            ) => {
                assert!(!cancel_close);
                oracle.phase = "TerminalCommitted";
                oracle.erased = false;
                oracle.durable_original = false;
                oracle.exec_count = 1;
                oracle.committed_value = true;
                oracle.record(
                    "AcceptCorrection",
                    "\"erased\":false,\"outcome\":\"CommittedClosed\"",
                );
            }
            WorthQueryCorrectionCustodyOutcome::Custody(WorthQuerySourceCustodyStep::Pending {
                custody,
                ..
            }) => {
                assert!(cancel_close);
                oracle.phase = "PendingCloseCommitted";
                oracle.erased = false;
                oracle.durable_original = true;
                oracle.exec_count = 1;
                oracle.committed_value = true;
                oracle.record(
                    "AcceptCorrection",
                    "\"erased\":false,\"outcome\":\"PendingClose\"",
                );
                assert!(matches!(
                    custody.resume(home.runtime(), &fresh),
                    WorthQuerySourceCustodyStep::Committed { .. }
                ));
                oracle.phase = "TerminalCommitted";
                oracle.durable_original = false;
                oracle.record("ClosePending", "\"succeeds\":true");
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
