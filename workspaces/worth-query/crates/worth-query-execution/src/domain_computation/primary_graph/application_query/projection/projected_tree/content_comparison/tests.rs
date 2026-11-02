//! Representation-only tests: no runtime, installed plan or Source authority.
use super::*;
use crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance;
use worth_relational::facade::identity::PartitionId;

fn row() -> WorthQueryApplicationProjectionNode {
    WorthQueryApplicationProjectionNode::new(
        EntityId::new(PartitionId::main(), 7, 1),
        None,
        true,
        Vec::new(),
        Vec::new(),
    )
}
fn tree(row: WorthQueryApplicationProjectionNode) -> WorthQueryApplicationDisclosedProjectionTree {
    WorthQueryApplicationWorkingProjectionTree::new(vec![row])
        .into_disclosed(&WorthQueryApplicationQueryGovernance::Public)
}
#[test]
fn independent_projection_bodies_keep_full_generation_occurrence_path_and_completeness() {
    let original = tree(row());
    let compare = |other| {
        original.same_one_hop_content(
            &tree(other),
            &mut |_| Ok::<_, ()>(()),
            &mut |_| (),
            &mut |_, _| panic!("representation fixture has no Native frames"),
        )
    };
    assert_eq!(compare(row()), Ok(true));
    let mut different = row();
    different.entity_id = EntityId::new(PartitionId::main(), 7, 2);
    assert_eq!(compare(different), Ok(false));
    let mut different = row();
    different.source_dependencies_complete = false;
    assert_eq!(compare(different), Ok(false));
    let mut different = row();
    different.source_path = Some("root.child".into());
    assert_eq!(compare(different), Ok(false));
    let mut different = row();
    different.incoming_relation = Some(RelationId::new(PartitionId::main(), 9, 1));
    assert_eq!(compare(different), Ok(false));
    let mut left = row();
    left.incoming_relation = Some(RelationId::new(PartitionId::main(), 9, 1));
    let mut right = row();
    right.incoming_relation = Some(RelationId::new(PartitionId::main(), 9, 2));
    assert_eq!(
        tree(left).same_one_hop_content(
            &tree(right),
            &mut |_| Ok::<_, ()>(()),
            &mut |_| (),
            &mut |_, _| panic!("representation fixture has no Native frames"),
        ),
        Ok(false)
    );
}
#[test]
fn scalar_comparison_admits_actual_borrowed_width_and_keeps_type_and_value() {
    let text = AspectValue::String("é-content".repeat(4096).into());
    let other = text.clone();
    let mut work = 0;
    assert!(same_value(&text, &other, &mut |units| {
        work += units;
        Ok::<_, ()>(())
    },)
    .unwrap());
    assert_eq!(
        work,
        (text.semantic_byte_width() + other.semantic_byte_width()) as u64
    );
    assert!(!same_value(
        &AspectValue::Int8(1),
        &AspectValue::Int64(1),
        &mut |_| Ok::<_, ()>(()),
    )
    .unwrap());
    assert!(!same_value(
        &text,
        &AspectValue::String("different".into()),
        &mut |_| Ok::<_, ()>(()),
    )
    .unwrap());
}
#[test]
fn comparison_preserves_first_and_final_owned_stop_and_separate_width_admission() {
    #[derive(Debug, Eq, PartialEq)]
    struct Stop(&'static str);
    let left = tree(row());
    let right = tree(row());
    let mut contacts = 0;
    let outcome = left.same_one_hop_content(
        &right,
        &mut |_| {
            contacts += 1;
            Err(Stop("first"))
        },
        &mut |_| Stop("content"),
        &mut |_, _| panic!("representation fixture has no Native frames"),
    );
    assert_eq!(outcome, Err(Stop("first")));
    assert_eq!(contacts, 1);
    let outcome = left.same_one_hop_content(
        &right,
        &mut |work| {
            if work == 0 {
                Err(Stop("final"))
            } else {
                Ok(())
            }
        },
        &mut |_| Stop("content"),
        &mut |_, _| panic!("representation fixture has no Native frames"),
    );
    assert_eq!(outcome, Err(Stop("final")));
    let mut widths = Vec::new();
    assert_eq!(
        admit_pair_width(usize::MAX, 1, &mut |work| {
            widths.push(work);
            Ok::<_, Stop>(())
        }),
        Ok(())
    );
    assert_eq!(widths, [usize::MAX as u64, 1]);
}

mod shape;
