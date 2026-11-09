//! Real Query replay refuses two writers claiming the same recorded edge.
use super::super::native_checkpoint_tamper::{field, items, optional_field};
use super::super::{lifecycle::*, *};
use super::multi::TwoProgram;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
type Host<S> = WorthQueryProgramApplicationRuntime<S, TwoProgram>;
pub(super) fn require_unique_edge_ownership<S: Scenario>(
    host: &Host<S>,
    retirement: &WorthQueryApplicationCommitReceipt,
    source: worth_relational::facade::identity::EntityId,
) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    super::super::correction_courts::reverse(
        &request,
        retirement,
        "restore-before-cold-edge-check",
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
    let tampered = super::super::native_checkpoint_tamper::mutate(&checkpoint, |envelope| {
        let group = optional_field(envelope, "coupled_authored_publication")?;
        let participants = field(&envelope[group.clone()], "participants");
        let participants_start = group.start + participants.start;
        let parts = items(&envelope[participants_start..group.start + participants.end]);
        assert_eq!(parts.len(), 2);
        let edge = |part: &std::ops::Range<usize>| {
            let start = participants_start + part.start;
            let origin = field(&envelope[start..participants_start + part.end], "origin");
            let origin_start = start + origin.start;
            let relations = field(&envelope[origin_start..start + origin.end], "relations");
            let relations_start = origin_start + relations.start;
            let edges = items(&envelope[relations_start..origin_start + relations.end]);
            assert_eq!(edges.len(), 1, "each writer actually created one edge");
            relations_start + edges[0].start..relations_start + edges[0].end
        };
        let first = edge(&parts[0]);
        let second = edge(&parts[1]);
        Some((first, envelope[second].to_vec()))
    });
    let clean = reopen::<S>(checkpoint);
    assert!(
        retire_sentinel(&clean).is_ok(),
        "the authentic grouped history admits the same Query candidate"
    );
    let corrupted = reopen::<S>(tampered);
    let scope = request_scope();
    let principal = authenticate_operator(corrupted.installed_schema(), &scope);
    let request = corrupted.runtime().request(&principal, &scope);
    let outcome = request
        .advanced()
        .mutate(Retire {
            relation_fault: 0,
            corrupt_restoration: false,
            scope: 1,
            id: 20,
            co_anchors: vec![],
            descendants: vec![],
        })
        .without_source()
        .idempotency(&"corrupt-edge-birth".to_owned())
        .execute_in_program(&corrupted, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("historical duplicate edge custody must refuse: {outcome:?}")
    };
    let failure = denial
        .authored_retirement_error()
        .expect("typed historical birth refusal");
    assert_eq!(
        failure.authored_retirement_refusal(),
        Some(worth_relational::facade::publication::AuthoredRetirementRefusal::CorruptPublication)
    );
    assert_eq!(
        failure.context.affected_records,
        [worth_relational::facade::transactions::RecordRef::Entity(
            source
        )]
    );
    assert!(
        request.query(query::Input { id: 40 }).execute().is_ok(),
        "refused candidate had no effect"
    );
}
pub(in super::super) fn reopen<S: Scenario>(
    checkpoint: WorthQueryApplicationCheckpoint,
) -> Host<S> {
    try_reopen(checkpoint).expect("both indexed digests are valid")
}
pub(in super::super) fn try_reopen<S: Scenario>(
    checkpoint: WorthQueryApplicationCheckpoint,
) -> Result<Host<S>, WorthQueryApplicationPreparationDenial> {
    let schema = S::declaration().unwrap();
    WorthQueryInterimCheckpointRestore::program(
        ApplicationProgramAuthoring::<S, TwoProgram>::begin()
            .with_schema(&schema)
            .validated_program()
            .unwrap(),
        schema,
        ((),),
        ApplicationBuildIdentity::parse("919b0039-0000-0000-0000-000000000002").unwrap(),
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        checkpoint,
    )
}
pub(super) fn retire_sentinel<S: Scenario>(
    host: &Host<S>,
) -> Result<(), worth_relational::facade::publication::PublicationError> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let outcome = request
        .advanced()
        .mutate(Retire {
            relation_fault: 0,
            corrupt_restoration: false,
            co_anchors: vec![],
            scope: 1,
            id: 40,
            descendants: vec![],
        })
        .without_source()
        .idempotency(&"history-sentinel".to_owned())
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    match outcome {
        WorthQueryApplicationMutationOutcome::Committed { .. } => Ok(()),
        WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            denial,
        )) => Err(denial
            .authored_retirement_error()
            .expect("typed Native history refusal")
            .clone()),
        _ => panic!("sentinel must commit or refuse with the original Native cause"),
    }
}
