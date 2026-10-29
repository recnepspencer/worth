//! Ordinary precise retirement preserves already owned historical results.
use super::*;
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use worth_query_host::facade::primary_graph::{
    MutationHandlerExecutionDenial, WorthQueryEntityResolutionDenialKind as ScopeKind,
    WorthQueryEntityResolutionRefusalKind as ResolutionRefusal, WorthQueryOutputDemandDenialKind,
    WorthQueryPriorOutputDenial, WorthQueryPriorOutputDenialKind,
};
use worth_query_host::facade::{application_entry::*, runtime::ExecutionAllocationPolicy};
#[test]
fn actual_v2_three_role_retirement_unlinks_only_observed_edges_and_reopens_current_source() {
    let original = host::install(model::authored_value()).unwrap();
    let _old = host::produce(&original);
    let target = journey::successor(host::capture(&original));
    let _new = successor::host::produce(&target);
    journey::assert_target_pages(&target, false);
    let source_id = successor::host::source_id(&target);
    let owned = successor::host::read_derived(&target);
    let scope = request_scope();
    let principal = authenticate_operator(target.installed_schema(), &scope);
    let request = target
        .runtime()
        .request(&principal, &scope)
        .on_branch(target.runtime().current_world());
    let observed = request
        .query(successor::query::ReadValue {
            identity: "authored-nested".into(),
        })
        .execute()
        .unwrap();
    let outcome = request
        .advanced()
        .mutate(successor::retire::Input {
            identity: "authored-nested".into(),
        })
        .expect_source(observed.observed_sources()[0].clone())
        .idempotency(&180601)
        .execute_in_program(&target, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("precise native retirement must publish")
    };
    let roles = receipt
        .outputs_of::<successor::output::RetiredOutputs>()
        .unwrap();
    assert!(roles.entity::<successor::output::RetiredOutput>().is_ok());
    assert!(roles.entity::<successor::output::RetiredLeft>().is_ok());
    assert!(roles.entity::<successor::output::RetiredRight>().is_ok());
    assert!(request
        .query(successor::page_query::ReadPage {
            identity: "derived-report".into()
        })
        .page(std::num::NonZeroUsize::MIN)
        .is_err());
    assert_eq!(successor::output::literals(&owned).1, 1012);
    assert_eq!(successor::host::read(&target), model::authored_value());
    let checkpoint = successor::host::capture(&target);
    drop(target);
    let reopened = successor::host::restore(checkpoint);
    assert_eq!(successor::host::source_id(&reopened), source_id);
    assert_eq!(successor::host::read(&reopened), model::authored_value());
    assert_eq!(successor::output::literals(&owned).1, 1012);
}

