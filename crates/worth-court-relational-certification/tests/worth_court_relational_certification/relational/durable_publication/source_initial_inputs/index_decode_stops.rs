//! The invoking request stops at the selected original index DATA decoder.
use super::selected_source::{MemberStop, SelectedSource};
use super::*;
use std::sync::{atomic::Ordering, Mutex};
use worth_execution::{
    CancellationSource, ExecutionRequestArrayBuilder, SerialMemoryBudget, SerialRequest,
};
use worth_relational::facade::{indexes::*, transactions::RelationalExecutionDenialCause};

#[test]
fn original_index_decode_refuses_cancelled_and_exhausted_invoking_requests_without_retaining_data()
{
    let policy = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let setup = ExecutionRequest::serial(&policy);
    let (runtime, source, member) = recovered_indexes(&setup);
    drop(capture(&runtime, &setup));

    let cancellation = CancellationSource::new();
    let budget = SerialMemoryBudget::new(None);
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    source.arm(member, MemberStop::Cancel(cancellation));
    assert!(matches!(
        runtime.complete_checkpoint_service().prepare(
            &ExecutionRequest::serial(&host),
            RelationalCheckpointArtifactPolicy::Embedded
        ),
        Err(RelationalNativeCheckpointCaptureDenial::Request(
            RelationalExecutionDenialCause::Cancelled
        ))
    ));
    assert!(
        source.fired.load(Ordering::Relaxed),
        "the original index member was read before decoder cancellation"
    );
    assert_eq!(budget.charged_memory_bytes(), 0);

    let limit = 64 * 1024 * 1024;
    let budget = SerialMemoryBudget::new(Some(limit));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let held = Arc::new(Mutex::new(None));
    source.arm(member, MemberStop::Exhaust { held: held.clone() });
    assert!(matches!(
        runtime.complete_checkpoint_service().prepare(
            &ExecutionRequest::serial(&host),
            RelationalCheckpointArtifactPolicy::Embedded
        ),
        Err(RelationalNativeCheckpointCaptureDenial::Request(
            RelationalExecutionDenialCause::PolicyMemoryExhausted { .. }
        ))
    ));
    assert!(
        source.fired.load(Ordering::Relaxed),
        "the original index member was read before decoder exhaustion"
    );
    // Release only the deliberate blocker. No failed invocation DATA remains.
    drop(
        held.lock()
            .unwrap()
            .take()
            .expect("the invoking original request funded the blocker"),
    );
    assert_eq!(budget.charged_memory_bytes(), 0);
    let success = capture(&runtime, &setup);
    assert_eq!(success.source_artifact().covered().0, 1);
}

fn recovered_indexes(
    request: &ExecutionRequest<'_, '_>,
) -> (
    RelationalRuntime,
    Arc<SelectedSource>,
    RelationalDurableMemberLocator,
) {
    let (runtime, medium) = initialized(request);
    let definition = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "original-decoder-stop".into(),
            branch_scoped: true,
            kind: DerivedIndexKind::EntityField {
                field_locator: worth_relational::facade::transactions::planned_single_field_locator(
                    worth_foundational::facade::AspectKey::new("name").unwrap(),
                    worth_foundational::facade::FieldKey::new("name").unwrap(),
                ),
            },
        })
        .unwrap();
    let commit = create_entity_outcome(request, &runtime, "decoder-original");
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let built = runtime.index_authority().build_for_basis(
        request,
        DerivedIndexBuildRequest {
            source_commit_id: commit.commit.commit_id,
            branch_id: commit.commit.branch_id.clone(),
            index_ids: vec![definition.index_id],
        },
        &basis,
    );
    assert!(built.failed_indexes.is_empty(), "{built:?}");
    assert!(!built.generations.is_empty());
    drop(basis);
    release_test_commit_snapshot(&runtime, &commit);
    let image = capture_external(&runtime, &medium, request);
    medium.file_checkpoint(image.checkpoint().clone());
    let payload = image
        .checkpoint()
        .bytes()
        .strip_prefix(b"WORTH-REL-CHK18\n")
        .unwrap();
    let member = rmp_serde::from_slice(crate::messagepack_value_spans::field(
        crate::messagepack_value_spans::field(payload, "manifest"),
        "derived_indexes",
    ))
    .unwrap();
    let observed = SelectedSource::new(medium.clone());
    let source: Arc<dyn RelationalDurableColdSource> = observed.clone();
    let cut = RelationalDurableReadProtection::capture(&source, request).unwrap();
    let initial = cut
        .read_initial_inputs_owned(&source, request, 1 << 30)
        .unwrap();
    let input = RelationalDurableRecoverySource::from_home_original_bytes(
        Some(home_image_checkpoint(&image, image.source_artifact().clone(), request).unwrap()),
        initial,
        ExecutionRequestArrayBuilder::allocate(*request, 0)
            .unwrap()
            .seal()
            .unwrap(),
        ExecutionRequestArrayBuilder::allocate(*request, 0)
            .unwrap()
            .seal()
            .unwrap(),
        cut.bounds(),
        source,
        None,
        request,
    )
    .unwrap();
    let mut recovered = receiver(&medium);
    let plan = recovered
        .durability()
        .recovery_plan_from_source(
            request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    recovered
        .durability_recovery()
        .recover_with_authority(request, plan)
        .unwrap();
    (recovered, observed, member)
}
