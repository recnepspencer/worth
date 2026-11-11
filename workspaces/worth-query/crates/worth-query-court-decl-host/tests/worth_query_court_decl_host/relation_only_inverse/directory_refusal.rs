//! Relation material without a durable representation refuses at original admission.
use super::*;
use worth_query_host::facade::{
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
#[test]
fn directory_relation_inverse_refuses_before_original_publication_and_survives_reopen() {
    use worth_query_host::facade::{
        application_entry::WorthQueryApplicationRequestMutationDenialKind as Kind,
        product::WorthQueryDurableStateAdmissionDenial as Durable,
    };
    let path = std::env::temp_dir().join(format!(
        "worth-relation-only-home-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let budget = worth_execution::SerialMemoryBudget::new(Some(256 << 20));
    let cancellation = worth_execution::CancellationSource::new();
    let request = worth_execution::SerialRequest::from_memory(budget, cancellation.token(), None);
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let open = || {
        let request = WorthQueryApplicationHomeRequest::new(WorthQueryApplicationHomeLocation::Directory(path.clone()), ApplicationProgramAuthoring::<Schema, world::Program>::begin().validated_program().unwrap(), WorthQueryApplicationProgramRoster::new(), Schema::declaration().unwrap(), ((),), ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(), host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), world::seed);
        with_bootstrap_advancement_from_request(execution, |phase| {
            request.open_in_advancement(&phase)
        })
        .unwrap()
        .unwrap_or_else(|cause| panic!("actual relation Home: {cause:?}"))
    };
    let home = open();
    let before = home.runtime().runtime().current_world();
    {
        let scope = request_scope();
        let principal = authenticate_operator(home.runtime().installed_schema(), &scope);
        let request = home.runtime().runtime().request(&principal, &scope);
        let Err(denial) = request
            .advanced()
            .mutate(Spawn {
                id: 10,
                part: true,
                link: true,
                inverse_fault: 0,
            })
            .without_source()
            .idempotency(&"relation-original".to_owned())
            .execute_in_program(
                home.runtime(),
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
        else {
            panic!("relation carrier is not admitted");
        };
        assert!(matches!(
            denial.kind(),
            Kind::DurableStateAdmission(Durable::UnsupportedCorrectionRelationPreimage)
        ));
        assert_eq!(home.runtime().runtime().current_world(), before);
        assert_eq!(
            request
                .query(query::Read { id: SCOPE })
                .execute()
                .unwrap()
                .rows()
                .len(),
            1
        );
        assert!(matches!(
            request.query(query::Read { id: 10 }).execute(),
            Err(WorthQueryApplicationRequestQueryDenial::ScopeResolution(_))
        ));
    }
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("no original custody: {cause:?}"))
        .reclaim();
    let home = open();
    let scope = request_scope();
    let principal = authenticate_operator(home.runtime().installed_schema(), &scope);
    assert_eq!(
        home.runtime()
            .request(&principal, &scope)
            .query(query::Read { id: SCOPE })
            .execute()
            .unwrap()
            .rows()
            .len(),
        1
    );
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("unchanged relation Home: {cause:?}"))
        .reclaim();
}
