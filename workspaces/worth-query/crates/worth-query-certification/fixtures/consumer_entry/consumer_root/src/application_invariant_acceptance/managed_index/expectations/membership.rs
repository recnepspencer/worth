//! Nonmatches and an empty Parent remain dependent on the complete Source roster.
use super::*;
#[test]
fn managed_row_expectation_preserves_nonmatches_and_recorded_retry() {
    run(row);
}
fn row(
    world: &host::Host,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Catalog>,
    scope: &WorthQueryRequestScope,
) {
    let request = world
        .runtime()
        .request(principal, scope)
        .on_branch(world.current_world());
    let index = request
        .query(find("al"))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    request.query(find("al")).rebuild_index(&index).unwrap();
    let original = request.query(find("al")).execute().unwrap();
    assert!(original.rows().iter().all(|row| row.identity != "item-a"));
    let source = original.observed_sources()[0].clone();
    let input = rename("al", "item-a", "alba");
    let first = request
        .advanced()
        .mutate(input.clone())
        .expect_source(source.clone())
        .idempotency(&0x919c001u64)
        .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        first,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    // The original Source owner survives the completed advancement. A new call
    // borrows its current request, and recorded resolution must skip execution.
    let retry_scope = authentication::request_scope();
    let retry_adapter = authentication::admit(world.installed_schema());
    let retry_principal = authentication::block_on(retry_adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &retry_scope,
    ))
    .unwrap();
    let retry_request = world
        .runtime()
        .request(&retry_principal, &retry_scope)
        .on_branch(world.current_world());
    let retry = retry_request
        .advanced()
        .mutate(input)
        .expect_source(source.clone())
        .idempotency(&0x919c001u64)
        .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        retry,
        WorthQueryApplicationMutationOutcome::AlreadyCommitted(_)
            | WorthQueryApplicationMutationOutcome::PreviouslyCommitted(_)
    ));
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    source_changed(
        request
            .advanced()
            .mutate(rename("al", "item-e", "changed"))
            .expect_source(source)
            .idempotency(&0x919c002u64)
            .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation),
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    let actual = request
        .query(query::ReadSource {
            directory: "catalog".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        actual
            .rows()
            .iter()
            .find(|row| row.identity == "item-a")
            .unwrap()
            .name,
        "alba"
    );
    assert_eq!(
        actual
            .rows()
            .iter()
            .find(|row| row.identity == "item-e")
            .unwrap()
            .name,
        ""
    );
    request.query(find("al")).rebuild_index(&index).unwrap();
    let indexed = request.query(find("al")).execute().unwrap();
    assert!(indexed.rows().iter().any(|row| row.identity == "item-a"));
    index.close();
}
#[test]
fn managed_empty_result_expectation_preserves_future_entrant_dependencies() {
    run(empty);
}
fn empty(
    world: &host::Host,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Catalog>,
    scope: &WorthQueryRequestScope,
) {
    let request = world
        .runtime()
        .request(principal, scope)
        .on_branch(world.current_world());
    let index = request
        .query(find("zzz"))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    request.query(find("zzz")).rebuild_index(&index).unwrap();
    let original = request.query(find("zzz")).execute().unwrap();
    assert!(original.rows().is_empty());
    let source = original.into_result_set_observation().unwrap();
    let first = request
        .advanced()
        .mutate(rename("zzz", "item-a", "zzz-entrant"))
        .expect_result_set(source.clone())
        .idempotency(&0x919c003u64)
        .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        first,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    source_changed(
        request
            .advanced()
            .mutate(rename("zzz", "item-e", "wrong"))
            .expect_result_set(source)
            .idempotency(&0x919c004u64)
            .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation),
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    request.query(find("zzz")).rebuild_index(&index).unwrap();
    let actual = request.query(find("zzz")).execute().unwrap();
    assert_eq!(
        actual
            .rows()
            .iter()
            .map(|row| row.identity.as_str())
            .collect::<Vec<_>>(),
        ["item-a"]
    );
    index.close();
}
