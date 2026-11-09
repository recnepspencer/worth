//! A recorded child's ownership mismatch names that child's source, not its ancestor.
use super::*;

#[test]
fn authored_cascade_corruption_names_the_mismatching_publications_source() {
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationToken::new(),
        None,
    );
    let execution = &ExecutionRequest::serial(&request);
    let mut runtime = registration::runtime();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(RelationalSchemaRegistry::new())
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let mismatching = installed
        .clone()
        .with_source_scope_owned_producers([("owned-parent".into(), KindId(1))]);
    let installed = installed.with_source_scope_owned_producers([
        ("owned-parent".into(), KindId(1)),
        ("owned-child".into(), KindId(1)),
    ]);
    let anchor_target = targets(&["authored-anchor"]);
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(
            creates("author", &anchor_target),
            execution.allocation_policy(),
        )
        .unwrap();
    let born = transaction.commit(execution, &runtime).unwrap();
    let anchor = born.created_entity(&anchor_target[0]).unwrap();
    release_test_commit_snapshot(&runtime, &born);
    let birth = |producer: &str, source, key: &str| {
        let target = targets(&[key]);
        let result = publish(
            &runtime,
            execution,
            &installed,
            creates(producer, &target),
            RequiredOutputPublication::Birth {
                admission: owner.admit_required_output(producer, [1; 32]),
                absence: None,
                meaning: RequiredOutputPublicationMeaning {
                    implementation: RequiredOutputProducingImplementation::from_checkpoint(
                        [0x11; 16], [0x22; 32],
                    )
                    .unwrap(),
                    family: producer.into(),
                    source_scope: source,
                    source_partition: worth_foundational::CanonicalDigestId::new([0; 32]),
                    source_epoch: worth_foundational::CanonicalDigestId::new([1; 32]),
                },
                roles: vec![RequiredOutputRoleTarget {
                    role: "output".into(),
                    entity_name: "node".into(),
                    kind: KindId(1),
                    posture: RequiredOutputRolePosture::Create,
                    target: EntityReference::Created(target[0].clone()),
                }],
            },
        );
        let id = result.created_entity(&target[0]).unwrap();
        release_test_commit_snapshot(&runtime, &result);
        id
    };
    let child_source = birth("owned-parent", anchor, "generated-source");
    let realization = birth("owned-child", child_source, "child-realization");
    assert_ne!(anchor, child_source);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let roles = [anchor, child_source, realization]
        .into_iter()
        .enumerate()
        .map(|(index, id)| RequiredOutputRoleTarget {
            role: format!("retire-{index}"),
            entity_name: "node".into(),
            kind: KindId(1),
            posture: RequiredOutputRolePosture::Retire,
            target: EntityReference::Existing(id),
        })
        .collect::<Vec<_>>();
    assert!(runtime
        .readmit_authored_source_retirement(execution, &installed, &basis, anchor, &roles)
        .unwrap()
        .is_some());
    // Both handles carry genuine installation authority. The second omits the
    // child's declaration, so the real recorded publication must fail closed.
    let error = runtime
        .readmit_authored_source_retirement(execution, &mismatching, &basis, anchor, &roles)
        .unwrap_err();
    let TransactionCommitError::Publication { error, .. } = error else {
        panic!("{error:?}")
    };
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(worth_relational::facade::publication::AuthoredRetirementRefusal::CorruptPublication)
    );
    assert_eq!(
        error.context.affected_records,
        [RecordRef::Entity(child_source)]
    );
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        basis.descriptor()
    );
}
