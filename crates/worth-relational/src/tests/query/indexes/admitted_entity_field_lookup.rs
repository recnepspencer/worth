use super::*;
use crate::facade::identity::KindId;
use crate::facade::indexes::{
    BoundedEntityFieldLookupAdmissionStop, BoundedEntityFieldLookupRequest, BoundedIndexParityMode,
};

#[test]
fn exact_admitted_lookup_preserves_production_and_certification_results() {
    let runtime = runtime_with_index_field_aspects();
    let created = create_entity_outcome(&runtime, "admitted-lookup");
    let locator = aspect_field_locator(aspect_key("name"), field_key("name"));
    let value = string_aspect_value("admitted-lookup");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(915),
        name: "entity.name.admitted".to_owned(),
        kind: DerivedIndexKind::EntityField {
            field_locator: locator.clone(),
        },
        branch_scoped: false,
    });
    let build = runtime
        .index_authority()
        .build_for_commit(DerivedIndexBuildRequest {
            source_commit_id: created.commit.commit_id,
            branch_id: BranchId("main".to_owned()),
            index_ids: vec![index.index_id],
        });
    assert!(build.failed_indexes.is_empty());
    let view = runtime
        .read_truth()
        .project_snapshot(&created.snapshot)
        .expect("issued exact snapshot");
    for mode in [
        BoundedIndexParityMode::Production,
        BoundedIndexParityMode::Certification,
    ] {
        let legacy = runtime
            .index_access()
            .execute_bounded_entity_field_lookup(
                BoundedEntityFieldLookupRequest::new(
                    created.snapshot.clone(),
                    index.index_id,
                    KindId(1),
                    locator.clone(),
                    value.clone(),
                    2,
                )
                .expect("bounded request"),
                mode,
            )
            .expect("legacy exact lookup");
        let admitted = runtime
            .index_access()
            .execute_bounded_entity_field_lookup_admitted(
                &view,
                index.index_id,
                KindId(1),
                &locator,
                &value,
                2,
                mode,
                |_, _| Ok::<_, ()>(()),
            )
            .expect("admitted exact lookup");
        assert_eq!(
            admitted.candidate_entity_ids(),
            legacy.candidate_entity_ids()
        );
        assert_eq!(
            admitted.examined_entry_count(),
            legacy.examined_entry_count()
        );
        assert_eq!(admitted.overflowed(), legacy.overflowed());
        assert_eq!(admitted.generation_id(), legacy.generation_id());
    }
}

#[test]
fn exact_admitted_lookup_keeps_a_one_short_stop_in_the_callers_lane() {
    let runtime = runtime_with_index_field_aspects();
    let created = create_entity_outcome(&runtime, "short-stop");
    let locator = aspect_field_locator(aspect_key("name"), field_key("name"));
    let value = string_aspect_value("short-stop");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(916),
        name: "entity.name.one-short".to_owned(),
        kind: DerivedIndexKind::EntityField {
            field_locator: locator.clone(),
        },
        branch_scoped: false,
    });
    let build = runtime
        .index_authority()
        .build_for_commit(DerivedIndexBuildRequest {
            source_commit_id: created.commit.commit_id,
            branch_id: BranchId("main".to_owned()),
            index_ids: vec![index.index_id],
        });
    assert!(build.failed_indexes.is_empty());
    let view = runtime
        .read_truth()
        .project_snapshot(&created.snapshot)
        .unwrap();
    let mut actual_work = 0_u64;
    runtime
        .index_access()
        .execute_bounded_entity_field_lookup_admitted(
            &view,
            index.index_id,
            KindId(1),
            &locator,
            &value,
            2,
            BoundedIndexParityMode::Certification,
            |work, _| {
                actual_work = actual_work.checked_add(work).unwrap();
                Ok::<_, ()>(())
            },
        )
        .expect("measured admitted lookup");
    assert!(actual_work > 1);
    let mut remaining = actual_work - 1;
    let denial = runtime
        .index_access()
        .execute_bounded_entity_field_lookup_admitted(
            &view,
            index.index_id,
            KindId(1),
            &locator,
            &value,
            2,
            BoundedIndexParityMode::Certification,
            |work, _| {
                if work > remaining {
                    return Err("work");
                }
                remaining -= work;
                Ok(())
            },
        )
        .expect_err("one short must stop before the final admitted read");
    assert!(matches!(
        denial,
        BoundedEntityFieldLookupAdmissionStop::Admission("work")
    ));
}
