//! The complete external-members checkpoint, including its outer manifest bytes, keeps custody.
use super::{fixture::Grid, original, recovery::publish_grid, transaction::fixture};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::{AspectKey, FieldKey};
use worth_relational::facade::{
    durability::*,
    indexes::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind},
    runtime::RelationalRuntime,
    transactions::{planned_single_field_locator, RelationalExecutionDenialCause},
};

fn populated(samples: u32) -> (RelationalRuntime, usize) {
    let cancel = CancellationSource::new();
    let request = original(&cancel);
    let execution = ExecutionRequest::serial(&request);
    let (runtime, contract) = fixture();
    publish_grid(
        &runtime,
        &execution,
        &contract,
        &Grid {
            rows: 1,
            columns: samples,
            samples: vec![0; samples as usize],
        },
    );
    (runtime, 43 + samples as usize * 8)
}

/// Register `count` index definitions so the outer manifest wire frame itself
/// exceeds transient 64 KiB region-capture buffers without inflating region rows.
fn register_manifest_indexes(runtime: &RelationalRuntime, count: usize) {
    let locator = planned_single_field_locator(
        AspectKey::new("samples").unwrap(),
        FieldKey::new("left").unwrap(),
    );
    let padding = "x".repeat(256);
    for ordinal in 0..count {
        runtime
            .index_authority()
            .register(DerivedIndexDefinition {
                index_id: DerivedIndexId(0),
                name: format!("manifest-index-{ordinal:04}-{padding}"),
                kind: DerivedIndexKind::EntityField {
                    field_locator: locator.clone(),
                },
                branch_scoped: false,
                unique_entity_kind: None,
            })
            .unwrap();
    }
}

#[test]
fn content_whole_checkpoint_original_custody_survives_clones_until_last_release() {
    let (runtime, frame_length) = populated(65_536);
    let cancel = CancellationSource::new();
    let budget = SerialMemoryBudget::new(Some(4 * 1024 * 1024));
    let request = SerialRequest::from_memory(budget.clone(), cancel.token(), None);
    let execution = ExecutionRequest::serial(&request);
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            &execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let retained = budget.charged_memory_bytes();
    assert!(
        retained >= (checkpoint.bytes().len() + frame_length) as u64,
        "outer checkpoint bytes and member bytes each own their actual grant"
    );
    let alias = checkpoint.clone();
    drop(checkpoint);
    assert_eq!(budget.charged_memory_bytes(), retained);
    drop(alias);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn content_whole_checkpoint_peak_budget_refuses_before_allocating_the_outer_frame() {
    let (runtime, _) = populated(65_536);
    register_manifest_indexes(&runtime, 600);
    let cancel = CancellationSource::new();
    let probe_budget = SerialMemoryBudget::new(None);
    let probe = SerialRequest::from_memory(probe_budget.clone(), cancel.token(), None);
    let probe = ExecutionRequest::serial(&probe);
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            &probe,
            probe.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let outer = checkpoint.bytes().len() as u64;
    assert!(
        outer > 2 * 65_536,
        "outer manifest ({outer}) must exceed 2x transient 64 KiB capture buffers so the peak ceiling refuses at outer allocation"
    );
    let retained = probe_budget.charged_memory_bytes();
    drop(checkpoint);
    let budget = SerialMemoryBudget::new(Some(retained - 1));
    let request = SerialRequest::from_memory(budget.clone(), cancel.token(), None);
    let execution = ExecutionRequest::serial(&request);
    let before = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .1;
    let refusal = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            &execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap_err();
    assert!(
        matches!(
            refusal,
            RelationalNativeCheckpointCaptureDenial::Request(
                RelationalExecutionDenialCause::PolicyMemoryExhausted {
                    requested,
                    admitted,
                    ..
                }
            ) if requested == outer && admitted < outer
        ),
        "expected exact outer manifest allocation refusal (outer={outer}), got {refusal:?}"
    );
    assert_eq!(budget.charged_memory_bytes(), 0);
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        before.descriptor()
    );
}

#[test]
fn content_whole_checkpoint_cancellation_after_outer_allocation_releases_all_capture_bytes() {
    let (runtime, _) = populated(65_536);
    register_manifest_indexes(&runtime, 2_000);
    let cancel = CancellationSource::new();
    let probe_budget = SerialMemoryBudget::new(None);
    let probe = SerialRequest::from_memory(probe_budget.clone(), cancel.token(), None);
    let probe = ExecutionRequest::serial(&probe);
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            &probe,
            probe.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let outer = checkpoint.bytes().len() as u64;
    assert!(
        outer > 2 * 65_536,
        "outer manifest ({outer}) must exceed 2x transient 64 KiB capture buffers so the threshold can only fire after outer allocation"
    );
    let retained = probe_budget.charged_memory_bytes();
    let outer_allocated_threshold = retained - outer / 2;
    drop(checkpoint);
    let budget = SerialMemoryBudget::new(Some(64 * 1024 * 1024));
    let request = SerialRequest::from_memory(budget.clone(), cancel.token(), None);
    let execution = ExecutionRequest::serial(&request);
    // Prior to outer manifest allocation, live charged bytes never exceed
    // (retained - outer) + 65_536 < retained - outer / 2. Skip past the
    // reservation-time consult (1) and prefix emission consult (2) so
    // cancellation triggers strictly during outer manifest emission, after
    // `ExecutionByteBuffer::allocate_in_stage` has allocated the outer backing.
    const CANCEL_DURING_OUTER_EMISSION_CONSULT: usize = 5;
    let (result, post_threshold_consults) =
        crate::cost::relational::checkpoint_stage::cancel_on_consult_at_memory_threshold(
            budget.clone(),
            outer_allocated_threshold,
            CANCEL_DURING_OUTER_EMISSION_CONSULT,
            cancel,
            || {
                runtime
                    .durability_authority()
                    .native_checkpoint_with_policy(
                        &execution,
                        execution.allocation_policy(),
                        RelationalCheckpointArtifactPolicy::ExternalMembers {
                            current_members: &[],
                            source: None,
                        },
                    )
            },
        );
    assert!(
        post_threshold_consults >= CANCEL_DURING_OUTER_EMISSION_CONSULT,
        "expected cancellation during outer manifest emission (consult >= {CANCEL_DURING_OUTER_EMISSION_CONSULT}), observed {post_threshold_consults}"
    );
    let refusal = result.unwrap_err();
    assert!(
        matches!(
            refusal,
            RelationalNativeCheckpointCaptureDenial::Request(
                RelationalExecutionDenialCause::Cancelled
            )
        ),
        "{refusal:?}"
    );
    assert_eq!(budget.charged_memory_bytes(), 0);
}