#[test]
fn retired_outputs_reissue_from_current_source_without_resurrecting_old_full_generations() {
    let a = model::authored_value();
    let mut b = a.clone();
    b.title = "intervening".into();
    let application = host::install(a.clone()).unwrap();
    let source_id = host::source_id(&application);
    let original = host::produce(&application);
    let original_receipt = original.application_commit_receipt().unwrap().clone();
    let old_ids = host::output_ids(&original);
    let b_receipt = host::edit_source(&application, b.clone(), 180611);
    assert!(!b_receipt.is_same_authoritative_commit(&original_receipt));
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let mut b_demand = request
        .demand(producer::Demand)
        .start_in_program::<_, program::Root>(&application)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(at_b) =
        b_demand.settle(&request).unwrap()
    else {
        panic!("the required B output must already be published");
    };
    assert_eq!(at_b.producer_contacts_in_this_demand(), 0);
    assert!(at_b
        .application_commit_receipt()
        .unwrap()
        .is_same_authoritative_commit(&b_receipt));
    let preserved = at_b.outputs_of::<output::PreservedOutputs>().unwrap();
    assert_eq!(
        [
            preserved
                .entity::<output::PreservedOutput>()
                .unwrap()
                .entity_id(),
            preserved
                .entity::<output::PreservedLeft>()
                .unwrap()
                .entity_id(),
            preserved
                .entity::<output::PreservedRight>()
                .unwrap()
                .entity_id(),
        ],
        old_ids
    );
    assert_eq!(host::read(&application), b);
    let b_value = host::read_derived(&application);
    assert_eq!(output::literals(&b_value).0, "INTERVENING");
    assert_eq!(output::literals(&b_value).1, 12);
    let b_read = request.retain_current_read().unwrap();
    request
        .at(&b_read)
        .require_current_output_demand(&at_b)
        .unwrap();
    drop(b_demand);

    // This installed action observes and unlinks the actual child relations,
    // deletes all three actual output entities, and publishes typed Retire roles.
    let observed = request
        .query(query::ReadValue {
            identity: "authored-nested".into(),
        })
        .execute()
        .unwrap();
    let retired = request
        .advanced()
        .mutate(retire::Input {
            identity: "authored-nested".into(),
        })
        .expect_source(observed.observed_sources()[0].clone())
        .idempotency(&180612)
        .execute_in_program(&application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed {
        receipt: retired, ..
    } = retired
    else {
        panic!("the ordinary output retirement must commit");
    };
    let roles = retired.outputs_of::<output::RetiredOutputs>().unwrap();
    assert_eq!(
        [
            roles.entity::<output::RetiredOutput>().unwrap().entity_id(),
            roles.entity::<output::RetiredLeft>().unwrap().entity_id(),
            roles.entity::<output::RetiredRight>().unwrap().entity_id(),
        ],
        old_ids
    );
    for identity in ["derived-report", "derived-left", "derived-right"] {
        let missing = request
            .query(derived_query::ReadValue {
                identity: identity.into(),
            })
            .execute()
            .err()
            .expect("each retired output is genuinely absent");
        assert!(
            matches!(&missing, WorthQueryApplicationRequestQueryDenial::ScopeResolution(cause)
            if cause.kind() == ScopeKind::Refused(ResolutionRefusal::UnknownEntity)),
            "{missing:?}"
        );
    }
    let retired_read = request.retain_current_read().unwrap();
    for stale in [&original, &at_b] {
        assert!(
            matches!(request.at(&retired_read).require_current_output_demand(stale),
            Err(WorthQueryOutputCurrentnessDenial::Output(cause)) if cause.kind() == WorthQueryOutputDemandDenialKind::Superseded)
        );
    }

    // A selected correspondence for dead output remains a typed integrity
    // failure. It must not fall back to the still older Create correspondence.
    let observed = request
        .query(query::ReadValue {
            identity: "authored-nested".into(),
        })
        .execute()
        .unwrap();
    let denied = request
        .advanced()
        .mutate(retire::Input {
            identity: "authored-nested".into(),
        })
        .expect_source(observed.observed_sources()[0].clone())
        .idempotency(&180614)
        .execute_in_program(&application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap_err();
    let WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Handler(denied),
    ) = denied
    else {
        panic!("the retired prior role must refuse: {denied:?}");
    };
    let denied = denied.downcast::<WorthQueryPriorOutputDenial>().unwrap();
    assert_eq!(
        denied.kind(),
        WorthQueryPriorOutputDenialKind::OutputUnavailable
    );
    assert_eq!(denied.role(), "derived-report");
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        retired_read.selected_commit()
    );

    // The source returns to A only through the performed source action's real
    // required-output custody. Native liveness must select fresh Initial output,
    // even though this is the same producer and parameters as the original A.
    let reissued_receipt = host::edit_source(&application, a.clone(), 180613);
    assert_eq!(
        reissued_receipt.installed_operation(),
        original_receipt.installed_operation()
    );
    for earlier in [&original_receipt, &b_receipt, &retired] {
        assert!(!reissued_receipt.is_same_authoritative_commit(earlier));
    }
    let created = reissued_receipt
        .outputs_of::<output::DerivedOutputs>()
        .unwrap();
    let new_ids = [
        created
            .entity::<output::CreatedOutput>()
            .unwrap()
            .entity_id(),
        created.entity::<output::CreatedLeft>().unwrap().entity_id(),
        created
            .entity::<output::CreatedRight>()
            .unwrap()
            .entity_id(),
    ];
    for entity in new_ids {
        assert_ne!(entity.generation_value(), 0);
        assert!(
            !old_ids.contains(&entity),
            "a retired full generation must never be reissued"
        );
    }
    assert_eq!(host::source_id(&application), source_id);
    assert_eq!(host::read(&application), a);
    let actual = host::read_derived(&application);
    assert_eq!(output::literals(&actual).0, "ASSEMBLY \u{0392}");
    assert_eq!(output::literals(&actual).1, 12);
    assert_eq!(output::literals(&actual).2, ["left", "right"]);
    let mut demand = request
        .demand(producer::Demand)
        .start_in_program::<_, program::Root>(&application)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(reissued) =
        demand.settle(&request).unwrap()
    else {
        panic!("the required publication must expose its actual new output");
    };
    assert_eq!(reissued.producer_contacts_in_this_demand(), 0);
    assert!(reissued
        .application_commit_receipt()
        .unwrap()
        .is_same_authoritative_commit(&reissued_receipt));
    assert_eq!(host::output_ids(&reissued), new_ids);
    let current = request.retain_current_read().unwrap();
    request
        .at(&current)
        .require_current_output_demand(&reissued)
        .unwrap();
    for stale in [&original, &at_b] {
        assert!(
            matches!(request.at(&current).require_current_output_demand(stale),
            Err(WorthQueryOutputCurrentnessDenial::Output(cause)) if cause.kind() == WorthQueryOutputDemandDenialKind::Superseded)
        );
    }
    drop(demand);
    let mut retry = request
        .demand(producer::Demand)
        .start_in_program::<_, program::Root>(&application)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(unchanged) =
        retry.settle(&request).unwrap()
    else {
        panic!("unchanged reissued output must remain current");
    };
    assert_eq!(unchanged.producer_contacts_in_this_demand(), 0);
    assert!(unchanged
        .application_commit_receipt()
        .unwrap()
        .is_same_authoritative_commit(&reissued_receipt));
    assert_eq!(host::output_ids(&unchanged), new_ids);
    let retry_read = request.retain_current_read().unwrap();
    request
        .at(&retry_read)
        .require_current_output_demand(&unchanged)
        .unwrap();
    request
        .at(&retry_read)
        .require_current_output_demand(&reissued)
        .unwrap();
    drop(retry);

    // A real later source edit must Preserve these NEW generations, rather
    // than selecting the old Preserve row or reopening Initial/Birth.
    let mut c = a.clone();
    c.title = "after reissue".into();
    let c_receipt = host::edit_source(&application, c.clone(), 180615);
    assert!(!c_receipt.is_same_authoritative_commit(&reissued_receipt));
    assert_eq!(
        c_receipt.installed_operation(),
        b_receipt.installed_operation()
    );
    let roles = c_receipt.outputs_of::<output::PreservedOutputs>().unwrap();
    assert_eq!(
        [
            roles
                .entity::<output::PreservedOutput>()
                .unwrap()
                .entity_id(),
            roles.entity::<output::PreservedLeft>().unwrap().entity_id(),
            roles
                .entity::<output::PreservedRight>()
                .unwrap()
                .entity_id(),
        ],
        new_ids
    );
    assert_eq!(host::read(&application), c);
    assert_eq!(
        output::literals(&host::read_derived(&application)).0,
        "AFTER REISSUE"
    );
    let mut demand = request
        .demand(producer::Demand)
        .start_in_program::<_, program::Root>(&application)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(at_c) = demand.settle(&request).unwrap()
    else {
        panic!("the subsequent Preserve must already be current");
    };
    assert_eq!(at_c.producer_contacts_in_this_demand(), 0);
    assert!(at_c
        .application_commit_receipt()
        .unwrap()
        .is_same_authoritative_commit(&c_receipt));
    let current = request.retain_current_read().unwrap();
    request
        .at(&current)
        .require_current_output_demand(&at_c)
        .unwrap();
    assert!(
        matches!(request.at(&current).require_current_output_demand(&reissued),
        Err(WorthQueryOutputCurrentnessDenial::Output(cause))
            if cause.kind() == WorthQueryOutputDemandDenialKind::Superseded)
    );
}
