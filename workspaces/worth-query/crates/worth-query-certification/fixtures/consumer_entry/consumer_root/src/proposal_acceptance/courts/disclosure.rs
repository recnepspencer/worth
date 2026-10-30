//! Real grant transitions separate READ refusal from capability and administrator authority.
use super::super::{
    access::{self, status, ReadKind},
    application::{model::Model, storage::Storage},
    authentication::{self, Actor, Caller},
    failure::Failure,
    host::{self, Host},
    schema::data::*,
};
use worth_query_host::facade::{
    application_entry::*, primary_graph::*, product::WorthQueryProductBranch,
    runtime::ExecutionAllocationPolicy,
};

fn change(
    host: &Host,
    caller: &Caller<'_>,
    branch: WorthQueryProductBranch,
    input: status::SetStatus,
    key: u64,
) -> Result<
    WorthQueryApplicationMutationOutcome<status::Denial, status::Changed>,
    WorthQueryApplicationRequestMutationDenial,
> {
    host.request(&caller.principal, caller.scope)
        .on_branch(branch)
        .advanced()
        .mutate(input)
        .without_source()
        .idempotency(&key)
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
}
fn read_denied(error: Failure) {
    let Failure::Query(WorthQueryApplicationRequestQueryDenial::Admission(cause)) = error else {
        panic!("expected ordinary READ admission refusal: {error:?}")
    };
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::Authorization(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                WorthQueryOperationAuthorizationRefusalKind::PermissionDenied
            )
        )
    );
}

#[test]
fn model_and_proposal_reads_recheck_independent_live_grants_before_protected_comparisons() {
    for kind in [ReadKind::Model, ReadKind::Proposal] {
        let host = host::install();
        let scope = authentication::request_scope();
        let author =
            authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
        let reviewer =
            authentication::authenticate(host.installed_schema(), Actor::Reviewer, &scope).unwrap();
        let model = Model::create(&host, &author, "model-1", 401, 7).unwrap();
        let storage = Storage::create(&model, &author).unwrap();
        let reference = storage
            .record(
                &model,
                &author,
                "proposal-1",
                402,
                vec![AuthoredIntent::Retention(RetentionEdit {
                    model: model.identity.clone(),
                    target: model.target.clone(),
                    days: 3,
                })],
            )
            .unwrap();
        assert_eq!(model.read(&reviewer).unwrap().days, 7);
        assert_eq!(
            storage.load(&reviewer, &reference).unwrap().intents.len(),
            1
        );
        let branch = match kind {
            ReadKind::Model => model.branch(),
            ReadKind::Proposal => storage.branch(),
        };
        let status = |readable| status::SetStatus {
            identity: access::grant_key("model-1", 2, kind),
            active: true,
            readable,
        };
        let denied = change(&host, &reviewer, branch, status(false), 403).unwrap_err();
        let WorthQueryApplicationRequestMutationDenial::Authorization(cause) = denied else {
            panic!("only the actual grantor may change this grant: {denied:?}")
        };
        assert_eq!(
            cause.kind(),
            WorthQueryOperationAuthorizationDenialKind::Refused(
                WorthQueryOperationAuthorizationRefusalKind::PermissionDenied
            )
        );
        // Denied administration cannot counterfeit the later READ refusal.
        assert_eq!(model.read(&reviewer).unwrap().days, 7);
        storage.load(&reviewer, &reference).unwrap();
        assert!(matches!(
            change(&host, &author, branch, status(false), 404).unwrap(),
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
        match kind {
            ReadKind::Model => {
                read_denied(model.read(&reviewer).unwrap_err());
                read_denied(model.describe(&reviewer).unwrap_err());
                storage.load(&reviewer, &reference).unwrap();
            }
            ReadKind::Proposal => {
                read_denied(storage.load(&reviewer, &reference).unwrap_err());
                let mut foreign = reference.clone();
                foreign.model.runtime += 1;
                read_denied(storage.load(&reviewer, &foreign).unwrap_err());
                assert_eq!(model.read(&reviewer).unwrap().days, 7);
            }
        }
        assert!(matches!(
            change(&host, &author, branch, status(true), 405).unwrap(),
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
        assert_eq!(model.read(&reviewer).unwrap().days, 7);
        storage.load(&reviewer, &reference).unwrap();
        if kind == ReadKind::Model {
            let mut inactive = status(true);
            inactive.active = false;
            assert!(matches!(
                change(&host, &author, branch, inactive, 406).unwrap(),
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ));
            // The other genuine domain-disclosure grant remains current.
            assert_eq!(model.read(&reviewer).unwrap().days, 7);
            assert!(matches!(
                change(&host, &author, branch, status(true), 407).unwrap(),
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ));
        }
        storage.close(&author).unwrap();
        model.close(&author).unwrap();
        assert!(host.branches().pending_cleanup().is_empty());
    }
}

#[test]
fn retained_descriptions_keep_the_original_basis_but_require_current_read_authority() {
    use worth_query_host::facade::product::{
        WorthQueryExpectedProductBasisSelectionDenial as Expected,
        WorthQueryProductBranchAdmissionDenial as Admission,
        WorthQueryProductBranchAdmissionRefusalKind as Refusal,
    };
    let host = host::install();
    let scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
    let reviewer =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &scope).unwrap();
    let model = Model::create(&host, &author, "model-1", 501, 7).unwrap();
    let request = host
        .request(&reviewer.principal, &scope)
        .on_branch(model.branch());
    let observation = request.retain_current_read().unwrap();
    let original = model.describe_observed(&reviewer, &observation).unwrap();
    let status = |principal, readable| status::SetStatus {
        identity: access::grant_key("model-1", principal, ReadKind::Model),
        active: true,
        readable,
    };
    // A real publication moves the basis without removing the reviewer's authority.
    assert!(matches!(
        change(&host, &author, model.branch(), status(1, false), 502).unwrap(),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let current = model.describe(&reviewer).unwrap();
    assert_ne!(current, original);
    assert_eq!(
        model.describe_observed(&reviewer, &observation).unwrap(),
        original
    );
    assert!(matches!(
        change(&host, &author, model.branch(), status(2, false), 503).unwrap(),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    read_denied(
        model
            .describe_observed(&reviewer, &observation)
            .unwrap_err(),
    );
    assert!(matches!(
        change(&host, &author, model.branch(), status(2, true), 504).unwrap(),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(
        model.describe_observed(&reviewer, &observation).unwrap(),
        original
    );
    assert_eq!(model.read(&reviewer).unwrap().days, 7);
    let stale = host
        .with_application_advancement(&scope, |phase| {
            host.branches()
                .select_expected(&original, &phase)
                .err()
                .expect("the original basis moved")
        })
        .unwrap();
    assert!(matches!(
        stale,
        Expected::Admission(Admission::Refused(Refusal::ObservationStaleSourceHead))
    ));
    let fresh = model.describe(&reviewer).unwrap();
    host.with_application_advancement(&scope, |phase| {
        host.branches().select_expected(&fresh, &phase).unwrap();
    })
    .unwrap();
    drop(observation);
    model.close(&author).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}
