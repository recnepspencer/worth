//! Same acknowledged successor and funded historical roster survive capture retry.
use super::*;
#[test]
fn companion_post_ack_capture_refusal_repairs_same_capsule_with_historical_roster() {
    let (original, settlement) = original(model::authored_value());
    let ids = companion_ids(&settlement);
    let source_id = old::reads::source_id(&original);
    let checkpoint = old::host::capture(&original);
    let held = checkpoint.clone();
    let bytes = checkpoint.bytes().to_vec();
    successor::output::reset_decodes();
    let mut callbacks = 0;
    let pending = crate::checkpoint_capture::stopped(|phase| {
        callbacks = 0;
        installation::in_memory_rostered_program_from_checkpoint_with_transition_in_advancement(
            phase,
            new::host::program::<new::program::TargetProgram>(),
            installation::WorthQueryApplicationProgramRoster::new(),
            successor::schema::CanonicalSchema::declaration().unwrap(),
            ((),),
            ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
            successor::host::limits(),
            checkpoint.clone(),
            installation::WorthQueryCheckpointProgramPredecessor::new(
                &old::host::program::<old::program::SourceProgram>()
                    .revision()
                    .to_string(),
            )
            .unwrap(),
            installation::WorthQueryCheckpointTransitionResources::bounded(65_536, 32, 65_536)
                .unwrap()
                .with_capture_memory_limit(0),
            |_, _| {
                callbacks += 1;
                Ok(())
            },
        )
    });
    let Some(installation::WorthQueryCheckpointCaptureDenial::Allocation(cause)) =
        pending.capture_denial()
    else {
        panic!("exact native allocation cause retained")
    };
    let requested = cause.requested_payload_bytes().unwrap();
    assert!(requested > 0);
    assert_eq!(
        cause.kind(),
        worth_query_host::facade::runtime::ExecutionAllocationDenialKind::Lease(
            worth_execution::LeaseDenial::MemoryExhausted(
                worth_execution::MemoryLimitDenial::Capacity {
                    requested,
                    admitted: 0,
                    level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 }
                }
            )
        )
    );
    assert_eq!(callbacks, 1);
    assert_eq!(successor::output::decodes(), 0);
    // An unfunded original request returns the same acknowledged capsule again.
    let zero = crate::checkpoint_capture::lease(0);
    let pending = pending
        .repair_to_checkpoint_with_request(worth_execution::ExecutionRequest::leased(&zero))
        .unwrap_err();
    assert!(pending.capture_denial().is_some());
    let funded = crate::checkpoint_capture::lease(64 * 1024 * 1024);
    let repaired = pending
        .repair_to_checkpoint_with_request(worth_execution::ExecutionRequest::leased(&funded))
        .unwrap();
    assert_eq!(callbacks, 1, "same-capsule repair never reauthors");
    assert_eq!(held.bytes(), bytes);
    let target = new::host::restore::<new::program::TargetProgram>(
        repaired,
        installation::WorthQueryApplicationProgramRoster::new(),
    );
    assert_eq!(new::reads::source_id(&target), source_id);
    assert_eq!(successor::output::decodes(), 0);
    assert_target(&target, ids);
}
