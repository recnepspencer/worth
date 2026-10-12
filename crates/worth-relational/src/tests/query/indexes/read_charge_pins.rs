//! Read-4: moved reads keep their parent's prices or pay more for newly funded work.
use super::*;
use crate::visibility::materialization::read_records::read_prices::{
    EXAMINE_SLOT, KEYED_RECORD, RESERVE_RECORD,
};
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling};
#[test]
fn admitted_field_revision_keeps_its_parent_charge_pin() {
    let host = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = AspectSchemaFixture {
        entity_aspects: vec![entity_field_aspect(aspect_key("name"), field_key("name"))],
        ..AspectSchemaFixture::default()
    }
    .build_runtime();
    let made = create_entity_outcome(&request, &runtime, "revision-pin");
    let entity = changed_entities(&made)[0];
    let view = runtime
        .read_truth()
        .project_snapshot(&made.snapshot)
        .unwrap();
    let locator = worth_foundational::facade::AspectFieldLocator::new(
        worth_foundational::facade::LocatorAuthority::Authoritative,
        aspect_key("name"),
        worth_foundational::facade::CanonicalFieldPath::single(field_key("name")),
    );
    // 7155ffdf95: entry + u32 partition path + slot/kind + one catalog
    // level + scalar binding/text comparisons + revisions + two hashes +
    // one revision-tree level + creation metadata. No price changed.
    let parent = 1 + 33 + 3 + 11 + (2 + 4 + (4 + 1)) + 2 + (4 + 4 + 2) + 11 + 1;
    assert_eq!(parent, 83);
    let mut admitted = 0;
    assert!(view
        .entity_field_revision_admitted(&request, entity, &locator, |units| {
            admitted += units;
            Ok::<_, std::convert::Infallible>(())
        })
        .unwrap()
        .unwrap()
        .is_some());
    assert_eq!(admitted, parent);
    let (answer, report) = request
        .run(ExecutionWorkCeiling::new(parent), |_| {
            view.entity_field_revision(&request, entity, &locator)
        })
        .unwrap();
    assert!(answer.unwrap().is_some());
    assert_eq!(
        report.charged_work(),
        parent,
        "the unrestricted twin now funds the same exact quote"
    );
    release_test_commit_snapshot(&runtime, &made);
}
#[test]
fn related_ordering_keeps_its_keyed_read_charge_pin() {
    let host = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = runtime_with_index_field_aspects();
    let parent = create_entity_outcome(&request, &runtime, "parent-pin");
    let child = create_entity_outcome(&request, &runtime, "child-pin");
    let relation = create_relation_outcome(
        &request,
        &runtime,
        changed_entities(&parent)[0],
        changed_entities(&child)[0],
        "ordering-pin",
    );
    let index = super::related_entity_ordered_lookup::register_related_name_index(&runtime);
    // At 7155ffdf95 the root's relation and child record getters consulted only.
    // The current unleased build funds those two keyed materializations.
    let parent_work = 0;
    let expected = parent_work + EXAMINE_SLOT + RESERVE_RECORD + KEYED_RECORD;
    let (built, report) = request
        .run(ExecutionWorkCeiling::new(expected), |_| {
            runtime.index_authority().build_for_commit(
                &request,
                DerivedIndexBuildRequest {
                    source_commit_id: relation.commit.commit_id,
                    branch_id: BranchId("main".into()),
                    index_ids: vec![index.index_id],
                },
            )
        })
        .unwrap();
    assert!(built.failed_indexes.is_empty(), "{built:?}");
    assert_eq!(report.charged_work(), expected);
    release_test_commit_snapshot(&runtime, &parent);
    release_test_commit_snapshot(&runtime, &child);
    release_test_commit_snapshot(&runtime, &relation);
}
#[test]
fn connectivity_keeps_its_record_read_charge_pin() {
    let host = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let left = create_entity(&request, &runtime, "left-pin");
    let right = create_entity(&request, &runtime, "right-pin");
    create_relation(&request, &runtime, left, right, "connectivity-pin");
    // At 7155ffdf95 the current traversal consulted only. It now pays
    // two entity slots, both edge visits, one record, one component and two frontier visits.
    let parent_work = 0;
    let expected = parent_work + 2 * EXAMINE_SLOT + 2 * EXAMINE_SLOT + KEYED_RECORD + 1 + 2;
    let (answer, report) = request
        .run(ExecutionWorkCeiling::new(expected), |_| {
            runtime.inspect_what_happened().connectivity_summary(
                &request,
                &connectivity_request(
                    crate::facade::inspection::InspectionScope::Current,
                    None,
                    None,
                    true,
                ),
            )
        })
        .unwrap();
    assert_eq!(answer.unwrap().enumerated_entity_count, 2);
    assert_eq!(report.charged_work(), expected);
}
