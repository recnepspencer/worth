//! Pure raw-tree representation twins; these tokens grant no installed access.
use super::*;
use worth_query_declaration::facade::{application_query::*, application_schema::*};

struct Query;
worth_query_declaration::worth_query_structured_value_binding!(UnitBinding for () { identity: "worth.query.content-comparison.unit" });
impl ApplicationQueryMarkerIdentity<()> for Query {
    type ParameterBinding = UnitBinding;
    type ResultBinding = UnitBinding;
    type Scope = ();
    const IDENTIFIER: &'static str = "content-shape";
    const QUERY_TYPE_NAME: &'static str = "content-shape";
    const SCOPE_TYPE_NAME: &'static str = "unit";
}
fn slot(name: &'static str) -> Arc<ApplicationQueryResultSlotKey> {
    let reference = ApplicationRelationRef::<(), (), (), ()>::from_schema_identifiers(
        "members",
        "scope",
        "item",
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
    );
    Arc::new(
        ApplicationQueryResultRelationRef::<
            Query,
            (),
            (),
            (),
            (),
            (),
            ForwardResultTraversal,
            ManyResults,
        >::forward_many(name, reference)
        .slot_key(),
    )
}
fn shape(slot: &Arc<ApplicationQueryResultSlotKey>) -> WorthQueryApplicationProjectionNode {
    let mut root = row();
    root.fields.push(WorthQueryApplicationProjectedField {
        result_path: "root.name".into(),
        slot_type: "name".into(),
        slot_key: slot.clone(),
        value: AspectValue::String("source-secret".into()),
    });
    let mut first = row();
    first.entity_id = EntityId::new(PartitionId::main(), 10, 1);
    first.incoming_relation = Some(RelationId::new(PartitionId::main(), 20, 1));
    let mut second = row();
    second.entity_id = EntityId::new(PartitionId::main(), 11, 1);
    second.incoming_relation = Some(RelationId::new(PartitionId::main(), 21, 1));
    root.relations.push(WorthQueryApplicationProjectedRelation {
        result_path: "root.children".into(),
        slot_type: "children".into(),
        slot_key: slot.clone(),
        cardinality: ApplicationQueryCardinality::Many,
        predicate_sources: vec![first.entity_id, second.entity_id],
        rows: vec![first, second],
    });
    root
}
#[test]
fn complete_raw_shape_compares_fields_relations_negative_support_order_and_depth() {
    let key = slot("content");
    let reference = tree(shape(&key));
    assert!(reference
        .same_one_hop_content(&tree(shape(&key)), &mut |_| Ok::<_, ()>(()))
        .unwrap());
    // Each hostile differs from a separately built lawful twin on one axis.
    for axis in 0..14 {
        let mut hostile = shape(&key);
        match axis {
            0 => hostile.fields[0].value = AspectValue::String("changed-secret".into()),
            1 => hostile.fields[0].slot_key = slot("other"),
            2 => hostile.fields[0].result_path = "root.other".into(),
            3 => hostile.fields[0].slot_type = "other".into(),
            4 => hostile.fields.clear(),
            5 => hostile.relations[0].slot_key = slot("other"),
            6 => hostile.relations[0].cardinality = ApplicationQueryCardinality::OptionalOne,
            7 => {
                hostile.relations[0].predicate_sources[0] =
                    EntityId::new(PartitionId::main(), 10, 2)
            }
            8 => hostile.relations[0].predicate_sources.reverse(),
            9 => hostile.relations[0].rows.reverse(),
            10 => {
                hostile.relations[0].rows[0].incoming_relation =
                    Some(RelationId::new(PartitionId::main(), 20, 2))
            }
            11 => hostile.relations[0].result_path = "root.other".into(),
            12 => hostile.relations[0].slot_type = "other".into(),
            13 => {
                let relation = shape(&key).relations.pop().unwrap();
                hostile.relations[0].rows[0].relations.push(relation);
            }
            _ => unreachable!(),
        }
        assert!(
            !reference
                .same_one_hop_content(&tree(hostile), &mut |_| Ok::<_, ()>(()))
                .unwrap(),
            "axis {axis}"
        );
    }
    let mut deeper = shape(&key);
    let relation = shape(&key).relations.pop().unwrap();
    deeper.relations[0].rows[0].relations.push(relation);
    let mut same_deeper = shape(&key);
    let relation = shape(&key).relations.pop().unwrap();
    same_deeper.relations[0].rows[0].relations.push(relation);
    assert!(!tree(deeper)
        .same_one_hop_content(&tree(same_deeper), &mut |_| Ok::<_, ()>(()))
        .unwrap());
}

mod deep;

mod encoding;
mod lifetime;
