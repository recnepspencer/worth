use super::fixtures::*;
use crate::tests::support::*;

#[test]
fn committed_birth_anchor_removal_split_merge_and_fork_keep_one_identity_owner() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = runtime();
    let born = commit(execution, &runtime, "main", birth(&[1, 2, 3, 0, 4]));
    let members = members(&born, 5);
    let rows = records(execution, &runtime, &born.snapshot);
    assert_eq!(rows.len(), 2);
    let original = identity(&rows, members[0]);
    let unrelated = identity(&rows, members[4]);
    assert!(rows
        .iter()
        .all(|(_, row)| row.born == born.commit.commit_id));
    let branch = create_branch_from_main(&runtime, "aggregate-fork");
    let fork_snapshot = snapshot_for_owner_branch(&runtime, &branch);
    let split = commit(
        execution,
        &runtime,
        "main",
        rewire(
            execution,
            &runtime,
            &born.snapshot,
            &[(members[1], members[0]), (members[3], members[2])],
        ),
    );
    let split_rows = records(execution, &runtime, &split.snapshot);
    assert_eq!(identity(&split_rows, members[0]), original);
    let split_identity = identity(&split_rows, members[2]);
    assert_ne!(split_identity, original);
    assert_eq!(identity(&split_rows, members[4]), unrelated);
    assert_eq!(records(execution, &runtime, &fork_snapshot), rows);
    assert_eq!(records(execution, &runtime, &born.snapshot), rows);

    let mut removal = rewire(
        execution,
        &runtime,
        &split.snapshot,
        &[(members[1], members[1])],
    );
    // Author the complete business permutation explicitly; inherited premerge
    // cardinality admission does not expand implicit cascade deletions here.
    let selected = runtime
        .read_truth()
        .read_snapshot(&split.snapshot, execution)
        .unwrap()
        .unwrap();
    let incident = selected
        .relations()
        .iter()
        .find(|edge| edge.kind.kind_id == KindId(2) && edge.source == members[0])
        .unwrap()
        .relation_id;
    removal = removal.push(MutationIntent::Relation(RelationMutationIntent::Delete(
        DeleteRelationIntent {
            relation_id: incident,
        },
    )));
    removal = removal.push(MutationIntent::Entity(EntityMutationIntent::Delete(
        DeleteEntityIntent {
            entity_id: members[0],
        },
    )));
    let removed = commit(execution, &runtime, "main", removal);
    let removed_rows = records(execution, &runtime, &removed.snapshot);
    assert_eq!(identity(&removed_rows, members[1]), original);
    let original_row = &removed_rows
        .iter()
        .find(|(_, row)| row.identity == original)
        .unwrap()
        .1;
    assert_eq!(original_row.seed, members[0]);
    assert_eq!(original_row.born, born.commit.commit_id);
    let merged = commit(
        execution,
        &runtime,
        "main",
        rewire(
            execution,
            &runtime,
            &removed.snapshot,
            &[(members[1], members[2]), (members[3], members[1])],
        ),
    );
    let merged_rows = records(execution, &runtime, &merged.snapshot);
    assert_eq!(identity(&merged_rows, members[1]), original);
    let retired = &merged_rows
        .iter()
        .find(|(_, row)| row.identity == split_identity)
        .unwrap()
        .1;
    assert_eq!(retired.current_anchor, None);
    assert_eq!(retired.retired, Some(merged.commit.commit_id));
    assert_eq!(identity(&merged_rows, members[4]), unrelated);
    let read = runtime
        .read_truth()
        .read_snapshot(&merged.snapshot, execution)
        .unwrap()
        .unwrap();
    let row_identity = |entity| {
        merged_rows
            .iter()
            .find(|(id, _)| *id == entity)
            .unwrap()
            .1
            .identity
    };
    let lineage: Vec<_> = read
        .relations()
        .into_iter()
        .filter(|edge| edge.kind.kind_id == KindId(4))
        .map(|edge| {
            let state = edge.authoritative_aspect_state.as_ref().unwrap();
            let worth_foundational::facade::ContractValidatedAspectValueView::Struct(fields) =
                state.get(&aspect_key("aggregate-lineage")).unwrap().view()
            else {
                panic!("lineage grammar")
            };
            let worth_foundational::facade::AspectValue::UInt64(at) =
                fields.get(&field_key("occurrence")).unwrap()
            else {
                panic!("occurrence")
            };
            let worth_foundational::facade::AspectValue::UInt8(kind) =
                fields.get(&field_key("kind")).unwrap()
            else {
                panic!("kind")
            };
            (
                *at,
                *kind,
                row_identity(edge.source),
                row_identity(edge.target),
            )
        })
        .collect();
    assert_eq!(lineage.len(), 2);
    assert_eq!(
        lineage
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>(),
        std::collections::BTreeSet::from([
            (split.commit.commit_id.0, 1u8, split_identity, original),
            (merged.commit.commit_id.0, 2u8, split_identity, original),
        ])
    );
    let ownership: Vec<_> = read
        .relations()
        .into_iter()
        .filter(|edge| edge.kind.kind_id == KindId(5) && read.get_entity(edge.source).is_some())
        .map(|edge| (edge.source, row_identity(edge.target)))
        .collect();
    assert_eq!(ownership.len(), 4);
    assert_eq!(
        ownership
            .into_iter()
            .collect::<std::collections::BTreeMap<_, _>>(),
        std::collections::BTreeMap::from([
            (members[1], original),
            (members[2], original),
            (members[3], original),
            (members[4], unrelated),
        ])
    );
}
