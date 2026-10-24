use super::*;
use crate::facade::indexes::{BoundedEntityFieldPageStop, DerivedIndexDiscardRequest};

#[test]
fn exact_posting_pages_resume_without_prefix_replay_and_reject_selector_or_generation_drift() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = runtime_with_index_field_aspects();
    let first = create_entity_outcome(execution, &runtime, "shared");
    let second = create_entity_outcome(execution, &runtime, "shared");
    let third = create_entity_outcome(execution, &runtime, "shared");
    let mut expected = vec![
        changed_entities(&first)[0],
        changed_entities(&second)[0],
        changed_entities(&third)[0],
    ];
    expected.sort();
    let locator = aspect_field_locator(aspect_key("name"), field_key("name"));
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity.name.pages".into(),
        kind: DerivedIndexKind::EntityField {
            field_locator: locator.clone(),
        },
        branch_scoped: false,
    });
    let build = || {
        runtime.index_authority().build_for_commit(
            execution,
            DerivedIndexBuildRequest {
                source_commit_id: third.commit.commit_id,
                branch_id: BranchId("main".into()),
                index_ids: vec![index.index_id],
            },
        )
    };
    assert!(build().failed_indexes.is_empty());
    let view = runtime
        .read_truth()
        .project_snapshot(&third.snapshot)
        .unwrap();
    let prepare = || {
        runtime
            .index_access()
            .prepare_entity_field_lookup(&view, index.index_id, KindId(1), &locator, execution)
            .unwrap()
    };
    let prepared = prepare();
    let value = string_aspect_value("shared");
    let first = prepared
        .execute_prepared_entity_field_page(&value, 1, None, execution, || Ok::<_, ()>(()))
        .unwrap();
    assert_eq!(first.candidate_entity_ids(), &expected[..1]);
    assert_eq!(first.examined_entry_count(), 1);
    let next = first.next_boundary().unwrap();
    let other_kind = runtime
        .index_access()
        .prepare_entity_field_lookup(&view, index.index_id, KindId(2), &locator, execution)
        .unwrap();
    assert!(matches!(
        other_kind.execute_prepared_entity_field_page(&value, 1, Some(next), execution, || Ok::<
            _,
            (),
        >(
            ()
        )),
        Err(BoundedEntityFieldPageStop::ForeignBoundary)
    ));
    let prior_build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: second.commit.commit_id,
            branch_id: BranchId("main".into()),
            index_ids: vec![index.index_id],
        },
    );
    assert!(prior_build.failed_indexes.is_empty());
    let prior_view = runtime
        .read_truth()
        .project_snapshot(&second.snapshot)
        .unwrap();
    let prior = runtime
        .index_access()
        .prepare_entity_field_lookup(&prior_view, index.index_id, KindId(1), &locator, execution)
        .unwrap();
    assert!(matches!(
        prior.execute_prepared_entity_field_page(&value, 1, Some(next), execution, || Ok::<_, ()>(
            ()
        )),
        Err(BoundedEntityFieldPageStop::ForeignBoundary)
    ));
    let foreign_runtime = runtime_with_index_field_aspects();
    let foreign_commit = create_entity_outcome(execution, &foreign_runtime, "shared");
    let foreign_index = foreign_runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "entity.name.pages".into(),
            kind: DerivedIndexKind::EntityField {
                field_locator: locator.clone(),
            },
            branch_scoped: false,
        });
    assert!(foreign_runtime
        .index_authority()
        .build_for_commit(
            execution,
            DerivedIndexBuildRequest {
                source_commit_id: foreign_commit.commit.commit_id,
                branch_id: BranchId("main".into()),
                index_ids: vec![foreign_index.index_id],
            }
        )
        .failed_indexes
        .is_empty());
    let foreign_view = foreign_runtime
        .read_truth()
        .project_snapshot(&foreign_commit.snapshot)
        .unwrap();
    let foreign = foreign_runtime
        .index_access()
        .prepare_entity_field_lookup(
            &foreign_view,
            foreign_index.index_id,
            KindId(1),
            &locator,
            execution,
        )
        .unwrap();
    assert!(matches!(
        foreign.execute_prepared_entity_field_page(
            &value,
            1,
            Some(next),
            execution,
            || Ok::<_, ()>(())
        ),
        Err(BoundedEntityFieldPageStop::ForeignBoundary)
    ));
    assert!(matches!(
        prepared.execute_prepared_entity_field_page(
            &string_aspect_value("other"),
            1,
            Some(next),
            execution,
            || Ok::<_, ()>(())
        ),
        Err(BoundedEntityFieldPageStop::ForeignBoundary)
    ));
    assert!(matches!(
        prepared.execute_prepared_entity_field_page(&value, 1, Some(next), execution, || Err(
            "admission stopped"
        )),
        Err(BoundedEntityFieldPageStop::Admission("admission stopped"))
    ));
    let rest = prepared
        .execute_prepared_entity_field_page(&value, 2, Some(next), execution, || Ok::<_, ()>(()))
        .unwrap();
    assert_eq!(rest.candidate_entity_ids(), &expected[1..]);
    assert_eq!(rest.examined_entry_count(), 2);
    assert!(rest.next_boundary().is_none());
    runtime
        .index_authority()
        .discard_generations(DerivedIndexDiscardRequest::all_bases(index.index_id))
        .unwrap();
    assert!(build().failed_indexes.is_empty());
    let rebuilt = prepare();
    assert!(matches!(
        rebuilt.execute_prepared_entity_field_page(
            &value,
            1,
            Some(next),
            execution,
            || Ok::<_, ()>(())
        ),
        Err(BoundedEntityFieldPageStop::ForeignBoundary)
    ));
    // The original issued pin remains usable, whereas a newly selected generation cannot adopt its cursor.
    assert_eq!(
        prepared
            .execute_prepared_entity_field_page(
                &value,
                2,
                Some(next),
                execution,
                || Ok::<_, ()>(())
            )
            .unwrap()
            .candidate_entity_ids(),
        &expected[1..]
    );
}
