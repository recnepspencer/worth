//! The second Complete carries actual covered and replayed-tail index owners.
use super::*;
use worth_relational::facade::indexes::*;
#[test]
fn cold_complete_recapture_preserves_original_index_generations_and_live_tail() {
    let policy = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&policy);
    let (runtime, medium) = initialized(&request);
    let definition = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "cold-original-name".into(),
            branch_scoped: true,
            kind: DerivedIndexKind::EntityField {
                field_locator: worth_relational::facade::transactions::planned_single_field_locator(
                    worth_foundational::facade::AspectKey::new("name").unwrap(),
                    worth_foundational::facade::FieldKey::new("name").unwrap(),
                ),
            },
        })
        .unwrap();
    let first = create_entity_outcome(&request, &runtime, "covered-index-value");
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let built = runtime.index_authority().build_for_basis(
        &request,
        DerivedIndexBuildRequest {
            source_commit_id: first.commit.commit_id,
            branch_id: first.commit.branch_id.clone(),
            index_ids: vec![definition.index_id],
        },
        &basis,
    );
    assert!(built.failed_indexes.is_empty(), "{built:?}");
    assert!(!built.generations.is_empty());
    drop(basis);
    release_test_commit_snapshot(&runtime, &first);
    let image = capture_external(&runtime, &medium, &request);
    let original = generations(&image);
    assert!(!original.is_empty());
    medium.file_checkpoint(image.checkpoint().clone());
    let later = create_entity_outcome(&request, &runtime, "live-tail-index-value");
    let tail_commit = later.commit.commit_id.0;
    release_test_commit_snapshot(&runtime, &later);
    let input = home_input(&medium, image.source_artifact().clone(), &request).unwrap();
    drop((image, runtime));
    let mut recovered = receiver(&medium);
    let plan = recovered
        .durability()
        .recovery_plan_from_source(
            &request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    recovered
        .durability_recovery()
        .recover_with_authority(&request, plan)
        .unwrap();
    let next = capture(&recovered, &request);
    let observed = generations(&next);
    for original in original {
        assert!(
            observed.contains(&original),
            "original covered index generation lost: {original:?}"
        );
    }
    assert!(observed
        .iter()
        .any(
            |generation| rmp_serde::from_slice::<u64>(crate::messagepack_value_spans::field(
                generation,
                "source_commit_id"
            ))
            .unwrap()
                == tail_commit
        ));
    assert_eq!(next.source_artifact().covered().0, 2);
}
fn generations(image: &CompleteRelationalNativeCheckpoint) -> Vec<Vec<u8>> {
    // Published member framing is independent of the producer's private constants.
    let prefix = b"WORTH-REL-MBR1\n\0\x01\x02";
    let payload = image
        .checkpoint()
        .members()
        .iter()
        .find_map(|member| match member {
            RelationalCheckpointMember::New { bytes, .. } => bytes.bytes().strip_prefix(prefix),
            RelationalCheckpointMember::Carried { .. } => None,
        })
        .expect("the actual Complete contains its derived-index member");
    let state = crate::messagepack_value_spans::sequence(payload)[0];
    let generations = crate::messagepack_value_spans::field(state, "generations");
    crate::messagepack_value_spans::sequence(generations)
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect()
}
