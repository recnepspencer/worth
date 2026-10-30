//! Ordinary action execution validates genuine identity and the installed invariant.
use super::{
    super::{
        application::model::Model,
        authentication::{self, Actor},
        host, retention,
    },
    retention_action::*,
};
use worth_query_host::facade::{application_entry::*, primary_graph::*};

#[test]
fn ordinary_retention_action_observes_real_runtime_incarnation_and_target_membership() {
    let host = host::install();
    let scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
    let model = Model::create(&host, &author, "model-1", 601, 7).unwrap();
    let other = Model::create(&host, &author, "model-2", 602, 5).unwrap();
    let request = host
        .request(&author.principal, author.scope)
        .on_branch(model.branch());
    complete(
        construct(&host, &model, &author, edit(&model, 3), 603),
        &request,
        3,
    );
    assert_eq!(model.read(&author).unwrap().days, 3);
    assert_eq!(other.read(&author).unwrap().days, 5);

    let foreign_host = host::install();
    let foreign_scope = authentication::request_scope();
    let foreign_author = authentication::authenticate(
        foreign_host.installed_schema(),
        Actor::Author,
        &foreign_scope,
    )
    .unwrap();
    let foreign_model = Model::create(&foreign_host, &foreign_author, "model-1", 604, 7).unwrap();
    assert_ne!(model.identity.runtime, foreign_model.identity.runtime);
    let mut foreign_runtime = edit(&model, 4);
    foreign_runtime.model = foreign_model.identity.clone();
    let mut wrong_incarnation = edit(&model, 4);
    wrong_incarnation.model.occurrence = other.identity.occurrence.clone();
    assert_ne!(
        model.identity.occurrence,
        wrong_incarnation.model.occurrence
    );
    let mut nonmember = edit(&model, 4);
    nonmember.target = other.target.clone();
    assert_ne!(model.target.occurrence, nonmember.target.occurrence);
    for (key, input, expected) in [
        (605, foreign_runtime, retention::Denial::ForeignRuntime),
        (606, wrong_incarnation, retention::Denial::ModelOccurrence),
        (607, nonmember, retention::Denial::TargetMembership),
    ] {
        let before = model.describe(&author).unwrap();
        let operation = construct(&host, &model, &author, input, key);
        let WorthQuerySourceOnlyAdvance::Terminal(terminal) = operation.advance(&request) else {
            panic!("bad stored identity must retain a typed domain terminal")
        };
        let Some(WorthQueryApplicationMutationOutcome::DomainDenied(observed)) =
            terminal.native_outcome()
        else {
            panic!(
                "expected domain identity refusal: {:?}",
                terminal.native_outcome()
            )
        };
        assert!(matches!(
            (observed, expected),
            (
                retention::Denial::ForeignRuntime,
                retention::Denial::ForeignRuntime
            ) | (
                retention::Denial::ModelOccurrence,
                retention::Denial::ModelOccurrence
            ) | (
                retention::Denial::TargetMembership,
                retention::Denial::TargetMembership
            )
        ));
        terminal
            .close(&request)
            .unwrap_or_else(|(_, cause)| panic!("close domain refusal: {cause:?}"));
        assert_eq!(model.describe(&author).unwrap(), before);
        assert_eq!(model.read(&author).unwrap().days, 3);
        assert_eq!(other.read(&author).unwrap().days, 5);
    }
    foreign_model.close(&foreign_author).unwrap();
    model.close(&author).unwrap();
    other.close(&author).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
    assert!(foreign_host.branches().pending_cleanup().is_empty());
}

#[test]
fn installed_retention_rule_allows_ten_and_refuses_eleven_without_publishing() {
    let host = host::install();
    let scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
    let model = Model::create(&host, &author, "model-1", 621, 7).unwrap();
    let request = host
        .request(&author.principal, author.scope)
        .on_branch(model.branch());
    complete(
        construct(&host, &model, &author, edit(&model, 10), 622),
        &request,
        10,
    );
    assert_eq!(model.read(&author).unwrap().days, 10);
    let before = model.describe(&author).unwrap();
    let operation = construct(&host, &model, &author, edit(&model, 11), 623);
    let WorthQuerySourceOnlyAdvance::Terminal(terminal) = operation.advance(&request) else {
        panic!("native rule violation must retain terminal outcome and close custody")
    };
    let Some(WorthQueryApplicationMutationOutcome::Commit(
        WorthQueryApplicationUncommitted::Denied(denial),
    )) = terminal.native_outcome()
    else {
        panic!(
            "expected actual commit invariant denial: {:?}",
            terminal.native_outcome()
        )
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::CustomInvariantDenied
    );
    assert!(denial.custom_invariant_violation_identity().is_some());
    assert!(denial.custom_invariant_failure().is_none());
    terminal
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("close rejected candidate: {cause:?}"));
    assert_eq!(model.describe(&author).unwrap(), before);
    assert_eq!(model.read(&author).unwrap().days, 10);
    model.close(&author).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}
