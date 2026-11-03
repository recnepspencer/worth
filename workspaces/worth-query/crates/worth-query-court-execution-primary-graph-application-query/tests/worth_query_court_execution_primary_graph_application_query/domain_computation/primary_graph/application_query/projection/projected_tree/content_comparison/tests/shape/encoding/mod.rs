//! Raw representation framing only; actual compiled-profile admission is a consumer court.
use crate::domain_computation::primary_graph::application_query::projection::projected_tree::content_comparison::tests::shape::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::projection::projected_tree::content::ContentWriter;
pub fn encode(node: WorthQueryApplicationProjectionNode, nested: bool) -> [u8; 32] {
    let entity = node.entity_id;
    let tree = tree(node);
    let mut admit = |_| Ok::<_, &'static str>(());
    let mut writer = ContentWriter::new(&mut admit, &|| "overflow");
    writer
        .bytes(if nested {
            b"worth:projected-source-entry:v2"
        } else {
            b"worth:projected-source-entry:v1"
        })
        .unwrap();
    writer.entity(entity).unwrap();
    if nested {
        tree.append_tree_content(&mut writer, 8, &mut |_| Ok(None), &mut |_| "storage")
            .unwrap();
    } else {
        tree.append_one_hop_content(&mut writer, || "unsupported")
            .unwrap();
    }
    writer.number(0).unwrap();
    writer.finish()
}
#[test]
fn shallow_entry_v1_keeps_literal_framed_bytes() {
    // Independently assembled length-prefixed protocol fixture, not a digest
    // obtained by evaluating this writer when the test runs.
    assert_eq!(
        encode(row(), false),
        [
            47, 212, 62, 172, 136, 197, 50, 129, 127, 150, 106, 221, 220, 0, 235, 174, 174, 174,
            29, 42, 13, 140, 90, 227, 40, 138, 69, 10, 18, 160, 139, 251
        ]
    );
}
#[test]
fn nested_framing_preserves_descendants_and_declared_empty_collections() {
    let key = slot("framed");
    let mut original = shape(&key);
    original.relations[0].rows[0] = shape(&key);
    let mut same = shape(&key);
    same.relations[0].rows[0] = shape(&key);
    let reference = encode(original, true);
    assert_eq!(reference, encode(same, true));
    let mut different = shape(&key);
    different.relations[0].rows[0] = shape(&key);
    different.relations[0].rows[0].fields[0].value =
        AspectValue::String("changed descendant".into());
    assert_ne!(reference, encode(different, true));
    let mut empty = shape(&key);
    empty.relations[0].rows.clear();
    let mut another = shape(&key);
    another.relations[0].rows.clear();
    assert_ne!(
        encode(empty, true),
        encode(another, false),
        "compiled deep profile has its own domain even without live descendants"
    );
    let mut empty = shape(&key);
    empty.relations[0].rows[0] = shape(&key);
    empty.relations[0].rows[0].relations[0].rows.clear();
    let mut absent = shape(&key);
    absent.relations[0].rows[0] = shape(&key);
    absent.relations[0].rows[0].relations.clear();
    assert_ne!(encode(empty, true), encode(absent, true));
}
#[test]
fn nested_encoding_preserves_original_work_memory_and_bound_refusals() {
    let key = slot("stopped");
    let tree = tree(shape(&key));
    for fail in [0, 1, 8, 32] {
        let mut visits = 0;
        let mut admit = |_| {
            let refused = visits == fail;
            visits += 1;
            if refused {
                Err("original-work")
            } else {
                Ok(())
            }
        };
        let mut writer = ContentWriter::new(&mut admit, &|| "overflow");
        assert_eq!(
            tree.append_tree_content(&mut writer, 8, &mut |_| Ok(None), &mut |_| "physical"),
            Err("original-work")
        );
    }
    let mut admit = |_| Ok::<_, &'static str>(());
    let mut writer = ContentWriter::new(&mut admit, &|| "overflow");
    assert_eq!(
        tree.append_tree_content(&mut writer, 8, &mut |_| Err("original-memory"), &mut |_| {
            "physical"
        }),
        Err("original-memory")
    );
    assert_eq!(
        tree.append_tree_content(&mut writer, 1, &mut |_| Ok(None), &mut |_| "declared-bound"),
        Err("declared-bound")
    );
}
