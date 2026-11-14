//! Real issued lifecycle receipts distinguish complete root truth from collateral.
use super::*;
fn original() -> query::Row {
    query::Row {
        class: 1,
        first: Some(11),
        second: None,
        required: None,
        third: None,
    }
}
fn retired(host: &world::Host) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let result = host
        .runtime()
        .request(&principal, &scope)
        .query(query::Read {
            key: "record".into(),
        })
        .execute();
    let cause = match result {
        Err(WorthQueryApplicationRequestQueryDenial::ScopeResolution(cause)) => cause,
        Err(other) => panic!("retired root wrong denial: {other:?}"),
        Ok(_) => panic!("retired root still resolves"),
    };
    assert_eq!(
        cause.kind(),
        WorthQueryEntityResolutionDenialKind::Refused(
            WorthQueryEntityResolutionRefusalKind::UnknownEntity
        )
    );
}
fn restored(host: &world::Host) {
    assert_eq!(world::read(host), original());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let result = request
        .query(sparse::link_query::Read {
            key: "record".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(result.rows()[0].annotation, Some(21));
    assert_eq!(
        result.rows()[0].note,
        None,
        "present annotation retains field absence"
    );
    let collateral = request
        .query(query::Read {
            key: "neighbour".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        collateral.rows()[0],
        query::Row {
            class: 3,
            first: None,
            second: None,
            required: None,
            third: Some(33)
        }
    );
    let collateral = request
        .query(sparse::link_query::Read {
            key: "neighbour".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(collateral.rows()[0].annotation, Some(81));
    assert_eq!(collateral.rows()[0].note, Some(78));
}
fn retire(host: &world::Host) -> WorthQueryApplicationCommitReceipt {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let outcome = host
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(retire::Input {
            key: "record".into(),
            collateral_third_observation: false,
        })
        .without_source()
        .idempotency(&"complete-retirement".to_owned())
        .execute_in_program(
            host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("retire: {outcome:?}")
    };
    receipt
}
fn reverse(
    host: &world::Host,
    source: &WorthQueryApplicationCommitReceipt,
    key: &str,
) -> WorthQueryApplicationCommitReceipt {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
    let selected = request.corrections().select(source, limits).unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency(key)
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("reverse must prepare")
    };
    let outcome = prepared.publish().unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("reverse: {outcome:?}")
    };
    receipt
}
#[test]
fn complete_retired_root_restores_value_field_absence_and_aspect_absence_then_reapplies() {
    let host = world::install_retirement();
    let source = retire(&host);
    let old_id = source
        .outputs_of::<retire::Outputs>()
        .unwrap()
        .entity::<retire::Retired>()
        .unwrap()
        .entity_id();
    retired(&host);
    let undo = reverse(&host, &source, "complete-undo");
    let successor = undo
        .outputs_of::<restore::Outputs>()
        .unwrap()
        .entity::<restore::Created>()
        .unwrap()
        .entity_id();
    assert_ne!(successor, old_id);
    restored(&host);
    let recovered = world::restore(world::checkpoint(&host));
    restored(&recovered);
    let host = recovered;
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
    let selected = request
        .corrections()
        .select_recorded(&source.correction_source(), limits)
        .unwrap();
    let page = request
        .corrections()
        .alternatives(&selected, limits, std::num::NonZeroUsize::MIN)
        .unwrap();
    assert_eq!(page.entries().len(), 1);
    assert_eq!(
        page.entries()[0].performed_commit(),
        undo.commit_reference()
    );
    let alternative = page.select(&page.entries()[0], limits).unwrap();
    let prepared = request
        .corrections()
        .reapply(&alternative)
        .idempotency("complete-reapply")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("reapply must prepare")
    };
    let outcome = prepared.publish().unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("reapply: {outcome:?}")
    };
    assert_eq!(
        receipt
            .outputs_of::<retire::Outputs>()
            .unwrap()
            .entity::<retire::Retired>()
            .unwrap()
            .entity_id(),
        successor
    );
    retired(&host);
}
#[test]
fn recorded_reverse_retirement_captures_the_newly_created_successor() {
    let host = world::install_retirement();
    let first = retire(&host);
    let original_id = first
        .outputs_of::<retire::Outputs>()
        .unwrap()
        .entity::<retire::Retired>()
        .unwrap()
        .entity_id();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let outcome = request
        .advanced()
        .mutate(restore::Input {
            image: Image {
                key: "record".into(),
                row: original(),
                annotation: Some(21),
                note: None,
            },
        })
        .without_source()
        .idempotency(&"ordinary-successor".to_owned())
        .execute_in_program(
            &host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed {
        receipt: created, ..
    } = outcome
    else {
        panic!("create successor: {outcome:?}")
    };
    let successor = created
        .outputs_of::<restore::Outputs>()
        .unwrap()
        .entity::<restore::Created>()
        .unwrap()
        .entity_id();
    assert_ne!(successor, original_id);
    restored(&host);
    let reversal = reverse(&host, &created, "reverse-created-successor");
    assert_eq!(
        reversal
            .outputs_of::<retire::Outputs>()
            .unwrap()
            .entity::<retire::Retired>()
            .unwrap()
            .entity_id(),
        successor
    );
    retired(&host);
}
#[test]
fn collateral_value_cannot_supply_a_missing_retired_root_observation() {
    let host = world::install_retirement();
    let before = world::pin(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let outcome = request
        .advanced()
        .mutate(retire::Input {
            key: "record".into(),
            collateral_third_observation: true,
        })
        .without_source()
        .idempotency(&"missing-root-third".to_owned())
        .execute_in_program(
            &host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("missing root truth: {outcome:?}")
    };
    assert_eq!(
        denial.preimage_retention_denial(),
        Some(WorthQueryPreImageRetentionDenial::MissingDemandedField)
    );
    assert_eq!(
        before.selected_commit(),
        world::pin(&host).selected_commit()
    );
    assert_eq!(world::read(&host), original());
    let collateral = request
        .query(sparse::link_query::Read {
            key: "neighbour".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        collateral.rows()[0].note,
        Some(77),
        "failed root capture publishes no collateral write"
    );
}
