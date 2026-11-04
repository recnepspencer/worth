//! Pure raw-tree representation twins; these tokens grant no installed access.
pub use crate::domain_computation::primary_graph::application_query::projection::projected_tree::content_comparison::tests::*;
pub use worth_query_declaration::facade::application_query::*;
pub use worth_query_declaration::facade::application_schema::*;

pub struct Query;
worth_query_declaration::worth_query_structured_value_binding!(pub UnitBinding for () { identity: "worth.query.content-comparison.unit" });
impl ApplicationQueryMarkerIdentity<()> for Query {
    type ParameterBinding = UnitBinding;
    type ResultBinding = UnitBinding;
    type Scope = ();
    const IDENTIFIER: &'static str = "content-shape";
    const QUERY_TYPE_NAME: &'static str = "content-shape";
    const SCOPE_TYPE_NAME: &'static str = "unit";
}
pub fn slot(name: &'static str) -> Arc<ApplicationQueryResultSlotKey> {
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
pub fn shape(slot: &Arc<ApplicationQueryResultSlotKey>) -> WorthQueryApplicationProjectionNode {
    let mut root = row();
    (*root.fields_mut_for_test()).push(WorthQueryApplicationProjectedField {
        result_path: "root.name".into(),
        slot_type: "name".into(),
        slot_key: slot.clone(),
        value: AspectValue::String("source-secret".into()),
    });
    let mut first = row();
    (*first.entity_id_mut_for_test()) = EntityId::new(PartitionId::main(), 10, 1);
    (*first.incoming_relation_mut_for_test()) = Some(RelationId::new(PartitionId::main(), 20, 1));
    let mut second = row();
    (*second.entity_id_mut_for_test()) = EntityId::new(PartitionId::main(), 11, 1);
    (*second.incoming_relation_mut_for_test()) = Some(RelationId::new(PartitionId::main(), 21, 1));
    (*root.relations_mut_for_test()).push(WorthQueryApplicationProjectedRelation {
        result_path: "root.children".into(),
        slot_type: "children".into(),
        slot_key: slot.clone(),
        cardinality: ApplicationQueryCardinality::Many,
        predicate_sources: vec![(*first.entity_id_for_test()), (*second.entity_id_for_test())],
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
            0 => (*(*hostile.fields_mut_for_test())[0].value_mut_for_test()) = AspectValue::String("changed-secret".into()),
            1 => (*(*hostile.fields_mut_for_test())[0].slot_key_mut_for_test()) = slot("other"),
            2 => (*(*hostile.fields_mut_for_test())[0].result_path_mut_for_test()) = "root.other".into(),
            3 => (*(*hostile.fields_mut_for_test())[0].slot_type_mut_for_test()) = "other".into(),
            4 => (*hostile.fields_mut_for_test()).clear(),
            5 => (*(*hostile.relations_mut_for_test())[0].slot_key_mut_for_test()) = slot("other"),
            6 => (*(*hostile.relations_mut_for_test())[0].cardinality_mut_for_test()) = ApplicationQueryCardinality::OptionalOne,
            7 => {
                (*(*hostile.relations_mut_for_test())[0].predicate_sources_mut_for_test())[0] =
                    EntityId::new(PartitionId::main(), 10, 2)
            }
            8 => (*(*hostile.relations_mut_for_test())[0].predicate_sources_mut_for_test()).reverse(),
            9 => (*(*hostile.relations_mut_for_test())[0].rows_mut_for_test()).reverse(),
            10 => {
                (*(*(*hostile.relations_mut_for_test())[0].rows_mut_for_test())[0].incoming_relation_mut_for_test()) =
                    Some(RelationId::new(PartitionId::main(), 20, 2))
            }
            11 => (*(*hostile.relations_mut_for_test())[0].result_path_mut_for_test()) = "root.other".into(),
            12 => (*(*hostile.relations_mut_for_test())[0].slot_type_mut_for_test()) = "other".into(),
            13 => {
                let relation = (*shape(&key).relations_mut_for_test()).pop().unwrap();
                (*(*(*hostile.relations_mut_for_test())[0].rows_mut_for_test())[0].relations_mut_for_test()).push(relation);
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
    let relation = (*shape(&key).relations_mut_for_test()).pop().unwrap();
    (*(*(*deeper.relations_mut_for_test())[0].rows_mut_for_test())[0].relations_mut_for_test()).push(relation);
    let mut same_deeper = shape(&key);
    let relation = (*shape(&key).relations_mut_for_test()).pop().unwrap();
    (*(*(*same_deeper.relations_mut_for_test())[0].rows_mut_for_test())[0].relations_mut_for_test()).push(relation);
    assert!(!tree(deeper)
        .same_one_hop_content(&tree(same_deeper), &mut |_| Ok::<_, ()>(()))
        .unwrap());
}

#[path = "deep/mod.rs"]
pub mod deep;
#[path = "encoding/mod.rs"]
pub mod encoding;
#[path = "lifetime/mod.rs"]
pub mod lifetime;
