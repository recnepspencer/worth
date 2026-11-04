//! A fresh real Query retirement reads persisted co-anchor history through Native.
use super::{
    co_courts::{create, produce},
    correction_courts::install_program,
    lifecycle::*,
    program::*,
    *,
};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
type S = Model<5>;
#[test]
fn corrupt_recorded_co_anchor_refuses_at_the_real_closure_owner_with_its_source_subject() {
    let host = install_program::<S>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let original = create(&host, &request, 10, vec![20]);
    let subject = original
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<Created<S>>()
        .unwrap()
        .entity_id();
    create(&host, &request, 40, vec![]);
    produce(&host, &request, 10);
    produce(&host, &request, 40);
    // This co-anchor owns no publication. Its actual Delete authenticates it;
    // another publication's source reachability cannot mask the header check.
    let retired = request
        .advanced()
        .mutate(Retire {
            relation_fault: 0,
            corrupt_restoration: false,
            scope: 1,
            id: 10,
            co_anchors: vec![20],
            descendants: vec![101, 102],
        })
        .without_source()
        .idempotency(&"retire-for-corruption".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            retired,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{retired:?}"
    );
    let checkpoint = host
        .with_host_advancement(|phase| {
            host.capture_application_checkpoint(
                &phase,
                WorthQueryCheckpointCapturePolicy::SystemAllocation,
            )
            .unwrap()
        })
        .unwrap();
    let tampered = co_checkpoint_tamper::tamper(&checkpoint);
    let clean = reopen(checkpoint);
    assert!(
        retire_sentinel(&clean).is_ok(),
        "the untampered persisted history admits the same real Query candidate"
    );
    let recovered = reopen(tampered);
    let error = retire_sentinel(&recovered)
        .expect_err("co-anchor with no Delete must refuse through the closure owner");
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(worth_relational::facade::publication::AuthoredRetirementRefusal::CorruptPublication)
    );
    assert_eq!(
        error.context.affected_records,
        vec![worth_relational::facade::transactions::RecordRef::Entity(
            subject
        )]
    );
    let scope = request_scope();
    let principal = authenticate_operator(recovered.installed_schema(), &scope);
    let request = recovered.runtime().request(&principal, &scope);
    for id in [40, 401, 402] {
        assert_eq!(
            request.query(query::Input { id }).execute().unwrap().rows()[0].id,
            id
        );
    }
}
fn reopen(
    checkpoint: WorthQueryApplicationCheckpoint,
) -> WorthQueryProgramApplicationRuntime<S, Program> {
    in_memory_program_from_checkpoint(
        ApplicationProgramAuthoring::<S, Program>::begin()
            .validated_program()
            .unwrap(),
        S::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("919b0039-0000-0000-0000-000000000002").unwrap(),
        host_limits(),
        checkpoint,
    )
    .expect("outer Query and Native indexed artifact digests are valid")
}
fn retire_sentinel(
    host: &WorthQueryProgramApplicationRuntime<S, Program>,
) -> Result<(), worth_relational::facade::publication::PublicationError> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let outcome = request
        .advanced()
        .mutate(Retire {
            relation_fault: 0,
            corrupt_restoration: false,
            scope: 1,
            id: 40,
            co_anchors: vec![],
            descendants: vec![401, 402],
        })
        .without_source()
        .idempotency(&"retire-sentinel".to_owned())
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    match outcome {
        WorthQueryApplicationMutationOutcome::Committed { .. } => Ok(()),
        WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            denial,
        )) => Err(denial
            .authored_retirement_error()
            .expect("original typed Native refusal")
            .clone()),
        _ => panic!("{outcome:?}"),
    }
}
