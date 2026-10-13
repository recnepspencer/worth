//! Exact conditional membership is not an ordinary mutation permission.
use super::*;
use worth_query_consumer_values::PlanarOperation;
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial;

#[test]
fn conditional_binding_refuses_ordinary_dispatch_without_mutation() {
    let _guard = checkpoint_recovery_test_guard();
    let application = support::install_program::<DemandProgram>(None, Default::default());
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let before = request.retain_current_read().unwrap();
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    let denied = request
        .mutate(PlanarMutation {
            scope_key: "anchor-a".to_owned(),
            operation: PlanarOperation::VerifyCurrentOutputs(vec![]),
        })
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&30_003)
        .execute_in_program(&application, ExecutionAllocationPolicy::SystemAllocation);
    assert!(
        matches!(
            denied,
            Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired)
        ),
        "conditional membership cannot authorize an ordinary call: {denied:?}"
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        before.selected_commit()
    );
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: "anchor-a".to_owned()
            })
            .execute()
            .unwrap()
            .rows(),
        source.rows()
    );
}
