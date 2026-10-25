//! Real native performance and linear repair custody under caller byte policy.
use super::*;
use application_installation::WorthQueryCheckpointCaptureDenial as CaptureDenial;
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
use worth_query_host::facade::runtime::{
    ExecutionAllocationDenialKind as AllocationKind, LeaseDenial,
};

pub(super) fn author_record(
    writer: &mut application_installation::WorthQueryCheckpointMigrationWriter<
        '_,
        TemporalHostSchema,
    >,
    key: &str,
    value: u64,
) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    writer.bind_entity(
        primary_graph::WorthQueryApplicationEntitySeed::new(
            UnrelatedRecord::reference(),
            primary_graph::WorthQueryApplicationEntityKey::new(key).unwrap(),
        )
        .field(UnrelatedValueField::reference(), value),
    )
}

pub(super) fn assert_target_record(
    checkpoint: application_installation::WorthQueryApplicationCheckpoint,
    value: u64,
) {
    let reopened = application_installation::in_memory_program_from_checkpoint(
        validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        configuration(),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        checkpoint::checkpoint_limits(),
        checkpoint,
    )
    .expect("ordinary target admission authenticates the repaired native successor");
    let selected = reopened
        .with_host_advancement(|host_phase| {
            reopened
                .on_branch(reopened.current_world())
                .select(&host_phase)
                .unwrap()
        })
        .expect("installed host call opens");
    let record = reopened
        .with_host_advancement(|host_phase| {
            selected
                .resolve_entity(
                    &host_phase,
                    UnrelatedValueField::reference(),
                    value,
                    &request_scope(),
                    primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .expect("the actual typed migrated value survives native recovery")
        })
        .expect("installed host call opens");
    assert_eq!(record.examined_candidate_count(), 1);
}

#[test]
fn checkpoint_transition_admission_refusal_retains_acknowledged_effects_for_explicit_repair() {
    let (source, predecessor) = source();
    let mut calls = 0;
    let pending = crate::checkpoint_capture::stopped(|phase| {
        calls = 0;
        application_installation::in_memory_rostered_program_from_checkpoint_with_transition_in_advancement(
            phase,
            validated_program(),
            application_installation::WorthQueryApplicationProgramRoster::new(),
            TemporalHostSchema::declaration().unwrap(),
            configuration(),
            ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
            checkpoint::checkpoint_limits(),
            source.clone(),
            predecessor.clone(),
            Resources::bounded(512, 32, 4096).unwrap().with_capture_memory_limit(0),
            |writer, _| {
                calls += 1;
                author_record(writer, "allocation-repaired-record", 127)
            },
        )
    });
    let Some(CaptureDenial::Allocation(denied)) = pending.capture_denial() else {
        panic!("exact native allocation cause must survive: {pending:?}");
    };
    assert_eq!(
        denied.kind(),
        AllocationKind::Lease(LeaseDenial::MemoryExhausted(
            worth_query_host::facade::runtime::MemoryLimitDenial::Capacity {
                requested: denied.requested_payload_bytes().unwrap(),
                admitted: 0,
                level: worth_query_host::facade::runtime::MemoryLimitLevel::Policy { ancestor: 0 }
            }
        ))
    );
    let native_quote = denied
        .requested_payload_bytes()
        .expect("checked real native payload quote");
    assert!(native_quote > 0);
    assert_eq!(
        calls, 1,
        "capture refusal follows actual authoring and acknowledgment"
    );
    let zero = crate::checkpoint_capture::lease(0);
    let pending = pending
        .repair_to_checkpoint_with_request(worth_execution::ExecutionRequest::leased(&zero))
        .unwrap_err();
    let Some(CaptureDenial::Execution(cause)) = pending.capture_denial() else {
        panic!("zero request must preserve the bootstrap admission refusal: {pending:?}");
    };
    let primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied {
        kind: primary_graph::WorthQueryProviderSessionDenialKind::ExecutionResource {
            denial: worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial::MemoryLimit { requested: admission_bytes, admitted: 0, .. }, ..
        }
    } = cause.kind() else { panic!("typed original request memory refusal: {cause:?}"); };
    assert!(admission_bytes > 0);
    let native_only = crate::checkpoint_capture::lease(admission_bytes + native_quote);
    let pending = pending
        .repair_to_checkpoint_with_request(worth_execution::ExecutionRequest::leased(&native_only))
        .expect_err("the final frame is admitted while native backing remains live");
    let Some(CaptureDenial::Allocation(denied)) = pending.capture_denial() else {
        panic!("exact final-frame allocation cause must survive: {pending:?}");
    };
    assert_eq!(
        denied.kind(),
        AllocationKind::Lease(LeaseDenial::MemoryExhausted(
            worth_query_host::facade::runtime::MemoryLimitDenial::Capacity {
                requested: denied.requested_payload_bytes().unwrap(),
                admitted: 0,
                level: worth_query_host::facade::runtime::MemoryLimitLevel::Policy { ancestor: 0 }
            }
        ))
    );
    let frame_quote = denied
        .requested_payload_bytes()
        .expect("checked real final-frame payload quote");
    assert!(frame_quote > native_quote);
    let released = native_only.reserve_memory(native_quote).unwrap();
    drop(released);
    drop(native_only);
    let repair = crate::checkpoint_capture::lease(
        admission_bytes + native_quote.checked_add(frame_quote).unwrap(),
    );
    let checkpoint = pending
        .repair_to_checkpoint_with_request(worth_execution::ExecutionRequest::leased(&repair))
        .expect("a fresh funded request captures the same acknowledged successor");
    assert_eq!(checkpoint.bytes().len() as u64, frame_quote);
    assert_eq!(checkpoint.charged_payload_bytes(), Some(frame_quote));
    assert_target_record(checkpoint, 127);
    assert_eq!(calls, 1, "repair cannot rerun authoring");
}
