//! Owned storage transitions and stops; this fixture reads no Native graph.
use super::*;
use crate::derived_relation::evaluation::memo::tests::{id, identity, registry};
use crate::derived_relation::DerivedRelationReads;
use crate::graph_member::ProjectedMemberCoordinate;
use crate::projected_member_domain::ProjectedMemberDomainIdentity;
use crate::schema::data::{SchemaId, SchemaVersionId};
use std::sync::Arc;

#[derive(Default)]
struct Control {
    calls: usize,
    stop_at: Option<usize>,
}
fn cancelled() -> Denial {
    Denial::Request(crate::execution::RelationalExecutionDenialCause::Cancelled)
}
impl SourceRead for Control {
    fn checkpoint(&mut self, _: usize) -> Result<(), Denial> {
        self.calls += 1;
        if self.stop_at.is_some_and(|at| self.calls >= at) {
            Err(cancelled())
        } else {
            Ok(())
        }
    }
    fn retain(&mut self, _: usize) -> Result<(), Denial> {
        self.checkpoint(0)
    }
    fn check_scratch_peak(&mut self, _: usize) -> Result<(), Denial> {
        self.checkpoint(0)
    }
    fn entity_kind(
        &mut self,
        _: crate::identity::data::EntityId,
    ) -> Result<crate::identity::data::KindId, Denial> {
        unreachable!("memo never reads graph metadata")
    }
    fn adjacency(
        &mut self,
        _: crate::derived_relation::evaluation::Adjacency,
    ) -> Result<Vec<crate::validation::data::StructuralRelationRecord>, Denial> {
        unreachable!("memo never acquires graph adjacency")
    }
    fn reads(&self) -> DerivedRelationReads {
        DerivedRelationReads::default()
    }
}
#[test]
fn singleton_keeps_original_anchor_and_promotion_admits_before_moving() {
    let registry = registry();
    let definition = registry.derived_relation(&identity("union")).unwrap();
    let original = Arc::new(ProjectedMemberCoordinate {
        domain: ProjectedMemberDomainIdentity {
            schema: SchemaId("memo-singleton".into()),
            version: SchemaVersionId(1),
            name: "case".into(),
        },
        carrier: id(1),
        case: "same".into(),
    });
    let first = Key::root(
        definition.clone(),
        0,
        Member::Projected(original.clone()),
        false,
    );
    let equal = Key::root(
        definition.clone(),
        0,
        Member::Projected(Arc::new((*original).clone())),
        false,
    );
    let second = Key::root(definition, 0, id(2).into(), false);
    let a = BTreeSet::from([id(3).into()]);
    let b = BTreeSet::from([id(4).into()]);
    let mut actual = MemoValues::default();
    let mut control = Control::default();
    assert_eq!(
        actual
            .insert(first.clone(), a.clone(), &mut control)
            .unwrap(),
        None
    );
    assert_eq!(
        actual
            .insert(equal.clone(), b.clone(), &mut control)
            .unwrap(),
        Some(a.clone())
    );
    let Bucket::One {
        anchor: Member::Projected(stored),
        ..
    } = actual.root.pages[&0].slots[0].as_deref().unwrap()
    else {
        panic!("equal replacement stays inline");
    };
    assert!(
        Arc::ptr_eq(stored, &original),
        "equal replacement preserves actual original anchor"
    );
    assert_eq!(
        actual.remove(&equal, &mut control).unwrap(),
        Some(b.clone())
    );
    assert!(matches!(
        actual.root.pages[&0].slots[0].as_deref().unwrap(),
        Bucket::Empty
    ));
    actual
        .insert(first.clone(), a.clone(), &mut control)
        .unwrap();
    let before = control.calls;
    actual
        .insert(second.clone(), b.clone(), &mut control)
        .unwrap();
    let promotion_calls = control.calls - before;
    assert!(matches!(
        actual.root.pages[&0].slots[0].as_deref().unwrap(),
        Bucket::Many(_)
    ));
    assert_eq!(actual.get(&first, &mut control).unwrap(), Some(&a));
    assert_eq!(actual.get(&second, &mut control).unwrap(), Some(&b));
    for offset in 1..=promotion_calls {
        let mut staged = MemoValues::default();
        let mut control = Control::default();
        staged
            .insert(first.clone(), a.clone(), &mut control)
            .unwrap();
        let stop = control.calls + offset;
        control.stop_at = Some(stop);
        assert!(matches!(
            staged.insert(second.clone(), b.clone(), &mut control),
            Err(Denial::Request(
                crate::execution::RelationalExecutionDenialCause::Cancelled
            ))
        ));
        assert_eq!(
            control.calls, stop,
            "no operation follows the original stop"
        );
        if offset < promotion_calls {
            let Bucket::One {
                anchor: Member::Projected(stored),
                values,
            } = staged.root.pages[&0].slots[0].as_deref().unwrap()
            else {
                panic!("all promotion admissions precede moving One");
            };
            assert!(Arc::ptr_eq(stored, &original));
            assert_eq!(values, &a);
        }
        // A final poll can stop an already edited stage; it still returns no value.
        assert!(matches!(
            staged.get(&second, &mut control),
            Err(Denial::Request(
                crate::execution::RelationalExecutionDenialCause::Cancelled
            ))
        ));
    }
}

