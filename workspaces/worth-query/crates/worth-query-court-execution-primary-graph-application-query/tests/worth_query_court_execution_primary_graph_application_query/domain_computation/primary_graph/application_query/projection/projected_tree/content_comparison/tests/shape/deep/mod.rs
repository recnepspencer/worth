//! Equality of raw deep representations, never installed nested admission.
use crate::domain_computation::primary_graph::application_query::projection::projected_tree::content_comparison::tests::shape::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::projection::projected_tree::content_storage::ProjectionContentStorageFault as Fault;
pub fn deep(key: &Arc<ApplicationQueryResultSlotKey>) -> WorthQueryApplicationProjectionNode {
    let mut root = shape(key);
    (*(*root.relations_mut_for_test())[0].rows_mut_for_test())[0] = shape(key);
    root
}
pub fn compare(
    left: WorthQueryApplicationProjectionNode,
    right: WorthQueryApplicationProjectionNode,
) -> Result<bool, Fault> {
    tree(left).same_source_content(
        &tree(right),
        4,
        4,
        &mut |_| Ok(()),
        &mut |_| Ok(None),
        &mut |cause| cause,
    )
}
#[test]
fn descendant_provenance_and_empty_support_cannot_be_omitted() {
    let key = slot("deep");
    assert_eq!(compare(deep(&key), deep(&key)), Ok(true));
    for axis in 0..17 {
        let mut hostile = deep(&key);
        let child = &mut (*(*hostile.relations_mut_for_test())[0].rows_mut_for_test())[0];
        match axis {
            0 => (*(*child.fields_for_test())[0].value_mut_for_test()) = AspectValue::String("changed".into()),
            1 => (*(*child.fields_for_test())[0].slot_key_mut_for_test()) = slot("deep"), // same text, foreign namespace
            2 => (*(*child.fields_for_test())[0].result_path_mut_for_test()) = "elsewhere.name".into(),
            3 => (*(*child.fields_for_test())[0].slot_type_mut_for_test()) = "other".into(),
            4 => (*child.source_dependencies_complete_mut_for_test()) = false,
            5 => (*(*child.relations_mut_for_test())[0].slot_key_mut_for_test()) = slot("deep"),
            6 => (*(*child.relations_mut_for_test())[0].cardinality_mut_for_test()) = ApplicationQueryCardinality::ExactlyOne,
            7 => {
                (*(*child.relations_mut_for_test())[0].predicate_sources_mut_for_test())[0] = EntityId::new(PartitionId::main(), 10, 2)
            }
            8 => (*(*child.relations_mut_for_test())[0].predicate_sources_mut_for_test()).reverse(),
            9 => (*(*child.relations_mut_for_test())[0].rows_for_test()).reverse(),
            10 => {
                (*(*(*child.relations_mut_for_test())[0].rows_for_test())[0].incoming_relation_mut_for_test()) =
                    Some(RelationId::new(PartitionId::main(), 20, 2))
            }
            11 => (*(*child.relations_mut_for_test())[0].result_path_mut_for_test()) = "root.other".into(),
            12 => (*(*child.relations_mut_for_test())[0].slot_type_mut_for_test()) = "other".into(),
            13 => (*(*child.relations_mut_for_test())[0].rows_for_test()).clear(),
            14 => (*child.relations_mut_for_test()).clear(),
            15 => (*child.entity_id_mut_for_test()) = EntityId::new(PartitionId::main(), 7, 2),
            16 => (*child.source_path_mut_for_test()) = Some("other.ancestor".into()),
            _ => unreachable!(),
        }
        assert_eq!(compare(deep(&key), hostile), Ok(false), "axis {axis}");
    }
    let mut empty = deep(&key);
    (*(*(*(*empty.relations_mut_for_test())[0].rows_for_test())[0].relations_for_test())[0].rows_for_test()).clear();
    let mut same_empty = deep(&key);
    (*(*(*(*same_empty.relations_mut_for_test())[0].rows_for_test())[0].relations_for_test())[0].rows_for_test()).clear();
    assert_eq!(compare(empty, same_empty), Ok(true));
}
#[test]
fn finite_stack_refuses_before_unfunded_or_undeclared_growth() {
    let key = slot("bounded");
    let left = tree(deep(&key));
    let right = tree(deep(&key));
    let mut reservations = 0;
    let mut reserve = |_| {
        reservations += 1;
        Ok(None)
    };
    assert_eq!(
        left.same_source_content(
            &right,
            usize::MAX,
            usize::MAX,
            &mut |_| Ok(()),
            &mut reserve,
            &mut |cause| cause
        ),
        Err(Fault::CapacityOverflow)
    );
    assert_eq!(reservations, 0);
    assert_eq!(
        left.same_source_content(
            &right,
            4,
            4,
            &mut |_| Ok(()),
            &mut |_| Err("original-memory"),
            &mut |_| "physical"
        ),
        Err("original-memory")
    );
    assert_eq!(
        left.same_source_content(
            &right,
            4,
            4,
            &mut |_| Err("original-work"),
            &mut |_| panic!("no reserve after stop"),
            &mut |_| "physical"
        ),
        Err("original-work")
    );
    let mut deeper = deep(&key);
    (*(*(*(*deeper.relations_mut_for_test())[0].rows_for_test())[0].relations_for_test())[0].rows_mut_for_test())[0] = deep(&key);
    let mut same = deep(&key);
    (*(*(*(*same.relations_mut_for_test())[0].rows_for_test())[0].relations_for_test())[0].rows_mut_for_test())[0] = deep(&key);
    assert_eq!(
        tree(deeper).same_source_content(
            &tree(same),
            3,
            3,
            &mut |_| Ok(()),
            &mut |_| Ok(None),
            &mut |cause| cause
        ),
        Err(Fault::CapacityOverflow)
    );
    assert_eq!(
        left.same_source_content(
            &right,
            4,
            4,
            &mut |work| if work == 0 { Err("final-stop") } else { Ok(()) },
            &mut |_| Ok(None),
            &mut |_| "physical"
        ),
        Err("final-stop")
    );
}
#[test]
fn shallow_profile_uses_no_scratch_and_still_refuses_unadmitted_descendants() {
    let key = slot("shallow");
    let left = tree(shape(&key));
    assert_eq!(
        left.same_source_content(
            &tree(shape(&key)),
            2,
            2,
            &mut |_| Ok::<_, ()>(()),
            &mut |_| panic!("shallow reserve"),
            &mut |_| panic!("shallow physical fault")
        ),
        Ok(true)
    );
    assert_eq!(
        tree(deep(&key)).same_source_content(
            &tree(deep(&key)),
            2,
            2,
            &mut |_| Ok::<_, ()>(()),
            &mut |_| panic!("shallow reserve"),
            &mut |_| panic!("shallow physical fault")
        ),
        Ok(false)
    );
}
