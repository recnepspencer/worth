//! Prepared reversals and reapplications enter the same Host source custody
//! spine (typed and type-erased) without a second manual state machine.
use super::numeric_home::*;
use super::*;
use std::num::NonZeroUsize;
use worth_foundational::facade::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_decl::facade::application_program::{
    ApplicationActionReference, ApplicationRootComposition,
};
use worth_query_host::facade::{
    application_entry::{WorthQueryApplicationRequestExt, WorthQuerySourceCompletion},
    WorthQueryAnyCorrectionCustodyOutcome, WorthQueryAnySourceCustody,
    WorthQueryAnySourceCustodyStep, WorthQueryCorrectionCustodyOutcome, WorthQuerySourceCustody,
    WorthQuerySourceCustodyStep,
};

type OtherAction = ApplicationActionReference<
    model::schema::NumericSchema,
    ApplicationRootComposition,
    model::program::NumericFeature,
    model::other_edit::OtherBinding,
    model::program::OtherCompletion,
>;

#[test]
fn operation_correction_and_reapplication_run_through_host_source_custody() {
    use model::{
        authentication::{authenticate_operator, request_scope},
        correction::RecordedEdit,
        edit::{Edit, Edited, Value},
        program::Action,
    };
    let path = fixture::location();
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open_program_home(path, execution, true);
    let branch = home.primary_graph().current_world();

    {
        let scope = request_scope();
        let principal = authenticate_operator(home.primary_graph().installed_schema(), &scope);
        let request = home.runtime().request(&principal, &scope).on_branch(branch);

        // 1. Commit an ordinary edit via type-erased Host source custody.
        let described = request
            .mutate(Edit {
                key: "numeric".to_owned(),
                value: Value::Primary(12),
            })
            .without_source()
            .idempotency_owned("custody-edit-1".to_owned())
            .unwrap_or_else(|(_, _, denial)| panic!("capture edit: {denial:?}"));
        let edit_receipt =
            match WorthQuerySourceCustody::<_, _, DirectoryProgram, Action>::Described(described)
                .erase()
                .resume(home.runtime(), &request)
            {
                WorthQueryAnySourceCustodyStep::Committed { evidence, .. } => evidence
                    .into_evidence::<Edited>()
                    .unwrap_or_else(|_| panic!("erased edit returns Edited"))
                    .live_receipt()
                    .cloned()
                    .expect("fresh edit retains live receipt"),
                _ => panic!("ordinary edit must commit"),
            };
        assert_eq!(read_rows(home.runtime(), branch)[0].primary, 12);

        // 2. Run reversal (OtherAction) through type-erased Host correction custody unification.
        let selected = request
            .corrections()
            .select(&edit_receipt, limits())
            .expect("edit receipt selects recorded source");
        let prepared = request
            .corrections()
            .reverse(&selected)
            .idempotency("custody-undo-1")
            .limits(limits())
            .prepare_operation::<RecordedEdit, DirectoryProgram, OtherAction>(home.runtime())
            .expect("reversal prepares operation correction");
        let _undo_receipt = match WorthQueryAnySourceCustody::accept_correction(prepared, &request)
        {
            WorthQueryAnyCorrectionCustodyOutcome::Custody(
                WorthQueryAnySourceCustodyStep::Committed { evidence, .. },
            ) => {
                assert!(matches!(
                    evidence.evidence::<Edited>(),
                    Some(WorthQuerySourceCompletion::Fresh { .. })
                ));
                evidence
                    .live_receipt()
                    .cloned()
                    .expect("fresh reversal retains live receipt")
            }
            _ => panic!("reversal must commit through erased Host custody"),
        };
        assert_eq!(read_rows(home.runtime(), branch)[0].primary, 5);

        // 3. Replaying the same reversal key enters Continued and settles via typed Host custody.
        let replay = request
            .corrections()
            .reverse(&selected)
            .idempotency("custody-undo-1")
            .limits(limits())
            .prepare_operation::<RecordedEdit, DirectoryProgram, OtherAction>(home.runtime())
            .expect("reversal replay prepares Continued");
        match WorthQuerySourceCustody::accept_correction(replay, &request) {
            WorthQueryCorrectionCustodyOutcome::Custody(
                WorthQuerySourceCustodyStep::Committed { evidence, .. },
            ) => {
                assert!(evidence.live_receipt().is_some() || evidence.recorded_receipt().is_some());
            }
            _ => panic!("replayed reversal must settle cleanly"),
        }
        assert_eq!(read_rows(home.runtime(), branch)[0].primary, 5);

        // 4. Reapply the rolled-back edit (Action) through Host correction custody.
        let alternatives = request
            .corrections()
            .alternatives(&selected, limits(), NonZeroUsize::new(8).unwrap())
            .expect("alternatives disclose rolled-back edit");
        let alternative = alternatives
            .entries()
            .first()
            .expect("one reapplicable alternative exists");
        let selected_alt = alternatives
            .select(alternative, limits())
            .expect("alternative selects cleanly");
        let reapply_prepared = request
            .corrections()
            .reapply(&selected_alt)
            .idempotency("custody-redo-1")
            .limits(limits())
            .prepare_operation::<RecordedEdit, DirectoryProgram, Action>(home.runtime())
            .expect("reapplication prepares operation correction");
        match WorthQuerySourceCustody::accept_correction(reapply_prepared, &request).erase() {
            WorthQueryAnyCorrectionCustodyOutcome::Custody(
                WorthQueryAnySourceCustodyStep::Committed { evidence, .. },
            ) => {
                assert!(evidence.live_receipt().is_some());
            }
            _ => panic!("reapplication must commit through Host custody"),
        }
        assert_eq!(read_rows(home.runtime(), branch)[0].primary, 12);
    }

    let closed = home
        .close_with_policy(ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, Some(256 << 20), None),
        ))
        .unwrap_or_else(|refused| panic!("policy close: {refused:?}"));
    assert!(closed.is_closed());
    closed.reclaim();
}
