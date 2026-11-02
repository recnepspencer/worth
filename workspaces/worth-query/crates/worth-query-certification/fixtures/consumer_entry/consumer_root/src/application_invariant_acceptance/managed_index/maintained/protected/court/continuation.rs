//! Fresh governance protects ordinary retained child pages and their metadata.
use super::super::super::{mutation, query as source, schema::Directory};
use super::super::page;
use super::*;
use std::num::NonZeroUsize;
type Page = WorthQueryApplicationContinuationPageResult<
    Catalog,
    page::Query,
    source::Parameters,
    page::Row,
    Directory,
>;
type Token = WorthQueryApplicationQueryContinuation<
    Catalog,
    page::Query,
    source::Parameters,
    page::Row,
    Directory,
>;
fn input() -> page::ReadPage {
    page::ReadPage {
        directory: "catalog".into(),
    }
}
fn width() -> NonZeroUsize {
    NonZeroUsize::MIN
}
fn assert_page(page: Page, identity: &str, next_ordinal: Option<u64>) -> Option<Token> {
    assert!(page.receipt().basis_released());
    assert_eq!(page.rows().len(), 1);
    assert_eq!(page.rows()[0].directory, "catalog");
    assert_eq!(page.rows()[0].children.len(), 1);
    assert_eq!(page.rows()[0].children[0].identity, identity);
    assert!(matches!(
        &page.rows()[0].children[0].name,
        WorthQueryApplicationDisclosed::Omitted(_)
    ));
    let (_, next, _) = page.into_parts();
    assert_eq!(
        next.as_ref().map(|token| token.page_ordinal()),
        next_ordinal
    );
    next
}
#[test]
fn protected_continuation_readmits_current_capability_read_and_hidden_pagination_influence() {
    lifecycle::isolated_allocation_case(run,concat!(module_path!(),"::protected_continuation_readmits_current_capability_read_and_hidden_pagination_influence"));
}
fn run() {
    let calls = Arc::new(prefix::Calls::default());
    let world = host::install(calls.clone(), Arc::new(rule::Calls::new(calls)));
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    for (offset, (identity, name)) in [
        ("page-a", "01-secret"),
        ("page-b", "02-secret"),
        ("page-c", "03-secret"),
    ]
    .into_iter()
    .enumerate()
    {
        create(
            &world,
            &principal,
            &scope,
            identity,
            name,
            0x919a_1000 + offset as u64,
        );
    }
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let refusal = request
        .query(input())
        .page(width())
        .err()
        .expect("governed page requires current capability");
    assert!(
        matches!(refusal,WorthQueryApplicationRequestQueryDenial::Admission(cause) if cause.kind()==WorthQueryApplicationQueryAdmissionDenialKind::DisclosureGovernanceRequired)
    );
    let refusal = request
        .query(page::ReadBlockedPage {
            directory: "catalog".into(),
        })
        .page_governed(
            access::ReadBoxes::reference(),
            access::InspectBoxes::reference(),
            purpose(),
            width(),
        )
        .err()
        .expect("hidden ordering lacks Pagination influence");
    assert!(
        matches!(refusal,WorthQueryApplicationRequestQueryDenial::Admission(cause) if cause.kind()==WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid)
    );
    let open = || {
        request
            .query(input())
            .page_governed(
                access::ReadBoxes::reference(),
                access::InspectBoxes::reference(),
                purpose(),
                width(),
            )
            .unwrap()
    };
    let old_commit = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    // Each cursor comes from its own genuine first-page read; denial consumes it.
    let widened = assert_page(open(), "page-a", Some(2)).unwrap();
    let refusal = request
        .query(input())
        .resume_governed(
            access::ReadBoxes::reference(),
            access::InspectBoxes::reference(),
            purpose(),
            widened,
            NonZeroUsize::new(17).unwrap(),
        )
        .err()
        .expect("requested resume width cannot exceed its installed binding maximum16");
    assert!(
        matches!(refusal, WorthQueryApplicationRequestQueryDenial::Limit(_)),
        "{refusal:?}"
    );
    let raw = assert_page(open(), "page-a", Some(2)).unwrap();
    let revoked = assert_page(open(), "page-a", Some(2)).unwrap();
    let unreadable = assert_page(open(), "page-a", Some(2)).unwrap();
    let retained = assert_page(open(), "page-a", Some(2)).unwrap();
    let refusal = request
        .query(input())
        .resume(raw, width())
        .err()
        .expect("cursor alone retains no disclosure authority");
    assert!(
        matches!(refusal,WorthQueryApplicationRequestQueryDenial::Admission(cause) if cause.kind()==WorthQueryApplicationQueryAdmissionDenialKind::DisclosureGovernanceRequired)
    );
    set_status(&world, &principal, &scope, false, true, 0x919a_1010);
    let refusal = request
        .query(input())
        .resume_governed(
            access::ReadBoxes::reference(),
            access::InspectBoxes::reference(),
            purpose(),
            revoked,
            width(),
        )
        .err()
        .expect("revoked current grant cannot resume retained data");
    assert!(
        matches!(refusal,WorthQueryApplicationRequestQueryDenial::CapabilityAdmission(cause) if cause.kind()==WorthQueryOperationAuthorizationDenialKind::Refused(WorthQueryOperationAuthorizationRefusalKind::CapabilityAuthorizationMissing))
    );
    set_status(&world, &principal, &scope, true, false, 0x919a_1011);
    let refusal = request
        .query(input())
        .resume_governed(
            access::ReadBoxes::reference(),
            access::InspectBoxes::reference(),
            purpose(),
            unreadable,
            width(),
        )
        .err()
        .expect("fresh capability cannot replace current independent READ");
    assert!(
        matches!(refusal,WorthQueryApplicationRequestQueryDenial::Admission(cause) if cause.kind()==WorthQueryApplicationQueryAdmissionDenialKind::Authorization(WorthQueryOperationAuthorizationDenialKind::Refused(WorthQueryOperationAuthorizationRefusalKind::PermissionDenied)))
    );
    set_status(&world, &principal, &scope, true, true, 0x919a_1012);
    // Reorder b before the old boundary in current data. Substituting current
    // data for the retained generation would skip b and incorrectly return c.
    let source = request
        .query(source::FindPrefix {
            directory: "catalog".into(),
            prefix: "02".into(),
        })
        .execute()
        .unwrap();
    let changed = request
        .advanced()
        .mutate(mutation::Rename {
            directory: "catalog".into(),
            item: "page-b".into(),
            prefix: "02".into(),
            replacement: "00-moved".into(),
        })
        .expect_result_set(source.into_result_set_observation().unwrap())
        .idempotency(&0x919a_1013u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            changed,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{changed:?}"
    );
    assert_ne!(
        request.retain_current_read().unwrap().selected_commit(),
        &old_commit
    );
    let next = request
        .query(input())
        .resume_governed(
            access::ReadBoxes::reference(),
            access::InspectBoxes::reference(),
            purpose(),
            retained,
            width(),
        )
        .unwrap();
    let WorthQueryApplicationBasisSelectionIdentity::Product(selected) =
        next.receipt().basis_identity().selection()
    else {
        panic!("real retained World selection")
    };
    assert_eq!(selected.selected_commit(), &old_commit);
    let final_token = assert_page(next, "page-b", Some(3)).unwrap();
    let last = request
        .query(input())
        .resume_governed(
            access::ReadBoxes::reference(),
            access::InspectBoxes::reference(),
            purpose(),
            final_token,
            width(),
        )
        .unwrap();
    assert!(assert_page(last, "page-c", None).is_none());
    // A genuinely fresh opening reflects the current reordered data.
    // compiled-example: begin
    let first = request
        .query(input())
        .page_governed(
            access::ReadBoxes::reference(),
            access::InspectBoxes::reference(),
            purpose(),
            width(),
        )
        .unwrap();
    let (rows, continuation, _) = first.into_parts();
    let second = request
        .query(input())
        .resume_governed(
            access::ReadBoxes::reference(),
            access::InspectBoxes::reference(),
            purpose(),
            continuation.expect("another child page"),
            width(),
        )
        .unwrap();
    // compiled-example: end
    assert_eq!(rows[0].children[0].identity, "page-b");
    assert_page(second, "page-a", Some(3));
}
