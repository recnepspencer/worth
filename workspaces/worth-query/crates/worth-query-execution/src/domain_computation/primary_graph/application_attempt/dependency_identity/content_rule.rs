//! Producer content meaning. Storage shape and custody are deliberately absent.
use super::super::WorthQueryApplicationObservedFact as Fact;
use super::canonical_encoding::{derive_identity, fact_entries, push, unsigned};
use worth_foundational::facade::{
    CanonicalBasisEntryKind as Kind, CanonicalBasisValue as Value, CanonicalDigestId,
};
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence as Work;

pub(super) type Derived = Result<([u8; 32], Work), ()>;
const LEAF: &str = "worth-query-producer-content-leaf-v1";
const PAIR: &str = "worth-query-producer-content-pair-v1";
const EMPTY: &str = "worth-query-producer-content-empty-v1";
const ROOT: &str = "worth-query-producer-content-root-v1";

pub(super) fn leaf(ordinal: usize, fact: &Fact) -> Derived {
    super::super::super::fact_verification_observation::content_leaf();
    derive_identity(LEAF, fact_entries(ordinal, fact)?)
}
pub(super) fn leaf_with_poll<E>(
    ordinal: usize,
    fact: &Fact,
    poll: impl FnMut() -> Result<(), E>,
) -> Result<Derived, E> {
    if let Fact::Aggregate { observation } = fact {
        super::super::super::fact_verification_observation::content_leaf();
        let entries =
            super::canonical_encoding::aggregate::entries_checked(ordinal, observation, poll)?;
        return Ok(derive_identity(LEAF, entries));
    }
    if let Fact::StructuralWalk { observation } = fact {
        super::super::super::fact_verification_observation::content_leaf();
        let entries = super::canonical_encoding::structural_walk::entries_checked(
            ordinal,
            observation,
            poll,
        )?;
        Ok(derive_identity(LEAF, entries))
    } else {
        Ok(leaf(ordinal, fact))
    }
}

pub(super) fn empty() -> Derived {
    let mut entries = Vec::with_capacity(1);
    push(&mut entries, "fact-count".into(), Kind::Shape, unsigned(0));
    derive_identity(EMPTY, entries)
}

/// Both entries are fixed-width digest values. Work and material bounds are
/// independent of their bytes; no variable-size subtree is re-encoded here.
pub(super) fn pair(left: [u8; 32], right: [u8; 32]) -> Derived {
    super::super::super::fact_verification_observation::content_pair();
    let mut entries = Vec::with_capacity(2);
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
    derive_identity(PAIR, entries)
}

pub(super) fn root(key: [u8; 32], count: usize, content: [u8; 32]) -> Derived {
    let mut entries = Vec::with_capacity(3);
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
        unsigned(u64::try_from(count).map_err(|_| ())?),
    );
    push(
        &mut entries,
        "content-root".into(),
        Kind::Identity,
        Value::BytesDigest(CanonicalDigestId::new(content)),
    );
    derive_identity(ROOT, entries)
}
