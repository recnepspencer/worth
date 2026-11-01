//! Current grants and records outside the model survive historical payload selection.
use super::*;
use std::num::NonZeroU64;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestQueryDenial,
    WorthQueryHistoryContextRefreshLimits,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    document_history::{navigation::GRANT, DetailedRetentionHistory},
    record_history::{capability, query},
    retention_entry::{SetRetentionInput, SetRetentionIntent},
};

fn refresh(host: &Runtime, document: &mut Document<'_>) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    document
        .refresh_context(
            &host.request(&principal, &scope),
            WorthQueryHistoryContextRefreshLimits {
                maximum_commits: NonZeroUsize::new(4).unwrap(),
                maximum_records: NonZeroUsize::new(256).unwrap(),
                maximum_evidence_bytes: NonZeroU64::new(8 * 1024 * 1024).unwrap(),
            },
        )
        .unwrap();
}

#[test]
fn read_and_history_metadata_do_not_authorize_navigation_after_its_own_grant_is_revoked() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let root = selection(&host, &document);
    let (mut document, _) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_c400),
        ),
        3,
    );
    let branch = selection(&host, &document)
        .physical_endpoint()
        .product_branch();
    super::super::metadata::grant_named(&host, branch, GRANT, "revoked", 0x9182_c401);
    refresh(&host, &mut document);
    let expected = selection(&host, &document);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    assert_eq!(document.current_step(&request).unwrap(), *expected.step());
    assert_eq!(value(&host, &document), 3);
    let cause = document
        .navigate::<Navigation>()
        .undo_step()
        .expected_selection(expected.clone())
        .idempotency(0x9182_c402)
        .prepare(&request)
        .unwrap_err();
    assert!(
        matches!(cause, Failure::Navigation(Denial::Authorization(_))),
        "{cause:?}"
    );
    assert_eq!(selection(&host, &document), expected);
    // Restoring the real current grant supplies the lawful twin; historical permission alone did not.
    super::super::metadata::grant_named(&host, branch, GRANT, "active", 0x9182_c403);
    refresh(&host, &mut document);
    let done = super::commands::undo(&host, &document, 0x9182_c402);
    assert_eq!(done.selection().step(), root.step());
    assert_eq!(value(&host, &document), SEED_RETENTION);
    close(&host, document, 3);
}

#[test]
fn checkout_preserves_current_outside_records_and_does_not_revive_historical_payload_grants() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let (document, historical) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_c410),
        ),
        3,
    );
    let (mut document, _) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 5, 0x9182_c411),
        ),
        5,
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let branch = selection(&host, &document)
        .physical_endpoint()
        .product_branch();
    super::super::metadata::grant_named(
        &host,
        branch,
        "record-history-payload-grant",
        "revoked",
        0x9182_c412,
    );
    let related_write = host
        .with_application_advancement(&scope, |phase| {
            host.request(&principal, &scope)
                .on_branch(branch)
                .advanced()
                .mutate(SetRetentionIntent {
                    input: SetRetentionInput {
                        identity: RELATED_DOCUMENT_IDENTITY.to_owned(),
                        retention_days: 6,
                    },
                })
                .without_source()
                .idempotency(&0x9182_c413u64)
                .execute_in_program_in_advancement(&phase, &host)
        })
        .unwrap()
        .unwrap();
    assert!(matches!(
        related_write,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    drop(related_write);
    refresh(&host, &mut document);
    let done = checkout(
        &host,
        &document,
        historical.id(),
        selection(&host, &document),
        0x9182_c414,
    );
    let selected = host
        .request(&principal, &scope)
        .on_branch(done.selection().physical_endpoint().product_branch());
    assert_eq!(value(&host, &document), 3);
    let related = selected
        .query(DocumentRetentionRead {
            identity: RELATED_DOCUMENT_IDENTITY.to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(related.rows()[0].retention_days, 6);
    let denied = selected
        .query(query::Input {
            identity: DOCUMENT_IDENTITY.to_owned(),
        })
        .execute_governed(
            capability::PayloadCapability::reference(),
            capability::PayloadRead::reference(),
            capability::Input {
                identity: DOCUMENT_IDENTITY.to_owned(),
            },
        )
        .err()
        .expect("revoked payload grant cannot authorize the selected context");
    assert!(
        matches!(
            denied,
            WorthQueryApplicationRequestQueryDenial::CapabilityAdmission(_)
        ),
        "{denied:?}"
    );
    let old = request
        .on_branch(historical.endpoint().product_branch())
        .query(query::Input {
            identity: DOCUMENT_IDENTITY.to_owned(),
        })
        .execute_governed(
            capability::PayloadCapability::reference(),
            capability::PayloadRead::reference(),
            capability::Input {
                identity: DOCUMENT_IDENTITY.to_owned(),
            },
        )
        .unwrap();
    assert_eq!(old.rows()[0].retention, 3);
    drop(old);
    drop(related);
    drop(denied);
    close(&host, document, 4);
}

#[test]
fn a_history_profile_without_snapshot_opt_in_refuses_even_with_real_navigation_authority() {
    let host = publish_on_first_program();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let owned = head(&host);
    let document = host
        .with_host_advancement(|phase| {
            host.request(&principal, &scope)
                .on_branch(owned.branch())
                .attach_history::<DetailedRetentionHistory>(
                    &phase,
                    owned,
                    DocumentRetentionRead {
                        identity: DOCUMENT_IDENTITY.to_owned(),
                    },
                )
                .unwrap()
        })
        .unwrap();
    let expected = document.selection(&request).unwrap();
    let cause = document
        .navigate::<Navigation>()
        .checkout(expected.step().id())
        .expected_selection(expected.clone())
        .idempotency(0x9182_c420)
        .prepare(&request)
        .unwrap_err();
    assert!(matches!(cause, Failure::Navigation(Denial::RequiresSemanticCorrection(
        worth_query_host::facade::application_entry::WorthQueryHistoryNavigationEligibilityDenial::NotDeclared
    ))), "{cause:?}");
    assert_eq!(document.selection(&request).unwrap(), expected);
    close(&host, document, 1);
}

#[test]
fn an_installed_capability_grant_record_cannot_become_a_snapshot_navigable_model() {
    use worth_query_testkit_document_retention::facade::document_retention_model::document_history::control_roles as controls;
    let host = publish_on_first_program();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let owned = head(&host);
    let document = host
        .with_host_advancement(|phase| {
            host.request(&principal, &scope)
                .on_branch(owned.branch())
                .attach_history::<controls::History>(
                    &phase,
                    owned,
                    controls::read::Input {
                        identity: controls::SUBJECT.to_owned(),
                    },
                )
                .unwrap()
        })
        .unwrap();
    let expected = document.selection(&request).unwrap();
    assert_eq!(
        document.read(&request).unwrap().rows()[0].identity,
        controls::SUBJECT
    );
    let cause = document
        .navigate::<controls::Navigation>()
        .checkout(expected.step().id())
        .expected_selection(expected.clone())
        .idempotency(0x9182_c430)
        .prepare(&request)
        .unwrap_err();
    assert!(matches!(cause, Failure::Navigation(Denial::RequiresSemanticCorrection(
        worth_query_host::facade::application_entry::WorthQueryHistoryNavigationEligibilityDenial::LiveControlDomain
    ))), "{cause:?}");
    assert_eq!(document.selection(&request).unwrap(), expected);
    close(&host, document, 1);
}
