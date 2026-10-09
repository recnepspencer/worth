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
        let mut charges = Vec::new();
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
                |work, _| {
                    charges.push(work);
                    Ok::<_, ()>(())
                },
            )
            .expect("admitted exact lookup");
        use crate::indexes::data::SelectedIndexReadWork::{Operation, OrderedNavigation};
        let navigation: u64 = charges
            .iter()
            .filter_map(|w| {
                if let OrderedNavigation(n) = w {
                    Some(*n)
                } else {
                    None
                }
            })
            .sum();
        let descents = charges
            .iter()
            .filter(|w| matches!(w, OrderedNavigation(_)))
            .count() as u64;
        assert!(
            navigation
                <= descents
                    * crate::indexes::data::SelectedIndexReadWork::MAXIMUM_ORDERED_DESCENT_WORK
        );
        if mode == BoundedIndexParityMode::Production {
            // One index, one exact scope/generation, one candidate, one scalar aspect:
            // catalog/key descents are navigation; validation, row reads and payload are operations.
            let classified: Vec<_> = charges
                .iter()
                .map(|w| matches!(w, OrderedNavigation(_)))
                .collect();
            assert_eq!(
                classified,
                [
                    true, false, false, false, true, false, true, true, true, true, false, false,
                    false, true, false, false, false, true, false, true, false
                ]
            );
            assert_eq!(
                charges
                    .iter()
                    .filter(|w| matches!(w, Operation(38)))
                    .count(),
                1,
                "one candidate row/validation probe"
            );
        }
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
                let work = match work {
                    crate::indexes::data::SelectedIndexReadWork::Operation(work)
                    | crate::indexes::data::SelectedIndexReadWork::OrderedNavigation(work) => work,
                };
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
                let work = match work {
                    crate::indexes::data::SelectedIndexReadWork::Operation(work)
                    | crate::indexes::data::SelectedIndexReadWork::OrderedNavigation(work) => work,
                };
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
