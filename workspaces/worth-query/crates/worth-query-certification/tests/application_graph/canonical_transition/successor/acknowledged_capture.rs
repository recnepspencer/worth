//! A real acknowledged codec successor survives exact capture failure and repair.
use super::*;
use std::num::NonZeroUsize;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
use worth_query_host::facade::runtime::{
    ExecutionAllocationDenialKind, LeaseDenial, MemoryLimitDenial, MemoryLimitLevel,
};

#[test]
fn acknowledged_codec_successor_keeps_exact_capture_cause_and_repairs_without_reauthoring() {
    let authored = model::authored_value();
    let checkpoint = predecessor_checkpoint();
    // Source identity is installation-specific; observe this exact checkpoint's
    // admitted authored record rather than borrowing another world's identity.
    let restored_source = super::super::host::restore(checkpoint.clone());
    let source_id = super::super::host::source_id(&restored_source);
    assert_eq!(super::super::host::read(&restored_source), authored);
    drop(restored_source);
    let original_bytes = checkpoint.bytes().to_vec();
    let retained = checkpoint.clone();
    let predecessor = installation::WorthQueryCheckpointProgramPredecessor::new(
        &super::super::host::program().revision().to_string(),
    )
    .unwrap();
    let mut callbacks = 0;
    output::reset_decodes();
    let pending = crate::checkpoint_capture::stopped(|phase| {
        callbacks = 0;
        installation::in_memory_rostered_program_from_checkpoint_with_transition_in_advancement(
            phase,
            host::program(),
            installation::WorthQueryApplicationProgramRoster::new(),
            schema::CanonicalSchema::declaration().unwrap(),
            ((),),
            ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
            host::limits(),
            checkpoint.clone(),
            predecessor.clone(),
            installation::WorthQueryCheckpointTransitionResources::bounded(4096, 32, 16384)
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
        panic!("exact native capture allocation refusal is retained: {pending:?}");
    };
    let requested = cause
        .requested_payload_bytes()
        .expect("native capture supplies its exact payload quote");
    assert!(requested > 0);
    assert_eq!(
        cause.kind(),
        ExecutionAllocationDenialKind::Lease(LeaseDenial::MemoryExhausted(
            MemoryLimitDenial::Capacity {
                requested,
                admitted: 0,
                level: MemoryLimitLevel::Policy { ancestor: 0 },
            }
        ))
    );
    assert_eq!(
        callbacks, 1,
        "capture refusal follows real transition authoring and acknowledgment"
    );
    assert_eq!(output::decodes(), 0);
    let repaired = worth_query_host::facade::application_contribution::with_bootstrap_advancement(
        ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), Some(64 * 1024 * 1024), None),
        ),
        |phase| {
            pending
                .repair_to_checkpoint(&phase, CapturePolicy::SystemAllocation)
                .expect("a fresh funded capture keeps the same acknowledged successor")
        },
    )
    .expect("the fresh bootstrap repair request is admitted");
    assert_eq!(callbacks, 1, "repair must never rerun authoring");
    assert_eq!(retained.bytes(), original_bytes);
    let reopened = host::restore(repaired);
    assert_eq!(host::source_id(&reopened), source_id);
    assert_eq!(host::read(&reopened), authored);
    let scope = crate::document_retention_model::operator_identity::request_scope();
    let principal = crate::document_retention_model::operator_identity::authenticate_operator(
        reopened.installed_schema(),
        &scope,
    );
    let request = reopened
        .runtime()
        .request(&principal, &scope)
        .on_branch(reopened.runtime().current_world());
    let mut regenerated = request
        .demand(producer::Demand)
        .start_in_program::<_, program::Root>(&reopened)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
        regenerated.settle(&request).unwrap()
    else {
        panic!("the repaired V2 successor must reconstruct its native derived output");
    };
    assert_eq!(settled.producer_contacts_in_this_demand(), 1);
    let value = host::read_derived(&reopened);
    assert_eq!(value.0.caption, "ASSEMBLY Β");
    assert_eq!(value.0.total_rank, 12);
    assert_eq!(value.0.labels, ["left", "right"]);
}
