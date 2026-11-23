use std::sync::Arc;

use super::same_commit::SameCommitNameReference;
use super::{
    ApplicationNameDiscriminatorEncoder, NameRef, NameSelection, NamedTarget, WorthQueryName,
    WorthQuerySameCommitName,
};

struct Face;

fn committed(byte: u8) -> WorthQueryName<Face> {
    WorthQueryName::from_admitted_digest([byte; 32])
}

fn fresh(candidate: &Arc<()>, output: usize) -> WorthQuerySameCommitName<Face> {
    WorthQuerySameCommitName::issued(SameCommitNameReference::issued(
        Arc::clone(candidate),
        output,
    ))
}

/// One discriminator shape, declared once over `NameRef`.
fn cut_by(
    cutter: &NameRef<Face>,
    boundary: Option<NameRef<Face>>,
) -> ApplicationNameDiscriminatorEncoder {
    let mut encoder = ApplicationNameDiscriminatorEncoder::new("worth.tests.cut-by.v1");
    encoder
        .variant("SidedCut")
        .field("cutter", cutter)
        .field("side", &1_u8)
        .field("boundary", &boundary);
    encoder
}

#[test]
fn a_committed_reference_encodes_exactly_as_the_name_it_holds() {
    let (cutter, boundary) = (committed(0xaa), committed(0xbb));
    let mut by_name = ApplicationNameDiscriminatorEncoder::new("worth.tests.cut-by.v1");
    by_name
        .variant("SidedCut")
        .field("cutter", &cutter)
        .field("side", &1_u8)
        .field("boundary", &Some(boundary));
    let by_ref = cut_by(&cutter.into(), Some(boundary.into()));
    assert_eq!(
        by_ref.finish_with_same_commit_names(),
        by_name.finish_with_same_commit_names()
    );
}

#[test]
fn a_same_commit_reference_becomes_the_committed_bytes_once_its_name_is_written() {
    let candidate = Arc::new(());
    let (cutter, boundary) = (committed(0xaa), committed(0xbb));
    let (committed_bytes, committed_refs, no_slots) =
        cut_by(&cutter.into(), Some(boundary.into())).finish_with_same_commit_names();
    assert!(no_slots.is_empty());
    assert_eq!(committed_refs, [[0xaa; 32], [0xbb; 32]]);

    let in_commit = cut_by(
        &fresh(&candidate, 0).into(),
        Some(fresh(&candidate, 1).into()),
    );
    let (mut bytes, refs, slots) = in_commit.finish_with_same_commit_names();
    assert!(refs.is_empty(), "a same-commit reference reveals no digest");
    assert_ne!(bytes, committed_bytes);
    assert_eq!(bytes.len(), committed_bytes.len());
    let outputs: Vec<_> = slots.iter().map(|slot| slot.reference().output()).collect();
    assert_eq!(outputs, [0, 1]);
    // Lowering writes each referenced name over its placeholder.
    for (slot, name) in slots.iter().zip([cutter, boundary]) {
        bytes[slot.offset()..slot.offset() + 32].copy_from_slice(name.as_bytes());
    }
    assert_eq!(bytes, committed_bytes);

    // One discriminator may mix the two forms.
    let (mut mixed, mixed_refs, mixed_slots) =
        cut_by(&cutter.into(), Some(fresh(&candidate, 1).into())).finish_with_same_commit_names();
    assert_eq!(mixed_refs, [[0xaa; 32]]);
    let [slot] = mixed_slots.as_slice() else {
        panic!("one same-commit reference leaves one slot: {mixed_slots:?}");
    };
    mixed[slot.offset()..slot.offset() + 32].copy_from_slice(boundary.as_bytes());
    assert_eq!(mixed, committed_bytes);
}

#[test]
fn references_compare_by_form_and_then_by_what_they_reference() {
    let candidate = Arc::new(());
    let name: NameRef<Face> = committed(1).into();
    assert_eq!(name, NameRef::Committed(committed(1)));
    assert_ne!(name, NameRef::Committed(committed(2)));
    let first: NameRef<Face> = fresh(&candidate, 0).into();
    assert_eq!(first.clone(), first);
    assert_ne!(first, NameRef::InCommit(fresh(&candidate, 1)));
    assert_ne!(first, NameRef::InCommit(fresh(&Arc::new(()), 0)));
    assert_ne!(first, name);
    assert!(format!("{name:?}").starts_with("Committed(WorthQueryName"));
    assert!(format!("{first:?}").starts_with("InCommit(WorthQuerySameCommitName"));
}

#[test]
fn a_named_target_holds_its_name_and_selection_and_orders_by_both() {
    let (low, high) = (committed(1), committed(2));
    let exact = NamedTarget::exact(low);
    assert_eq!(exact.name(), low);
    assert_eq!(exact.selection(), &NameSelection::Exact);
    assert_eq!(exact, NamedTarget::new(low, NameSelection::Exact));
    assert_eq!(exact.clone().into_parts(), (low, NameSelection::Exact));
    assert_ne!(exact, NamedTarget::all_descendants(low));

    let bounded = NamedTarget::new(
        low,
        NameSelection::bounded_by_carriers([high.erased()]).unwrap(),
    );
    let mut targets = vec![
        NamedTarget::exact(high),
        bounded.clone(),
        NamedTarget::all_descendants(low),
        exact.clone(),
    ];
    targets.sort();
    assert_eq!(
        targets,
        [
            exact,
            NamedTarget::all_descendants(low),
            bounded,
            NamedTarget::exact(high)
        ]
    );
    assert!(format!("{:?}", targets[0]).starts_with("NamedTarget { name: WorthQueryName"));
}
