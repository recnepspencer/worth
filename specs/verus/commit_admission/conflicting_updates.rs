//! proof-mirror: crates/worth-relational-commit-admission/src/intent_admission/conflict_detection.rs:10-52 sha256:aa34fe0d26bfd62ca36d296a0b40a2318abbc8b46c69f8c6d46353a6a635bf1f
//! proof-mirror: crates/worth-relational-contracts/src/transactions/data/mutation_planning.rs:226-308 sha256:bbda6d599d173b249221516296aff1baad6da6a38227f060bec716f586404ece

//! Both Rust passes insert typed keys into a set and stop on the first duplicate.
//! The model treats key equality as uninterpreted and proves acceptance, independent
//! of which diagnostic is selected first when several duplicates exist.

use vstd::prelude::*;

verus! {

spec fn distinct<K>(keys: Seq<K>) -> bool {
    forall|i: int, j: int| 0 <= i < j < keys.len()
        ==> #[trigger] keys[i] != #[trigger] keys[j]
}

spec fn avoids<K>(keys: Seq<K>, seen: Set<K>) -> bool {
    forall|i: int| 0 <= i < keys.len() ==> !seen.contains(#[trigger] keys[i])
}

spec fn accepts<K>(keys: Seq<K>, seen: Set<K>) -> bool
    decreases keys.len(),
{
    if keys.len() == 0 {
        true
    } else {
        !seen.contains(keys[0]) && accepts(keys.drop_first(), seen.insert(keys[0]))
    }
}

proof fn scan_detects_duplicates<K>(keys: Seq<K>, seen: Set<K>)
    ensures accepts(keys, seen) <==> distinct(keys) && avoids(keys, seen),
    decreases keys.len(),
{
    reveal(accepts);
    if keys.len() == 0 {
        return;
    }
    let rest = keys.drop_first();
    let prior = seen.insert(keys[0]);
    scan_detects_duplicates(rest, prior);
    if accepts(keys, seen) {
        assert forall|i: int, j: int| 0 <= i < j < keys.len() implies
            #[trigger] keys[i] != #[trigger] keys[j] by {
            assert(rest[j - 1] == keys[j]);
            if i == 0 {
                assert(!prior.contains(rest[j - 1]));
                assert(prior.contains(keys[0]));
            } else {
                assert(rest[i - 1] == keys[i]);
                assert(rest[i - 1] != rest[j - 1]);
            }
        }
        assert forall|i: int| 0 <= i < keys.len() implies !seen.contains(#[trigger] keys[i]) by {
            if i > 0 {
                assert(rest[i - 1] == keys[i]);
                assert(!prior.contains(rest[i - 1]));
            }
        }
    }
    if distinct(keys) && avoids(keys, seen) {
        assert(!seen.contains(keys[0]));
        assert forall|i: int, j: int| 0 <= i < j < rest.len() implies
            #[trigger] rest[i] != #[trigger] rest[j] by {
            assert(rest[i] == keys[i + 1]);
            assert(rest[j] == keys[j + 1]);
            assert(keys[i + 1] != keys[j + 1]);
        }
        assert forall|i: int| 0 <= i < rest.len() implies !prior.contains(#[trigger] rest[i]) by {
            assert(rest[i] == keys[i + 1]);
            assert(keys[0] != keys[i + 1]);
            assert(!seen.contains(keys[i + 1]));
        }
    }
}

spec fn separated<K>(left: Seq<K>, right: Seq<K>) -> bool {
    forall|i: int, j: int| 0 <= i < left.len() && 0 <= j < right.len()
        ==> #[trigger] left[i] != #[trigger] right[j]
}

proof fn join_preserves_distinctness<K>(left: Seq<K>, right: Seq<K>)
    ensures distinct(left + right) <==> distinct(left) && distinct(right) && separated(left, right),
{
    let joined = left + right;
    if distinct(joined) {
        assert forall|i: int, j: int| 0 <= i < j < left.len() implies
            #[trigger] left[i] != #[trigger] left[j] by {
            assert(joined[i] != joined[j]);
        }
        assert forall|i: int, j: int| 0 <= i < j < right.len() implies
            #[trigger] right[i] != #[trigger] right[j] by {
            assert(joined[i + left.len()] != joined[j + left.len()]);
        }
        assert forall|i: int, j: int| 0 <= i < left.len() && 0 <= j < right.len() implies
            #[trigger] left[i] != #[trigger] right[j] by {
            assert(joined[i] != joined[j + left.len()]);
        }
    }
    if distinct(left) && distinct(right) && separated(left, right) {
        assert forall|i: int, j: int| 0 <= i < j < joined.len() implies
            #[trigger] joined[i] != #[trigger] joined[j] by {
            if j < left.len() {
                assert(left[i] != left[j]);
            } else if i >= left.len() {
                assert(right[i - left.len()] != right[j - left.len()]);
            } else {
                assert(left[i] != right[j - left.len()]);
            }
        }
    }
}

proof fn admission_is_symmetric_and_disjoint_writes_do_not_interfere<K>(
    left: Seq<K>,
    right: Seq<K>,
)
    ensures
        accepts(left + right, Set::empty()) == accepts(right + left, Set::empty()),
        separated(left, right) ==>
            (accepts(left + right, Set::empty()) ==
                (accepts(left, Set::empty()) && accepts(right, Set::empty()))),
{
    assert(separated(left, right) <==> separated(right, left));
    join_preserves_distinctness(left, right);
    join_preserves_distinctness(right, left);
    scan_detects_duplicates(left, Set::empty());
    scan_detects_duplicates(right, Set::empty());
    scan_detects_duplicates(left + right, Set::empty());
    scan_detects_duplicates(right + left, Set::empty());
}

}
