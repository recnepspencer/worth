//! Distinct branch, current-authority, and stale-target controls for retained reapplication.
use super::{
    authentication::*,
    edit::Value,
    host,
    query::{Read, Row},
    tree_reapplication::*,
};
use worth_query_host::facade::{
    application_entry::*, primary_graph::*, product::WorthQueryProductBranch,
};

pub fn run() {
    inherited_consumption_and_foreign_sibling();
    stale_preparation();
    fresh_authority();
    changed_source_support();
    current_source_decision();
}

fn current_source_decision() {
    let host = host::install();
    let source = host::edit(&host, Value::Primary(11), "current-decision-source");
    let reversal = reverse(&host, &source, "current-decision-reverse");
    let alternative = alternative(&host, &source, &reversal, 1);
    host::edit(&host, Value::Ceiling(9), "new-source-ceiling");
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let before = super::required_output_courts::head(&host);
    let preparation = request
        .corrections()
        .reapply(&alternative)
        .idempotency("current-decision-reapply")
        .limits(limits())
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Settled(
        WorthQueryApplicationMutationOutcome::DomainDenied(denial),
    ) = preparation
    else {
        panic!("retained original input11 must reach the source handler's new ceiling9 decision");
    };
    assert_eq!(denial.binding(), "numeric-correction.edit.binding.v1");
    assert!(denial.downcast_ref::<super::edit::MissingValue>().is_some());
    assert_eq!(super::required_output_courts::head(&host), before);
    assert_eq!(
        host::read(&host),
        [Row {
            primary: 5,
            secondary: 7
        }]
    );
    println!("reapplication evaluates original input under the source handler's current decision");
}

fn fork(host: &host::Host) -> WorthQueryProductBranch {
    host.with_host_advancement(|phase| {
        host.runtime()
            .branches()
            .fork(host.runtime().current_world())
            .components(|components| components.fork_relational().reuse_exact_signal_basis())
            .create(&phase)
            .unwrap()
    })
    .unwrap()
}

fn inherited_consumption_and_foreign_sibling() {
    let host = host::install();
    let source = host::edit(&host, Value::Primary(11), "fork-source");
    let reversal = reverse(&host, &source, "fork-reverse");
    let before = fork(&host);
    let main_alternative = alternative(&host, &source, &reversal, 1);
    reapply(&host, &main_alternative, "main-consumption");
    let after = fork(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let after_request = host.runtime().request(&principal, &scope).on_branch(after);
    let after_alternative = alternative_on(&host, after, &source, &reversal, 1);
    assert!(
        matches!(
            after_request
                .corrections()
                .reapply(&after_alternative)
                .idempotency("fork-after-consumption")
                .limits(limits())
                .prepare(),
            Err(WorthQueryApplicationCorrectionDenial::Derivation(
                WorthQueryCorrectionDerivationDenial::NoActiveReversal
            ))
        ),
        "changing branch identity after consumption cannot reset eligibility"
    );

    let before_alternative = alternative_on(&host, before, &source, &reversal, 1);
    let inherited = reapply_on(
        &host,
        before,
        &before_alternative,
        "fork-before-consumption",
    );
    assert_eq!(
        inherited.commit_reference().parents,
        vec![reversal.commit_reference().commit_id]
    );
    assert_ne!(
        inherited.commit_reference().branch_id,
        reversal.commit_reference().branch_id
    );
    assert_eq!(
        host::read_on(&host, before),
        [Row {
            primary: 11,
            secondary: 7
        }]
    );
    let before_request = host.runtime().request(&principal, &scope).on_branch(before);
    let selected = before_request
        .corrections()
        .select(&inherited, limits())
        .unwrap();
    let prepared = before_request
        .corrections()
        .reverse(&selected)
        .idempotency("fork-reverse-reapply")
        .limits(limits())
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("fork reversal prepares")
    };
    let WorthQueryApplicationMutationOutcome::Committed {
        receipt: sibling_reversal,
        ..
    } = prepared.publish().unwrap()
    else {
        panic!("fork reversal commits")
    };
    let sibling = alternative_on(&host, before, &source, &sibling_reversal, 2);
    let main = host.runtime().request(&principal, &scope);
    let retargeted = main
        .corrections()
        .reapply(&sibling)
        .idempotency("foreign-sibling")
        .limits(limits())
        .prepare()
        .err()
        .expect("a sibling's selected carrier must be refused");
    assert!(
        matches!(
            &retargeted,
            WorthQueryApplicationCorrectionDenial::Derivation(
                WorthQueryCorrectionDerivationDenial::TargetIncarnationChanged
            )
        ),
        "a selected alternative retains its original branch incarnation: {retargeted:?}"
    );
    let fork_source = before_request
        .corrections()
        .select(&source, limits())
        .unwrap();
    let fork_page = before_request
        .corrections()
        .alternatives(
            &fork_source,
            limits(),
            std::num::NonZeroUsize::new(2).unwrap(),
        )
        .unwrap();
    let sibling_ref = fork_page
        .entries()
        .iter()
        .find(|entry| entry.performed_commit() == sibling_reversal.commit_reference())
        .unwrap();
    let main_source = main.corrections().select(&source, limits()).unwrap();
    let main_page = main
        .corrections()
        .alternatives(&main_source, limits(), std::num::NonZeroUsize::MIN)
        .unwrap();
    let foreign = main_page
        .select(sibling_ref, limits())
        .err()
        .expect("fresh main selection must refuse the sibling's canonical fact");
    assert!(
        matches!(
            &foreign,
            WorthQueryApplicationCorrectionDenial::Derivation(
                WorthQueryCorrectionDerivationDenial::ForeignAlternative
            )
        ),
        "descriptive sibling metadata is absent from the target's canonical ancestry: {foreign:?}"
    );
    assert_eq!(
        host::read(&host),
        [Row {
            primary: 11,
            secondary: 7
        }]
    );
    assert_eq!(
        host::read_on(&host, before),
        [Row {
            primary: 5,
            secondary: 7
        }]
    );
    println!("fork-before/after consumption and foreign sibling refusal passed");
}

