//! The only parent observation belongs to the compiled peer path link.
use super::{
    actions::IntroduceParent,
    evidence::audit,
    operator_identity::{authenticate_operator, request_scope},
    peer_audit::Append,
    peer_evidence::journal,
    row_query,
    world::install,
};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt,
        WorthQueryApplicationRequestMutationDenial as RequestDenial,
    },
    primary_graph::{
        MutationHandlerExecutionDenial, WorthQueryApplicationAttemptDenial as Attempt,
        WorthQueryApplicationAttemptDenialKind as Kind,
        WorthQueryApplicationUncommitted as Uncommitted,
    },
    runtime::ExecutionAllocationPolicy as Allocation,
};
#[test]
fn independent_path_link_captures_original_scope_and_denies_absent_or_ambiguous_anchors_atomically()
{
    for row in ["row-a", "row-missing", "row-ambiguous"] {
        let app = install();
        let scope = request_scope();
        let principal = authenticate_operator(app.installed_schema(), &scope);
        let result = app
            .request(&principal, &scope)
            .advanced()
            .mutate(Append {
                row: row.into(),
                malformed_name: false,
            })
            .without_source()
            .idempotency(&1100)
            .execute_in_program(&app, Allocation::SystemAllocation);
        let row_value = app
            .request(&principal, &scope)
            .query(row_query::Read(row.into()))
            .execute()
            .unwrap()
            .rows()[0]
            .0;
        let recorded = audit(&app, "anchor-check");
        if row == "row-a" {
            assert!(
                matches!(result, Ok(Outcome::Committed { .. })),
                "{result:?}"
            );
            assert_eq!(row_value, 99);
            assert_eq!(recorded, Some(0));
            let entries = journal(&app, "parent-a");
            assert_eq!(entries.len(), 1);
            assert_eq!(
                (
                    entries[0].name.as_str(),
                    entries[0].revision,
                    entries[0].actor
                ),
                ("row-a", 7, 1)
            );
            assert!(journal(&app, "parent-c").is_empty());
        } else {
            match result {
                Err(RequestDenial::Handler(MutationHandlerExecutionDenial::Handler(cause))) => {
                    assert_eq!(
                        cause.downcast::<Attempt>().unwrap().kind(),
                        if row == "row-missing" {
                            Kind::MissingAuthoritativeFact
                        } else {
                            Kind::AmbiguousRelation
                        }
                    );
                }
                other => panic!("{row}: {other:?}"),
            }
            assert_eq!(row_value, 7);
            assert_eq!(recorded, None);
            assert!(journal(&app, "parent-a").is_empty());
            assert!(journal(&app, "parent-c").is_empty());
        }
    }
}
#[test]
fn path_anchor_adjacency_race_stales_the_whole_candidate_before_any_audit_or_scope_write() {
    let app = install();
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    let mut request = app
        .request(&principal, &scope)
        .advanced()
        .mutate(Append {
            row: "row-a".into(),
            malformed_name: false,
        })
        .without_source()
        .idempotency(&1110);
    let prepared = match request
        .prepare_in_program(&app, Allocation::SystemAllocation)
        .unwrap()
    {
        Preparation::Prepared(candidate) => candidate,
        _ => panic!("fresh path-link candidate"),
    };
    let winner = app
        .request(&principal, &scope)
        .advanced()
        .mutate(IntroduceParent {
            row: "row-a".into(),
            identity: "concurrent-parent".into(),
            optional_company: Some("company-a".into()),
        })
        .without_source()
        .idempotency(&1111)
        .execute_in_program(&app, Allocation::SystemAllocation)
        .unwrap();
    assert!(matches!(winner, Outcome::Committed { .. }), "{winner:?}");
    assert!(matches!(
        prepared.commit(Allocation::SystemAllocation),
        Outcome::Commit(Uncommitted::Stale(_))
    ));
    assert!(journal(&app, "parent-a").is_empty());
    assert!(journal(&app, "concurrent-parent").is_empty());
    assert_eq!(
        app.request(&principal, &scope)
            .query(row_query::Read("row-a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .0,
        7
    );
    assert_eq!(audit(&app, "anchor-check"), None);
}

#[test]
fn malformed_peer_only_member_denies_before_scope_or_root_creation_effects() {
    let app = install();
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    let result = app
        .request(&principal, &scope)
        .advanced()
        .mutate(Append {
            row: "row-a".into(),
            malformed_name: true,
        })
        .without_source()
        .idempotency(&1120)
        .execute_in_program(&app, Allocation::SystemAllocation)
        .unwrap();
    assert!(
        matches!(result,Outcome::DomainDenied(worth_query_host::facade::primary_graph::WorthQueryOperationBodyDenial::InputMismatch{ref member}) if member=="audit_name"),
        "{result:?}"
    );
    assert_eq!(
        app.request(&principal, &scope)
            .query(row_query::Read("row-a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .0,
        7
    );
    assert!(journal(&app, "parent-a").is_empty());
    assert_eq!(audit(&app, "anchor-check"), None);
}
