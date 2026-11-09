//! Actual Native allocations remain in their creator's custody before World.
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_relational::facade::{
    inspection::observe_maintenance_work, publication::AuthoredRetirementRefusal as Cause,
    transactions::RecordRef,
};
fn refuses(worker: bool, authored: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = if worker {
        world::install_worker_only()
    } else {
        world::install()
    };
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let owner = world::model(&host, host.current_world()).native_id;
    let before = failure::checkpoint(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let _fault = if authored {
        host.runtime()
            .destroy_the_authored_prefixs_fresh_record_for_test()
    } else {
        host.runtime()
            .destroy_another_writers_fresh_record_for_test()
    };
    let (outcome, work) = observe_maintenance_work(16, || {
        if authored {
            request
                .advanced()
                .mutate(new_source::Intent {
                    key: "fresh".into(),
                    value: 2,
                })
                .without_source()
                .idempotency(&"fresh-prefix-destruction".to_owned())
                .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
                .unwrap()
        } else {
            request
                .advanced()
                .mutate(Intent {
                    key: "a".into(),
                    value: 2,
                })
                .without_source()
                .idempotency(&"fresh-writer-destruction".to_owned())
                .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
                .unwrap()
        }
    });
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("a writer must not destroy another writer's actual fresh allocation");
    };
    let error = denial
        .authored_retirement_error()
        .expect("Native fresh custody refusal");
    let Some(Cause::ForeignFreshCoupledRecord {
        creator,
        writer,
        owner: actual_owner,
        source,
    }) = error.authored_retirement_refusal()
    else {
        panic!(
            "expected exact fresh-record custody, got {:?}",
            error.authored_retirement_refusal()
        );
    };
    assert_eq!(actual_owner, owner);
    if authored {
        assert_eq!(creator, None);
        assert_eq!(writer, 0);
        assert_ne!(source, owner);
        assert_eq!(error.context.affected_records[0], RecordRef::Entity(source));
    } else {
        assert_eq!(creator, Some(0));
        assert_eq!(writer, 1);
        assert_eq!(source, owner);
        assert_ne!(error.context.affected_records[0], RecordRef::Entity(owner));
    }
    assert_eq!(
        error.context.affected_records[1..],
        [RecordRef::Entity(owner), RecordRef::Entity(source)]
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test() - contacts,
        if authored { 1 } else { 2 }
    );
    assert_eq!(work.published_rows.len() + work.omitted_publications, 0);
    failure::assert_model_unchanged(&host, &before);
}
#[test]
fn a_real_suffix_cannot_destroy_another_routes_fresh_record() {
    refuses(false, false);
}
#[test]
fn worker_only_real_suffix_cannot_destroy_another_routes_fresh_record() {
    refuses(true, false);
}
#[test]
fn a_real_suffix_cannot_destroy_the_authored_prefixs_fresh_source() {
    refuses(false, true);
}
#[test]
fn worker_only_real_suffix_cannot_destroy_the_authored_prefixs_fresh_source() {
    refuses(true, true);
}
