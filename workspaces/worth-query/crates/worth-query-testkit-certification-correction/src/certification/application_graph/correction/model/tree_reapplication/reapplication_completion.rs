//! Completes and closes the actual correction Source, retaining its whole evidence.
use super::*;

pub(super) fn complete(
    host: &host::Host,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    alternative: &WorthQuerySelectedCorrectionAlternative<'_, NumericSchema>,
    key: &str,
) -> (
    WorthQueryClosedOperation,
    WorthQuerySourceCompletion<super::super::edit::Edited>,
) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope).on_branch(branch);
    let prepared = request
        .corrections()
        .reapply(alternative)
        .idempotency(key)
        .limits(limits())
        .prepare_operation::<RecordedEdit, Program, Action>(host)
        .unwrap();
    let advance = match prepared {
        WorthQueryOperationCorrectionPreparation::Prepared(prepared) => prepared.publish().unwrap(),
        WorthQueryOperationCorrectionPreparation::Continued(advance) => advance,
        _ => panic!("reapplication must preserve an ordinary source owner"),
    };
    let completed = match advance {
        WorthQuerySourceOnlyAdvance::Completed(completed) => completed,
        WorthQuerySourceOnlyAdvance::Refused { denial, .. } => {
            panic!("reapplication source refused: {denial:?}")
        }
        WorthQuerySourceOnlyAdvance::Terminal(terminal) => {
            panic!("reapplication terminal: {:?}", terminal.native_outcome())
        }
        WorthQuerySourceOnlyAdvance::Progress(_) => {
            panic!("numeric source has no pending delivery")
        }
    };
    completed
        .close_with_evidence(&request)
        .unwrap_or_else(|(_, denial)| panic!("reapplication normal close: {denial:?}"))
}