#[test]
fn sparse_pages_preserve_boundaries_direction_and_absent_no_allocation() {
    let registry = registry();
    let definition = registry.derived_relation(&identity("union")).unwrap();
    let page = root_pages::PAGE_NODES;
    let mut actual = MemoValues::default();
    let mut control = Control::default();
    let mut reference = BTreeMap::new();
    // Numeric container mechanics only: no Source read or registered-node grant.
    let nodes = [0, page - 1, page, page + 1, usize::MAX / 2, usize::MAX];
    let mut keys = Vec::new();
    for node in nodes {
        for reverse in [false, true] {
            let key = Key::root(definition.clone(), node, id(1).into(), reverse);
            assert_eq!(actual.get(&key, &mut control).unwrap(), None);
            assert_eq!(actual.remove(&key, &mut control).unwrap(), None);
            keys.push(key);
        }
    }
    assert!(
        actual.root.pages.is_empty(),
        "absent reads/removals allocate no pages"
    );
    for (ordinal, key) in keys.iter().enumerate() {
        let value = BTreeSet::from([id(ordinal as u64 + 3).into()]);
        assert_eq!(
            actual
                .insert(key.clone(), value.clone(), &mut control)
                .unwrap(),
            reference.insert(key.clone(), value)
        );
    }
    let expected_pages: BTreeSet<_> = nodes.into_iter().map(|node| node / page).collect();
    assert_eq!(
        actual.root.pages.keys().copied().collect::<BTreeSet<_>>(),
        expected_pages
    );
    assert!(actual
        .root
        .pages
        .values()
        .all(|page| page.slots.len() <= 2 * root_pages::PAGE_NODES));
    assert_eq!(
        actual
            .root
            .pages
            .values()
            .map(|page| page.slots.iter().flatten().count())
            .sum::<usize>(),
        keys.len()
    );
    for key in &keys {
        assert_eq!(actual.get(key, &mut control).unwrap(), reference.get(key));
        assert_eq!(
            actual.remove(key, &mut control).unwrap(),
            reference.remove(key)
        );
        assert_eq!(actual.get(key, &mut control).unwrap(), None);
    }
}

#[test]
fn every_page_growth_stop_preserves_existing_values_and_original_refusal() {
    let registry = registry();
    let definition = registry.derived_relation(&identity("union")).unwrap();
    let first = Key::root(definition.clone(), 0, id(1).into(), false);
    let last = Key::root(definition, root_pages::PAGE_NODES - 1, id(2).into(), true);
    let original = BTreeSet::from([id(3).into()]);
    let proposed = BTreeSet::from([id(4).into()]);
    let mut funded = MemoValues::default();
    let mut control = Control::default();
    funded
        .insert(first.clone(), original.clone(), &mut control)
        .unwrap();
    assert_eq!(funded.root.pages[&0].slots.len(), 1);
    let before = control.calls;
    funded
        .insert(last.clone(), proposed.clone(), &mut control)
        .unwrap();
    let growth_calls = control.calls - before;
    assert_eq!(
        funded.root.pages[&0].slots.len(),
        2 * root_pages::PAGE_NODES
    );
    for offset in 1..=growth_calls {
        let mut staged = MemoValues::default();
        let mut control = Control::default();
        staged
            .insert(first.clone(), original.clone(), &mut control)
            .unwrap();
        let stop = control.calls + offset;
        control.stop_at = Some(stop);
        assert!(matches!(
            staged.insert(last.clone(), proposed.clone(), &mut control),
            Err(Denial::Request(
                crate::execution::RelationalExecutionDenialCause::Cancelled
            ))
        ));
        assert_eq!(control.calls, stop);
        let Bucket::One { values, .. } = staged.root.pages[&0].slots[0].as_deref().unwrap() else {
            panic!("original cell remains inline");
        };
        assert_eq!(values, &original);
        assert!(matches!(
            staged.get(&last, &mut control),
            Err(Denial::Request(
                crate::execution::RelationalExecutionDenialCause::Cancelled
            ))
        ));
    }
}
