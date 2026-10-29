//! Foreign installation custody and wrong query parameters cannot become mutation input.
use super::*;
#[test]
fn managed_expectation_checks_actual_runtime_and_parameters_before_mutation() {
    run(affinity);
}
fn affinity(
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
    let source = original.observed_sources()[0].clone();
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let mismatch = request
        .advanced()
        .mutate(rename("zzz", "item-a", "no"))
        .expect_source(source.clone())
        .idempotency(&0x919c005u64)
        .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap_err();
    assert!(
        matches!(mismatch, WorthQueryApplicationRequestMutationDenial::SourceExpectation(ref cause) if cause.kind() == WorthQuerySourceExpectationDenialKind::SourceParametersMismatch)
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    let other = host::install(Arc::new(prefix::Calls::default()));
    let adapter = authentication::admit(other.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        scope,
    ))
    .unwrap();
    let other_request = other
        .runtime()
        .request(&principal, scope)
        .on_branch(other.current_world());
    let other_head = other_request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let foreign = other_request
        .advanced()
        .mutate(rename("al", "item-a", "no"))
        .expect_source(source)
        .idempotency(&0x919c006u64)
        .execute_in_program(&other, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap_err();
    assert!(
        matches!(foreign, WorthQueryApplicationRequestMutationDenial::SourceExpectation(ref cause) if cause.kind() == WorthQuerySourceExpectationDenialKind::ForeignApplication)
    );
    assert_eq!(
        other_request
            .retain_current_read()
            .unwrap()
            .selected_commit(),
        &other_head
    );
    index.close();
}
