//! Extend the existing dependency identity with original producing meaning.
use super::canonical_encoding::{derive_identity, push, unsigned};
use super::content_rule::Derived;
use worth_foundational::facade::{
    CanonicalBasisEntryKind as Kind, CanonicalBasisValue as Value, CanonicalDigestId,
};
use worth_relational::facade::mvcc::RequiredOutputProducingImplementation;

pub(super) fn identity(
    facts: [u8; 32],
    implementation: RequiredOutputProducingImplementation,
) -> Derived {
    let mut entries = Vec::with_capacity(4);
    push(
        &mut entries,
        "fact-dependency".into(),
        Kind::Identity,
        Value::BytesDigest(CanonicalDigestId::new(facts)),
    );
    push(
        &mut entries,
        "producing-edition".into(),
        Kind::Identity,
        Value::BytesDigest(CanonicalDigestId::new(implementation.edition())),
    );
    let build = implementation.build_identity();
    push(
        &mut entries,
        "build-high".into(),
        Kind::Identity,
        unsigned(u64::from_be_bytes(
            build[..8].try_into().expect("UUID high width"),
        )),
    );
    push(
        &mut entries,
        "build-low".into(),
        Kind::Identity,
        unsigned(u64::from_be_bytes(
            build[8..].try_into().expect("UUID low width"),
        )),
    );
    derive_identity("worth-query-producer-built-dependencies-v1", entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn changed_build_forces_a_distinct_execution_even_with_equal_facts_and_edition() {
        let original =
            RequiredOutputProducingImplementation::from_checkpoint([1; 16], [2; 32]).unwrap();
        let changed =
            RequiredOutputProducingImplementation::from_checkpoint([3; 16], [2; 32]).unwrap();
        assert_eq!(
            identity([4; 32], original).unwrap().0,
            identity([4; 32], original).unwrap().0
        );
        assert_ne!(
            identity([4; 32], original).unwrap().0,
            identity([4; 32], changed).unwrap().0
        );
    }
}