fn stale_preparation() {
    let host = host::install();
    let source = host::edit(&host, Value::Primary(11), "stale-reapply-source");
    let reversal = reverse(&host, &source, "stale-reapply-reverse");
    let alternative = alternative(&host, &source, &reversal, 1);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let prepared = request
        .corrections()
        .reapply(&alternative)
        .idempotency("stale-reapply")
        .limits(limits())
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("reapplication prepares before the concurrent edit")
    };
    host::edit(
        &host,
        Value::Secondary(19),
        "advance-after-reapply-preparation",
    );
    let outcome = prepared.publish().unwrap();
    assert!(
        matches!(&outcome, WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(denial))
        if denial.kind() == WorthQueryApplicationCommitDenialKind::ProductBasisStale),
        "{outcome:?}"
    );
    assert_eq!(
        host::read(&host),
        [Row {
            primary: 5,
            secondary: 19
        }]
    );
    println!("stale reapplication preparation preserves the concurrent edit");
}

fn fresh_authority() {
    use super::principal_status::DisablePrincipal;
    let host = host::install();
    let source = host::edit(&host, Value::Primary(11), "authority-source");
    let reversal = reverse(&host, &source, "authority-reverse");
    let alternative = alternative(&host, &source, &reversal, 1);
    let scope = request_scope();
    let original = authenticate_operator(host.installed_schema(), &scope);
    let current = authenticate_reviewer(host.installed_schema(), &scope);
    let fresh = host.runtime().request(&current, &scope);
    let disabled = fresh
        .advanced()
        .mutate(DisablePrincipal { identity: 1 })
        .without_source()
        .idempotency(&"reapply-disable-original".to_owned())
        .execute_in_program(
            &host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(
        disabled,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert!(
        matches!(
            host.runtime()
                .request(&original, &scope)
                .corrections()
                .reapply(&alternative)
                .idempotency("revoked-reapply")
                .limits(limits())
                .prepare(),
            Err(WorthQueryApplicationCorrectionDenial::Derivation(
                WorthQueryCorrectionDerivationDenial::Principal(_)
            ))
        ),
        "retained original input does not preserve the original actor's revoked authority"
    );
    let prepared = fresh
        .corrections()
        .reapply(&alternative)
        .idempotency("fresh-reapply")
        .limits(limits())
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("fresh actor prepares original source input")
    };
    assert!(matches!(
        prepared.publish().unwrap(),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(
        fresh
            .query(Read {
                key: "numeric".to_owned()
            })
            .execute()
            .unwrap()
            .rows(),
        [Row {
            primary: 11,
            secondary: 7
        }]
    );
    println!("reapplication uses current source authority after original actor revocation");
}

fn changed_source_support() {
    use super::program::ChangedSource;
    let host = host::install();
    let source = host::edit(&host, Value::Primary(11), "support-source");
    let reversal = reverse(&host, &source, "support-reverse");
    let alternative = alternative(&host, &source, &reversal, 1);
    super::adoption_courts::adopt::<ChangedSource>(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let before = super::required_output_courts::head(&host);
    let denial = request
        .corrections()
        .reapply(&alternative)
        .idempotency("changed-source-reapply")
        .limits(limits())
        .prepare()
        .err()
        .expect("changed source action meaning must refuse preparation");
    assert!(
        matches!(&denial, WorthQueryApplicationCorrectionDenial::Derivation(
        WorthQueryCorrectionDerivationDenial::Action(
            worth_query_host::facade::domain::WorthQueryActionResolutionDenial::MeaningMismatch))),
        "reapplication must compare the original source action and its delivery: {denial:?}"
    );
    assert_eq!(super::required_output_courts::head(&host), before);
    assert_eq!(
        host::read(&host),
        [Row {
            primary: 5,
            secondary: 7
        }]
    );
    println!("changed source delivery refuses retained reapplication");
}
