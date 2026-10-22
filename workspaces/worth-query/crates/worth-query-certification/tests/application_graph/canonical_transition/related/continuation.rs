//! A genuine old ordered token retains changed derived payload across adoption.
use super::*;
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use std::num::NonZeroUsize;
use worth_query_host::facade::{application_entry::*, primary_graph::*};
#[path = "continuation/native.rs"]
mod native;
#[path = "continuation/refusals.rs"]
mod refusals;

#[test]
fn actual_related_v1_token_keeps_old_values_while_v2_produces_edits_and_reopens() {
    let original = host::install(model::authored_value()).unwrap();
    let source_id = host::source_id(&original);
    assert_eq!(host::read(&original), model::authored_value());
    let catalogue = original.installed_schema().value_bindings().clone();
    let settled = host::produce(&original);
    let old_ids = host::output_ids(&settled);
    let observation = settled.observation().retain();
    let first = journey::page(&original);
    journey::assert_old(&first, "derived-left");
    assert!(output::decodes() > 0);
    let generation = first
        .receipt()
        .ordered_index_generation()
        .expect("actual ordered generation");
    let (old_rows, token, receipt) = first.into_parts();
    assert!(receipt.basis_released());
    let token = token.expect("real second child");
    assert_eq!(token.page_ordinal(), 2);
    let checkpoint = host::capture(&original);
    let retained = checkpoint.clone();
    let bytes = checkpoint.bytes().to_vec();
    let target = journey::successor(checkpoint);
    assert_eq!(successor::output::decodes(), 0);
    assert_eq!(successor::host::source_id(&target), source_id);
    assert_eq!(successor::host::read(&target), model::authored_value());
    assert!(catalogue.same_installation_as(original.installed_schema().value_bindings()));
    assert!(!catalogue.same_installation_as(target.installed_schema().value_bindings()));
    let foreign = target
        .runtime()
        .with_host_advancement(|phase| {
            observation
                .select_on(&phase, target.runtime())
                .err()
                .expect("old observation cannot select target owner")
        })
        .unwrap();
    assert!(matches!(
        foreign,
        WorthQueryProductBranchAdmissionDenial::Refused(
            WorthQueryProductBranchAdmissionRefusalKind::ForeignOwner
        )
    ));
    assert_eq!(successor::output::decodes(), 0);
    let target_settled = successor::host::produce(&target);
    let new_ids = successor::host::output_ids(&target_settled);
    assert!(new_ids.iter().all(|id| !old_ids.contains(id)));
    journey::assert_target_pages(&target, false);
    let updated = host::edit_source(&original, journey::edited(), 0x2711801);
    let roles = updated.outputs_of::<output::PreservedOutputs>().unwrap();
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
                .entity_id()
        ],
        old_ids
    );
    assert_eq!(output::literals(&host::read_derived(&original)).1, 37);
    journey::assert_original_current(&original);
    let scope = request_scope();
    let principal = authenticate_operator(original.installed_schema(), &scope);
    let second = original
        .runtime()
        .request(&principal, &scope)
        .on_branch(original.runtime().current_world())
        .query(page_query::ReadPage {
            identity: "derived-report".into(),
        })
        .resume(token, NonZeroUsize::MIN)
        .unwrap();
    journey::assert_old(&second, "derived-right");
    assert_eq!(
        second.receipt().ordered_index_generation(),
        Some(generation)
    );
    assert!(second.continuation().is_none());
    assert_eq!(output::literals(&old_rows[0].children[0].1).1, 12);
    let updated = successor::host::edit_source(&target, journey::edited(), 0x2711802);
    let roles = updated
        .outputs_of::<successor::output::PreservedOutputs>()
        .unwrap();
    assert_eq!(
        [
            roles
                .entity::<successor::output::PreservedOutput>()
                .unwrap()
                .entity_id(),
            roles
                .entity::<successor::output::PreservedLeft>()
                .unwrap()
                .entity_id(),
            roles
                .entity::<successor::output::PreservedRight>()
                .unwrap()
                .entity_id()
        ],
        new_ids
    );
    journey::assert_target_pages(&target, true);
    let current = successor::host::capture(&target);
    drop(target_settled);
    drop(target);
    let reopened = successor::host::restore(current);
    assert_eq!(successor::host::source_id(&reopened), source_id);
    journey::assert_target_pages(&reopened, true);
    assert_eq!(retained.bytes(), bytes);
    assert!(original.runtime().close_live_delivery().owner_terminal());
    assert_eq!(output::literals(&old_rows[0].children[0].1).1, 12);
}
