//! Public lifecycle and refusal evidence observes the actual committed world.
use super::{authentication::*, authored_identity_model::*, host, query};
use worth_query_host::facade::{
    application_entry::*, primary_graph::*, runtime::ExecutionAllocationPolicy,
};

pub fn run() {
    create_retire_and_restore();
    wrong_retirement();
    wrong_restoration_key();
}
fn create(
    application: &host::Host,
    name: &str,
    inverse_target: Option<&str>,
) -> WorthQueryApplicationCommitReceipt {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let key = format!("create-{name}");
    let outcome = application
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(Create {
            key: "numeric".to_owned(),
            name: name.to_owned(),
            payload: 41,
            inverse_target: inverse_target.map(str::to_owned),
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("authored create: {outcome:?}")
    };
    receipt
}
fn retire(
    application: &host::Host,
    name: &str,
    wrong_restore_key: bool,
) -> WorthQueryApplicationCommitReceipt {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let key = format!("retire-{name}");
    let outcome = application
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(Retire {
            key: "numeric".to_owned(),
            target: name.to_owned(),
            wrong_restore_key,
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("authored retirement: {outcome:?}")
    };
    receipt
}
fn read(application: &host::Host, key: &str) -> Vec<query::Row> {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    application
        .runtime()
        .request(&principal, &scope)
        .query(query::Read {
            key: key.to_owned(),
        })
        .execute()
        .unwrap()
        .rows()
        .to_vec()
}
fn absent(application: &host::Host, key: &str) {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    match application
        .runtime()
        .request(&principal, &scope)
        .query(query::Read {
            key: key.to_owned(),
        })
        .execute()
    {
        Err(WorthQueryApplicationRequestQueryDenial::ScopeResolution(cause)) => assert_eq!(
            cause.kind(),
            WorthQueryEntityResolutionDenialKind::Refused(
                WorthQueryEntityResolutionRefusalKind::UnknownEntity
            )
        ),
        Err(other) => panic!("unexpected absence cause: {other:?}"),
        Ok(_) => panic!("retired authored entity remained readable"),
    }
}
fn create_retire_and_restore() {
    let application = host::install();
    let created = create(&application, "authored-a", None);
    let original = created
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<Created>()
        .unwrap()
        .entity_id();
    assert_eq!(
        read(&application, "authored-a"),
        vec![query::Row {
            primary: 41,
            secondary: 3
        }]
    );
    super::authored_identity_reference::inspect(&application, original, false);
    let retired = retire(&application, "authored-a", false);
    assert_eq!(
        retired
            .outputs_of::<RetiredOutputs>()
            .unwrap()
            .entity::<Retired>()
            .unwrap()
            .entity_id(),
        original
    );
    absent(&application, "authored-a");
    // Independent parent edits must survive restoration of the deleted authored entity.
    host::edit(
        &application,
        super::edit::Value::Secondary(29),
        "independent-parent-edit",
    );
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let limits = super::required_output_courts::limits();
    let selected = request.corrections().select(&retired, limits).unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("restore-authored-a")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("authored restoration must prepare")
    };
    let outcome = prepared.publish().unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("authored restoration: {outcome:?}")
    };
    let successor = receipt
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<Created>()
        .unwrap()
        .entity_id();
    assert_ne!(
        successor, original,
        "restoration must allocate a new full physical identity"
    );
    assert_eq!(
        read(&application, "authored-a"),
        vec![query::Row {
            primary: 41,
            secondary: 3
        }]
    );
    assert_eq!(host::read(&application)[0].secondary, 29);
    super::authored_identity_reference::inspect(&application, original, true);
    assert_eq!(
        receipt
            .aftermath_causality()
            .unwrap()
            .recorded_source()
            .unwrap()
            .relational(),
        retired.commit_reference()
    );
    let head = super::required_output_courts::head(&application);
    let retry = request
        .corrections()
        .reverse(&selected)
        .idempotency("restore-authored-a")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Settled(
        WorthQueryApplicationMutationOutcome::AlreadyCommitted(replayed),
    ) = retry
    else {
        panic!("authored restoration retry must replay")
    };
    assert_eq!(replayed.commit_reference(), receipt.commit_reference());
    assert_eq!(super::required_output_courts::head(&application), head);

    let second = create(&application, "authored-b", None);
    let selected = request.corrections().select(&second, limits).unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("undo-authored-b")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("create reversal must prepare")
    };
    let outcome = prepared.publish().unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("create reversal: {outcome:?}")
    };
    assert_eq!(
        receipt
            .outputs_of::<RetiredOutputs>()
            .unwrap()
            .entity::<Retired>()
            .unwrap()
            .entity_id(),
        second
            .outputs_of::<CreatedOutputs>()
            .unwrap()
            .entity::<Created>()
            .unwrap()
            .entity_id()
    );
    absent(&application, "authored-b");
    assert_eq!(read(&application, "authored-a")[0].primary, 41);
    println!("authored create retires exact output; deletion restores logical key and payload under new physical identity; retry preserves head");
}
fn refuse_inverse(
    application: &host::Host,
    source: &WorthQueryApplicationCommitReceipt,
    key: &str,
) {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let limits = super::required_output_courts::limits();
    let selected = request.corrections().select(source, limits).unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency(key)
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("adversarial inverse must reach native candidate validation")
    };
    let head = super::required_output_courts::head(application);
    let outcome = prepared.publish().unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("bad authored correspondence must be refused: {outcome:?}")
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::CorrectionIdentityMismatch
    );
    assert_eq!(super::required_output_courts::head(application), head);
}
fn wrong_retirement() {
    let application = host::install();
    create(&application, "unrelated", None);
    let source = create(&application, "original", Some("unrelated"));
    let original = read(&application, "original");
    let unrelated = read(&application, "unrelated");
    refuse_inverse(&application, &source, "wrong-retirement");
    assert_eq!(read(&application, "original"), original);
    assert_eq!(read(&application, "unrelated"), unrelated);
    println!("wrong original identity refuses before commit and preserves both authored entities");
}
fn wrong_restoration_key() {
    let application = host::install();
    create(&application, "original", None);
    create(&application, "unrelated", None);
    let source = retire(&application, "original", true);
    let unrelated = read(&application, "unrelated");
    refuse_inverse(&application, &source, "wrong-restoration-key");
    absent(&application, "original");
    absent(&application, "original-wrong");
    assert_eq!(read(&application, "unrelated"), unrelated);
    println!(
        "wrong restoration key refuses before commit and preserves absence and unrelated entity"
    );
}
