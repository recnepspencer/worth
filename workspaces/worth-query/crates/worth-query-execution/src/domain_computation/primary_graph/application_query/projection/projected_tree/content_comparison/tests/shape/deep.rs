//! Equality of raw deep representations, never installed nested admission.
use super::*;
use crate::domain_computation::primary_graph::application_query::ProjectionContentStorageFault as Fault;
fn deep(key: &Arc<ApplicationQueryResultSlotKey>) -> WorthQueryApplicationProjectionNode {
    let mut root = shape(key);
    root.relations[0].rows[0] = shape(key);
    root
}
fn compare(
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
        let child = &mut hostile.relations[0].rows[0];
        match axis {
            0 => child.fields[0].value = AspectValue::String("changed".into()),
            1 => child.fields[0].slot_key = slot("deep"), // same text, foreign namespace
            2 => child.fields[0].result_path = "elsewhere.name".into(),
            3 => child.fields[0].slot_type = "other".into(),
            4 => child.source_dependencies_complete = false,
            5 => child.relations[0].slot_key = slot("deep"),
            6 => child.relations[0].cardinality = ApplicationQueryCardinality::ExactlyOne,
            7 => {
                child.relations[0].predicate_sources[0] = EntityId::new(PartitionId::main(), 10, 2)
            }
            8 => child.relations[0].predicate_sources.reverse(),
            9 => child.relations[0].rows.reverse(),
            10 => {
                child.relations[0].rows[0].incoming_relation =
                    Some(RelationId::new(PartitionId::main(), 20, 2))
            }
            11 => child.relations[0].result_path = "root.other".into(),
            12 => child.relations[0].slot_type = "other".into(),
            13 => child.relations[0].rows.clear(),
            14 => child.relations.clear(),
            15 => child.entity_id = EntityId::new(PartitionId::main(), 7, 2),
            16 => child.source_path = Some("other.ancestor".into()),
            _ => unreachable!(),
        }
        assert_eq!(compare(deep(&key), hostile), Ok(false), "axis {axis}");
    }
    let mut empty = deep(&key);
    empty.relations[0].rows[0].relations[0].rows.clear();
    let mut same_empty = deep(&key);
    same_empty.relations[0].rows[0].relations[0].rows.clear();
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
    deeper.relations[0].rows[0].relations[0].rows[0] = deep(&key);
    let mut same = deep(&key);
    same.relations[0].rows[0].relations[0].rows[0] = deep(&key);
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
