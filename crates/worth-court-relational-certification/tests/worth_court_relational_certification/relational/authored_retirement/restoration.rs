//! Restore the exact recorded derived payload and edges under fresh generations.
use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn check(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    owner: &worth_proof::InstalledApplicationPublicationOwner,
    retired: &CommitResult,
    source: EntityId,
    expected: &[EntityId],
    depth: usize,
    cold: bool,
) {
    let saved = runtime
        .durability_authority()
        .native_checkpoint(execution, execution.allocation_policy())
        .unwrap();
    let mut reopened = AspectSchemaFixture::with_default_declared_aspects(
        worth_relational::facade::config::CascadeDeletePolicy::RetainDanglingForAudit,
    )
    .build_runtime();
    let receipt = reopened
        .prepare_initial_schema_installation()
        .unwrap()
        .install(RelationalSchemaRegistry::new())
        .unwrap();
    reopened
        .durability_recovery()
        .restore_native_checkpoint(execution, &saved)
        .unwrap();
    let receipt = reopened
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    let (cold_owner, cold_installed) = receipt.issue_application_publication_owner();
    let cold_installed = cold_installed.with_source_scope_owned_producers(
        [
            "parent-realizer",
            "child-realizer",
            "leaf-realizer",
            "other-owner",
        ]
        .map(|name| (name.to_owned(), KindId(1))),
    );
    let (runtime, installed, owner) = if cold {
        (&reopened, &cold_installed, &cold_owner)
    } else {
        (runtime, installed, owner)
    };
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let recreated = targets(&["restored-authored-source"]);
    let plan = runtime
        .prepare_authored_source_restoration(
            execution,
            installed,
            &basis,
            retired.commit.commit_id,
            source,
            recreated[0].clone(),
        )
        .unwrap()
        .unwrap();
    let admission = owner
        .admit_authored_output_retirements(plan.authorities())
        .unwrap();
    let authored = WorkerIntentBatch::new("authored-restore").push(MutationIntent::Create(
        CreateIntent::Entity(EntitySpec {
            partition_id: recreated[0].partition_id,
            kind_id: recreated[0].kind_id,
            client_key: recreated[0].client_key.clone(),
            fields: fields("authored-source"),
        }),
    ));
    let batch = plan.extend_batch(execution, authored).unwrap();
    let restored = publish(
        runtime,
        execution,
        installed,
        batch,
        RequiredOutputPublication::AuthoredCascadeRestoration {
            admission,
            restoration: plan,
        },
    );
    let fresh_source = restored.created_entity(&recreated[0]).unwrap();
    assert_ne!(
        fresh_source, source,
        "restoration never revives the original generation"
    );
    assert_eq!(
        restored
            .changed_records
            .iter()
            .filter(|record| matches!(record, RecordRef::Entity(_)))
            .count(),
        expected.len(),
        "the anchor and all owned members are restored by the same candidate"
    );
    let pinned = runtime
        .read_truth()
        .pin_read_capability(&restored.snapshot, &[], execution)
        .unwrap();
    release_test_commit_snapshot(runtime, &restored);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    assert!(
        runtime
            .prepare_authored_source_restoration(
                execution,
                installed,
                &basis,
                retired.commit.commit_id,
                source,
                targets(&["duplicate-restoration"])[0].clone()
            )
            .is_err(),
        "one retired cascade cannot be restored twice"
    );
    let view = runtime
        .read_truth()
        .project_observation(&basis.observation())
        .unwrap();
    let mut names = std::collections::BTreeMap::new();
    for record in &restored.changed_records {
        if let RecordRef::Entity(id) = record {
            let name = view
                .read_exact_entity_state(execution, *id, |_, state| {
                    text(state.unwrap(), "name", pinned.symbols())
                })
                .unwrap()
                .unwrap();
            assert!(names.insert(name, *id).is_none());
        }
    }
    let mut wanted = vec![
        "authored-source",
        "parent-realization",
        "child-source-or-cell",
    ];
    if depth == 3 {
        wanted.extend([
            "child-realization",
            "leaf-source",
            "leaf-realization",
            "leaf-cell",
        ]);
    }
    wanted.sort_unstable();
    assert_eq!(names.keys().map(String::as_str).collect::<Vec<_>>(), wanted);
    let mut edges = Vec::new();
    for record in &restored.changed_records {
        if let RecordRef::Relation(id) = record {
            let ends = view
                .read_exact_relation_state(execution, *id, |metadata, state| {
                    assert_eq!(
                        text(state.unwrap(), "label", pinned.symbols()),
                        "source association"
                    );
                    let name = |id| {
                        names
                            .iter()
                            .find(|(_, value)| **value == id)
                            .unwrap()
                            .0
                            .clone()
                    };
                    (name(metadata.source), name(metadata.target))
                })
                .unwrap()
                .unwrap();
            edges.push(ends);
        }
    }
    let mut wanted_edges = vec![
        ("parent-realization", "authored-source"),
        ("child-source-or-cell", "authored-source"),
    ];
    if depth == 3 {
        wanted_edges.extend([
            ("child-realization", "child-source-or-cell"),
            ("leaf-source", "child-source-or-cell"),
            ("leaf-realization", "leaf-source"),
            ("leaf-cell", "leaf-source"),
        ]);
    }
    wanted_edges.sort_unstable();
    edges.sort_unstable();
    assert_eq!(
        edges
            .iter()
            .map(|(a, b)| (a.as_str(), b.as_str()))
            .collect::<Vec<_>>(),
        wanted_edges
    );
    // A new demand can recover every distinct restored publication from actual Native history.
    let mut routes = vec![(
        "parent-realizer",
        "authored-source",
        ["parent-realization", "child-source-or-cell"],
    )];
    if depth == 3 {
        routes.extend([
            (
                "child-realizer",
                "child-source-or-cell",
                ["child-realization", "leaf-source"],
            ),
            (
                "leaf-realizer",
                "leaf-source",
                ["leaf-realization", "leaf-cell"],
            ),
        ]);
    }
    for (producer, source, members) in routes {
        let meaning = RequiredOutputPublicationMeaning {
            implementation: RequiredOutputProducingImplementation::from_checkpoint(
                [0x11; 16], [0x22; 32],
            )
            .unwrap(),
            family: producer.into(),
            source_scope: names[source],
            source_partition: [0; 32],
            source_epoch: [1; 32],
        };
        let roles = members
            .into_iter()
            .enumerate()
            .map(|(index, name)| RequiredOutputRoleTarget {
                role: format!("output-{index}"),
                entity_name: "node".into(),
                kind: KindId(1),
                posture: RequiredOutputRolePosture::Create,
                target: EntityReference::Existing(names[name]),
            })
            .collect::<Vec<_>>();
        runtime
            .readmit_required_output_publication(
                execution, installed, &basis, producer, &[1; 32], &meaning, &roles,
            )
            .unwrap();
    }
}

pub(super) fn text(
    state: &worth_foundational::facade::AuthoritativeRecordAspectState,
    aspect: &str,
    symbols: &worth_relational::facade::symbols::StringInterner,
) -> String {
    use worth_foundational::facade::{AspectValue, ContractValidatedAspectValueView as View};
    match state.get(&aspect_key(aspect)).unwrap().view() {
        View::Scalar(AspectValue::String(value)) => match value {
            worth_foundational::facade::InternedString::Raw(value) => value.clone(),
            worth_foundational::facade::InternedString::Symbol(value) => symbols
                .resolve(worth_relational::facade::symbols::Symbol(value.0))
                .unwrap()
                .to_owned(),
        },
        other => panic!("expected fixture scalar text: {other:?}"),
    }
}
