//! Historical material follows the original created role after live retirement.
use super::{
    authentication::*, host, source_output_correction, source_output_handlers,
    source_output_model::*,
};
use worth_query_host::facade::{
    application_entry::*, primary_graph::*, runtime::ExecutionAllocationPolicy,
};

pub fn run() {
    historical_created_output();
    filtered_disclosure();
}

fn issue(application: &host::Host) -> (WorthQueryApplicationCommitReceipt, Issued) {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let key = "issue-output".to_owned();
    let outcome = application
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(Issue {
            key: "numeric".to_owned(),
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, result } = outcome else {
        panic!("ordinary issue must commit: {outcome:?}");
    };
    (receipt, result)
}

fn read(application: &host::Host, key: &str) -> Vec<super::query::Row> {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    application
        .runtime()
        .request(&principal, &scope)
        .query(super::query::Read {
            key: key.to_owned(),
        })
        .execute()
        .unwrap()
        .rows()
        .to_vec()
}

fn historical_created_output() {
    let application = host::install();
    let (source, issued) = issue(&application);
    assert_ne!(issued.key, issued.decoy);
    assert_eq!(read(&application, &issued.key)[0].primary, 17);
    assert_eq!(read(&application, &issued.decoy)[0].primary, 777);
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let key = "change-issued-value".to_owned();
    let edit = request
        .advanced()
        .mutate(super::other_edit::OtherEdit {
            key: issued.key.clone(),
            value: super::edit::Value::Primary(29),
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(&application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        edit,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(read(&application, &issued.key)[0].primary, 29);
    let key = "retire-issued-output".to_owned();
    let retired = request
        .advanced()
        .mutate(Retire {
            key: "numeric".to_owned(),
            output_key: issued.key.clone(),
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(&application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            retired,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "ordinary retirement: {retired:?}"
    );
    match request
        .query(super::query::Read {
            key: issued.key.clone(),
        })
        .execute()
    {
        Err(WorthQueryApplicationRequestQueryDenial::ScopeResolution(cause)) => assert_eq!(
            cause.kind(),
            WorthQueryEntityResolutionDenialKind::Refused(
                WorthQueryEntityResolutionRefusalKind::UnknownEntity
            )
        ),
        Err(other) => panic!("retired output read failed for another reason: {other:?}"),
        Ok(_) => panic!("the live output must be absent after ordinary retirement"),
    }
    let limits = super::required_output_courts::limits();
    let selected = request.corrections().select(&source, limits).unwrap();
    let contacts = source_output_handlers::record_contacts();
    let derivations = source_output_correction::derivations();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("record-issued-output")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("original created output must remain available");
    };
    let outcome = prepared.publish().unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, result } = outcome else {
        panic!("correction must publish: {outcome:?}");
    };
    let result = result.downcast_ref::<Recorded>().unwrap();
    assert_eq!(result.original_key, issued.key);
    assert_eq!(result.payload, 17);
    assert_eq!(
        host::read(&application)[0],
        super::query::Row {
            primary: 17,
            secondary: 8
        }
    );
    assert_eq!(read(&application, &issued.decoy)[0].primary, 777);
    assert_eq!(
        receipt
            .aftermath_causality()
            .unwrap()
            .recorded_source()
            .relational(),
        source.commit_reference()
    );
    let head = super::required_output_courts::head(&application);
    let retry = request
        .corrections()
        .reverse(&selected)
        .idempotency("record-issued-output")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Settled(
        WorthQueryApplicationMutationOutcome::AlreadyCommitted(replayed),
    ) = retry
    else {
        panic!("exact correction must replay");
    };
    assert_eq!(replayed.commit_reference(), receipt.commit_reference());
    assert_eq!(source_output_handlers::record_contacts(), contacts + 1);
    assert_eq!(source_output_correction::derivations(), derivations + 1);
    assert_eq!(super::required_output_courts::head(&application), head);
    println!(
        "historical created output survives live change/retirement and ignores an earlier decoy"
    );
}

fn filtered_disclosure() {
    let application = host::install();
    let (source, _) = issue(&application);
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let limits = super::required_output_courts::limits();
    let selected = request.corrections().select(&source, limits).unwrap();
    host::edit(
        &application,
        super::edit::Value::ReadEnabled(false),
        "hide-output-source",
    );
    assert!(host::read(&application).is_empty());
    let head = super::required_output_courts::head(&application);
    let contacts = source_output_handlers::record_contacts();
    let derivations = source_output_correction::derivations();
    let denied = request
        .corrections()
        .reverse(&selected)
        .idempotency("hidden-output-source")
        .limits(limits)
        .prepare();
    assert!(matches!(
        denied,
        Err(WorthQueryApplicationCorrectionDenial::Derivation(
            WorthQueryCorrectionDerivationDenial::SubjectNotDisclosed
        ))
    ));
    assert_eq!(source_output_handlers::record_contacts(), contacts);
    assert_eq!(source_output_correction::derivations(), derivations);
    assert_eq!(super::required_output_courts::head(&application), head);
    println!("filtered source disclosure refuses before historical output derivation");
}
