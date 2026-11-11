//! Independent read admission and exact predecessor values through public demands.
use super::*;
use family::predecessor_family as derived;
pub(super) fn reset(wrong_scope: bool) {
    PROBE.with(|probe| {
        *probe.borrow_mut() = Probe {
            wrong_scope,
            ..Probe::default()
        }
    });
}
pub(super) fn settle(
    request: &family::Request<'_, '_, '_>,
    host: &Host,
) -> Result<(), WorthQueryOutputDemandDenial> {
    let mut demand = request
        .demand(derived::Demand("a".into()))
        .retain_program_outputs::<program::Program, program::Tree>(host)
        .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
    for _ in 0..128 {
        demand = match demand.advance(request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
            WorthQueryProgramOutputOperationAdvance::Settled(settled) => {
                settled.deliver_current(request).unwrap();
                settled
                    .close(request)
                    .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
                return Ok(());
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                return Err(demand_denial(&denial))
            }
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                let denial = demand_denial(terminal.denial());
                terminal
                    .close(request)
                    .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
                return Err(denial);
            }
        };
    }
    panic!("predecessor demand did not settle");
}
pub(super) fn edit_model<'a>(
    document: world::Document<'a>,
    host: &'a Host,
    request: &family::Request<'_, '_, '_>,
    value: u64,
) -> world::Document<'a> {
    let mut edit = document
        .edit(edit::Intent {
            key: "a".into(),
            value,
        })
        .idempotency_owned(request, format!("predecessor-model-{value}"))
        .unwrap()
        .retain_in_program::<program::Program, super::super::program::EditAction>(host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    loop {
        edit = match edit.advance(request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(next) => {
                edit = next;
                break;
            }
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
        };
    }
    edit.finish(request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        .document
}
fn exact_old_inputs(host: &Host) {
    reset(false);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle(&request, host).unwrap();
    PROBE.with(|probe| {
        assert_eq!(
            probe.borrow().values.as_slice(),
            &[Err(
                WorthQueryPredecessorReadDenial::PredecessorBasisUnavailable
            )]
        )
    });
    let document = edit_model(world::attach(host), host, &request, 2);
    let branch = document
        .selection(&request)
        .unwrap()
        .physical_endpoint()
        .product_branch();
    let edited_request = host.request(&principal, &scope).on_branch(branch);
    settle(&edited_request, host).unwrap();
    PROBE.with(|probe| {
        assert_eq!(
            probe.borrow().values.last(),
            Some(&Ok(Some(1))),
            "old inputs must differ from C value 2"
        )
    });
    let count = PROBE.with(|probe| probe.borrow().values.len());
    settle(&edited_request, host).unwrap();
    PROBE.with(|probe| {
        assert_eq!(
            probe.borrow().values.len(),
            count,
            "a current output must not reenter its producer"
        )
    });
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
#[test]
fn exact_old_inputs_and_current_retry() {
    exact_old_inputs(&install(false));
}
#[test]
fn worker_only_old_input_read() {
    exact_old_inputs(&install(true));
}

#[test]
fn repeated_loans_in_one_admitted_producer_retain_the_same_predecessor_truth() {
    for worker in [false, true] {
        let host = install(worker);
        reset(false);
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        settle(&request, &host).unwrap();
        let document = edit_model(world::attach(&host), &host, &request, 2);
        let branch = document
            .selection(&request)
            .unwrap()
            .physical_endpoint()
            .product_branch();
        let request = host.request(&principal, &scope).on_branch(branch);
        PROBE.with(|probe| {
            probe.borrow_mut().around_read = Some(Box::new(|read| {
                let first = read();
                assert_eq!(first, Ok(Some(1)), "the first loan observes D, not C");
                let second = read();
                assert_eq!(
                    second, first,
                    "fresh authorization must merge by full truth"
                );
                second
            }));
        });
        settle(&request, &host).unwrap();
        PROBE.with(|probe| {
            assert!(
                probe.borrow().around_read.is_none(),
                "producer ran both loans"
            );
            assert_eq!(probe.borrow().values.last(), Some(&Ok(Some(1))));
        });
        assert_eq!(
            request
                .query(query::first_output::Read("predecessor:a".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .value,
            9002,
            "the read-only loans preserve the producer's declared current-value output"
        );
        let reads = PROBE.with(|probe| probe.borrow().values.len());
        settle(&request, &host).unwrap();
        PROBE.with(|probe| assert_eq!(probe.borrow().values.len(), reads));
        host.with_host_advancement(|phase| document.close(&phase).unwrap())
            .unwrap();
    }
}

#[test]
fn read_scope_mismatch_cannot_be_swallowed_by_authorized_producer() {
    let host = install(false);
    reset(true);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = host.current_world();
    let denial =
        settle(&request, &host).expect_err("producer grant cannot authorize an unrelated scope");
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::RequestAuthorization(
            WorthQueryOperationAuthorizationRefusalKind::ScopeMismatch
        )
    );
    assert!(denial.subject().contains("ReadWrongScope"), "{denial:?}");
    assert_eq!(host.current_world(), before);
}
#[test]
fn producer_permission_does_not_supply_missing_read_permission() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = install(false);
    reset(false);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle(&request, &host).unwrap();
    let document = world::attach(&host);
    let mut edit = document
        .edit(member_edit::Intent {
            key: "m".into(),
            value: 6,
        })
        .idempotency_owned(&request, "predecessor-revoke-read".to_owned())
        .unwrap()
        .retain_in_program::<program::Program, super::super::program::MemberEditAction>(&host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    loop {
        edit = match edit.advance(&request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(next) => {
                edit = next;
                break;
            }
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
        };
    }
    let document = edit
        .finish(&request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        .document;
    let document = edit_model(document, &host, &request, 2);
    let branch = document
        .selection(&request)
        .unwrap()
        .physical_endpoint()
        .product_branch();
    let request = host.request(&principal, &scope).on_branch(branch);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let before = document.selection(&request).unwrap();
    let denial =
        settle(&request, &host).expect_err("C lacks the read grant even though D allowed it");
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 1,
        "outer producer admitted and ran"
    );
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::RequestAuthorization(
            WorthQueryOperationAuthorizationRefusalKind::PermissionDenied
        )
    );
    assert!(
        denial.subject().contains("ReadOldInputs") && denial.subject().contains("InputReadPolicy"),
        "{denial:?}"
    );
    assert_eq!(
        document.selection(&request).unwrap(),
        before,
        "no output or history effect"
    );
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}

pub(super) fn demand_denial(
    denial: &WorthQueryProgramOutputAdvanceDenial,
) -> WorthQueryOutputDemandDenial {
    let WorthQueryProgramOutputAdvanceDenial::Outputs(
        WorthQueryRequiredOutputPreparationDenial::Demand(
            WorthQueryApplicationOutputDemandDenial::Demand(denial),
        ),
    ) = denial
    else {
        panic!("{denial:?}")
    };
    denial.clone()
}

#[test]
fn undeclared_read_cannot_be_swallowed() {
    let host = install(false);
    reset(false);
    PROBE.with(|probe| probe.borrow_mut().undeclared = true);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = host.current_world();
    let denial =
        settle(&request, &host).expect_err("an installed READ is not a declared predecessor read");
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::PredecessorRead(Box::new(
            WorthQueryPredecessorReadDenial::UndeclaredPredecessorRead
        ))
    );
    assert_eq!(host.current_world(), before);
}
