//! Compensation preserves intervening movements through a distinct current handler.
use super::{authentication::*, compensation_movement::*, edit::Value, host, query::Read};
use worth_query_host::facade::{
    application_entry::*, domain::PublishedAftermathPosture, primary_graph::*,
    runtime::ExecutionAllocationPolicy,
};

pub fn run() {
    arithmetic_and_replay();
    current_authority();
}

fn credit(application: &host::Host, amount: u64, key: &str) -> WorthQueryApplicationCommitReceipt {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let key = key.to_owned();
    let outcome = application
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(Credit {
            key: "numeric".to_owned(),
            amount,
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("the ordinary credit must perform: {outcome:?}");
    };
    receipt
}

fn arithmetic_and_replay() {
    let application = host::install();
    host::edit(&application, Value::Primary(100), "initial-balance");
    let source = credit(&application, 30, "first-credit");
    assert_eq!(
        source.published_aftermath_posture(),
        Some(PublishedAftermathPosture::Compensatable)
    );
    credit(&application, 7, "intervening-credit");
    host::edit(
        &application,
        Value::Secondary(19),
        "disjoint-compensation-edit",
    );
    assert_eq!(host::read(&application)[0].primary, 137);
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let limits = super::required_output_courts::limits();
    let selected = request.corrections().select(&source, limits).unwrap();
    let contacts = debit_contacts();
    let preparation = request
        .corrections()
        .reverse(&selected)
        .idempotency("compensate-first-credit")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = preparation else {
        panic!("explicit compensation must prepare across an intervening balance change");
    };
    assert_eq!(debit_contacts(), contacts + 1);
    assert_eq!(
        host::read(&application)[0].primary,
        137,
        "preparation does not publish"
    );
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } =
        prepared.publish().unwrap()
    else {
        panic!("the fresh compensating debit must perform");
    };
    assert_eq!(
        host::read(&application),
        [super::query::Row {
            primary: 100 + 30 + 7 - 30,
            secondary: 19
        }]
    );
    assert_eq!(
        receipt
            .aftermath_causality()
            .unwrap()
            .recorded_source()
            .unwrap()
            .relational(),
        source.commit_reference()
    );
    let before_replay = super::required_output_courts::head(&application);
    let retry = request
        .corrections()
        .reverse(&selected)
        .idempotency("compensate-first-credit")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Settled(
        WorthQueryApplicationMutationOutcome::AlreadyCommitted(replayed),
    ) = retry
    else {
        panic!("exact compensation replay must return the original receipt");
    };
    assert_eq!(replayed.commit_reference(), receipt.commit_reference());
    assert_eq!(
        replayed.committed_product_publication().composite_commit(),
        receipt.committed_product_publication().composite_commit()
    );
    assert_eq!(
        replayed
            .aftermath_causality()
            .unwrap()
            .recorded_source()
            .unwrap()
            .relational(),
        source.commit_reference()
    );
    assert_eq!(
        debit_contacts(),
        contacts + 1,
        "replay does not admit another debit handler"
    );
    assert_eq!(host::read(&application)[0].primary, 107);
    assert_eq!(
        super::required_output_courts::head(&application),
        before_replay,
        "exact compensation replay cannot advance the actual World head"
    );
    println!("compensation preserves an intervening movement and exact causal replay");
}

fn current_authority() {
    let application = host::install();
    host::edit(
        &application,
        Value::Primary(100),
        "authority-initial-balance",
    );
    let source = credit(&application, 30, "authority-first-credit");
    credit(&application, 7, "authority-intervening-credit");
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let limits = super::required_output_courts::limits();
    let selected = request.corrections().select(&source, limits).unwrap();
    let reviewer = authenticate_reviewer(application.installed_schema(), &scope);
    let fresh = application.runtime().request(&reviewer, &scope);
    let key = "revoke-compensating-actor".to_owned();
    let revoked = fresh
        .advanced()
        .mutate(super::principal_status::DisablePrincipal { identity: 1 })
        .without_source()
        .idempotency(&key)
        .execute_in_program(&application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        revoked,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let contacts = debit_contacts();
    let denied = request
        .corrections()
        .reverse(&selected)
        .idempotency("revoked-compensation")
        .limits(limits)
        .prepare();
    assert!(
        matches!(
            denied,
            Err(WorthQueryApplicationCorrectionDenial::Derivation(
                WorthQueryCorrectionDerivationDenial::Principal(_)
            ))
        ),
        "current disabled mapping must refuse compensation"
    );
    assert_eq!(debit_contacts(), contacts);
    assert_eq!(
        fresh
            .query(Read {
                key: "numeric".to_owned()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .primary,
        137
    );
    let selected = fresh.corrections().select(&source, limits).unwrap();
    let lawful = fresh
        .corrections()
        .reverse(&selected)
        .idempotency("fresh-compensating-actor")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(lawful) = lawful else {
        panic!("the currently authorized twin must prepare");
    };
    assert!(matches!(
        lawful.publish().unwrap(),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(debit_contacts(), contacts + 1);
    assert_eq!(
        fresh
            .query(Read {
                key: "numeric".to_owned()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .primary,
        107
    );
    println!("compensation requires current authority and admits the lawful twin");
}
