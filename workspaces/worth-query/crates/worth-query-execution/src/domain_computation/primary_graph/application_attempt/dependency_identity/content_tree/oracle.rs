//! Cold whole-content oracle: no retained tree, production shape or patch code.
use super::super::super::WorthQueryApplicationObservedFact as Fact;
use super::super::canonical_encoding::{derive_identity, fact_entries, push, unsigned};
use super::*;
use worth_foundational::facade::AspectKey;
use worth_foundational::facade::{
    CanonicalBasisEntryKind as Kind, CanonicalBasisValue as Value, CanonicalDigestId,
};
use worth_relational::facade::identity::{EntityId, PartitionId};

fn fact(ordinal: usize, revision: u64) -> Fact {
    Fact::SourceAspectRevision {
        entity_id: EntityId::new(PartitionId::main(), ordinal as u64, 1),
        aspect: AspectKey::new("content-oracle").unwrap(),
        native_revision: Some(revision),
    }
}
fn body(facts: &[Fact]) -> FactPayload {
    Arc::<[_]>::from(facts).into()
}
fn control() -> StorageControl<'static, 'static> {
    StorageControl::new(
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        None,
    )
}

// Independent specification implementation: find the minimum priority directly,
// rather than calling Execution's stack constructor/edit/priority functions.
fn priority(ordinal: usize) -> (u64, usize) {
    let mut n = (ordinal as u64).wrapping_add(0x9e3779b97f4a7c15);
    n = (n ^ (n >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    n = (n ^ (n >> 27)).wrapping_mul(0x94d049bb133111eb);
    (n ^ (n >> 31), ordinal)
}
fn cold_pair(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let mut entries = Vec::new();
    push(
        &mut entries,
        "left".into(),
        Kind::Identity,
        Value::BytesDigest(CanonicalDigestId::new(left)),
    );
    push(
        &mut entries,
        "right".into(),
        Kind::Identity,
        Value::BytesDigest(CanonicalDigestId::new(right)),
    );
    derive_identity("worth-query-producer-content-pair-v1", entries)
        .unwrap()
        .0
}
pub(super) fn cold(facts: &[Fact], key: [u8; 32]) -> [u8; 32] {
    fn subtree(facts: &[Fact], offset: usize, empty: [u8; 32]) -> [u8; 32] {
        let Some(root) = (0..facts.len()).min_by_key(|index| priority(offset + index)) else {
            return empty;
        };
        let left = subtree(&facts[..root], offset, empty);
        let right = subtree(&facts[root + 1..], offset + root + 1, empty);
        let leaf = derive_identity(
            "worth-query-producer-content-leaf-v1",
            fact_entries(offset + root, &facts[root]).unwrap(),
        )
        .unwrap()
        .0;
        cold_pair(cold_pair(left, leaf), right)
    }
    let mut empty_basis = Vec::new();
    push(
        &mut empty_basis,
        "fact-count".into(),
        Kind::Shape,
        unsigned(0),
    );
    let empty = derive_identity("worth-query-producer-content-empty-v1", empty_basis)
        .unwrap()
        .0;
    let mut entries = Vec::new();
    push(
        &mut entries,
        "declared-key".into(),
        Kind::Identity,
        Value::BytesDigest(CanonicalDigestId::new(key)),
    );
    push(
        &mut entries,
        "fact-count".into(),
        Kind::Shape,
        unsigned(facts.len() as u64),
    );
    push(
        &mut entries,
        "content-root".into(),
        Kind::Identity,
        Value::BytesDigest(CanonicalDigestId::new(subtree(facts, 0, empty))),
    );
    derive_identity("worth-query-producer-content-root-v1", entries)
        .unwrap()
        .0
}

#[test]
fn cold_content_matches_native_edits_independently_of_storage_and_history() {
    let original: Vec<_> = (0..65).map(|ordinal| fact(ordinal, 1)).collect();
    let original_body = body(&original);
    let (first, _) = FactContent::build(original_body.clone(), control()).unwrap();
    assert_eq!(first.identity([9; 32]).unwrap().0, cold(&original, [9; 32]));
    assert!(first.same_body(&original_body));
    assert!(!first.same_body(&body(&original)));
    let mut current = first;
    let mut values = original.clone();
    for (ordinal, revision) in [(32, 2), (0, 7), (64, 9), (32, 1), (64, 1), (0, 1)] {
        values[ordinal] = fact(ordinal, revision);
        let (next, work) = current
            .refresh(body(&values), [ordinal], control())
            .unwrap();
        assert_eq!(next.identity([9; 32]).unwrap().0, cold(&values, [9; 32]));
        assert!(
            work.digest_derivations() < 65,
            "changed-path hashing is not a cold population walk"
        );
        current = next;
    }
    assert_eq!(
        current.identity([9; 32]).unwrap().0,
        cold(&original, [9; 32])
    );
    for values in [
        vec![],
        vec![fact(0, 1)],
        vec![fact(0, 1), fact(0, 1)],
        vec![fact(1, 1), fact(0, 1)],
    ] {
        let (content, _) = FactContent::build(body(&values), control()).unwrap();
        assert_eq!(content.identity([9; 32]).unwrap().0, cold(&values, [9; 32]));
    }
    assert_ne!(cold(&original, [9; 32]), cold(&original, [10; 32]));
    assert_ne!(cold(&original[..64], [9; 32]), cold(&original, [9; 32]));
    let mut reversed = original.clone();
    reversed.reverse();
    assert_ne!(cold(&reversed, [9; 32]), cold(&original, [9; 32]));
    let (short, _) = current.refresh(body(&values[..33]), [], control()).unwrap();
    let (regrown, _) = short.refresh(body(&values), 33..65, control()).unwrap();
    assert_eq!(
        regrown.identity([9; 32]).unwrap().0,
        cold(&original, [9; 32])
    );
}

#[test]
fn native_content_refusal_preserves_prior_and_funding_survives_clones() {
    use crate::domain_computation::primary_graph::application_contribution::with_bootstrap_advancement;
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(1 << 24), Some(1_000_000)),
    );
    with_bootstrap_advancement(policy, |phase| {
        let request = phase.execution_request();
        let worth_execution::ExecutionAllocationPolicy::Execution(lease) =
            request.allocation_policy()
        else {
            panic!("native payer");
        };
        let control = StorageControl::new(request.allocation_policy(), None);
        let baseline = lease.charged_memory_bytes();
        let values: Vec<_> = (0..33).map(|ordinal| fact(ordinal, 1)).collect();
        let original_body = body(&values);
        let (original, _) = FactContent::build(original_body.clone(), control).unwrap();
        let identity = original.identity([9; 32]).unwrap().0;
        let before = lease.charged_memory_bytes();
        assert!(before > baseline);
        let mut changed = values.clone();
        changed[17] = fact(17, 2);
        let blocked = lease
            .reserve_memory(
                lease
                    .available_memory_bytes()
                    .expect("this fixture supplies a finite memory ceiling")
                    - 1,
            )
            .unwrap();
        assert!(original.refresh(body(&changed), [17], control).is_err());
        assert_eq!(original.identity([9; 32]).unwrap().0, identity);
        assert!(original.same_body(&original_body));
        drop(blocked);
        assert_eq!(lease.charged_memory_bytes(), before);
        let cancelled = worth_execution::CancellationSource::new();
        let child = lease.controlled_child(cancelled.token(), None);
        cancelled.cancel();
        assert!(original
            .refresh(
                body(&changed),
                [17],
                StorageControl::new(
                    worth_execution::ExecutionAllocationPolicy::Execution(&child),
                    None
                )
            )
            .is_err());
        let (next, _) = original.refresh(body(&changed), [17], control).unwrap();
        assert_eq!(next.identity([9; 32]).unwrap().0, cold(&changed, [9; 32]));
        let clone = next.clone();
        let weak = Arc::downgrade(&next.0);
        drop(next);
        drop(original);
        assert!(weak.upgrade().is_some());
        drop(clone);
        assert!(weak.upgrade().is_none());
        assert_eq!(lease.charged_memory_bytes(), baseline);
    })
    .unwrap();
}
