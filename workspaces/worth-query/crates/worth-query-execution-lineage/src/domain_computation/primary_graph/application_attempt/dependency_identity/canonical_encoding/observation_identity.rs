//! Pure canonical observation-identity basis encoding.
use super::{
    append_fact, push, text, unsigned, CanonicalBasisEntry, CanonicalBasisEntryKind,
    CanonicalBasisValue, CanonicalDigestId, WorthQueryApplicationObservedFact as Fact,
    INDEX_GUARD_RULE_VERSION, RULE_VERSION,
};
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactSequence;

pub fn basis<E>(
    operation: [u8; 32],
    facts: &(impl FactSequence + ?Sized),
    mut before_fact: impl FnMut() -> Result<(), E>,
) -> Result<(&'static str, Vec<CanonicalBasisEntry>), E> {
    let mut entries = Vec::new();
    push(
        &mut entries,
        "declared-key".into(),
        CanonicalBasisEntryKind::Identity,
        CanonicalBasisValue::BytesDigest(CanonicalDigestId::new(operation)),
    );
    push(
        &mut entries,
        "fact-count".into(),
        CanonicalBasisEntryKind::Shape,
        unsigned(u64::try_from(facts.len()).unwrap_or(u64::MAX)),
    );
    for (index, fact) in facts.iter().enumerate() {
        before_fact()?;
        let prefix = format!("fact.{index}");
        push(
            &mut entries,
            format!("{prefix}.locator"),
            CanonicalBasisEntryKind::Locator,
            text(fact.locator_identity()),
        );
        if let Fact::Aggregate { observation } = fact {
            super::aggregate::append(&mut entries, &prefix, observation, &mut before_fact)?;
        } else {
            append_fact(&mut entries, &prefix, fact);
        }
    }
    let rule = if facts
        .iter()
        .any(|fact| matches!(fact, Fact::IndexedEntitySelection { .. }))
    {
        INDEX_GUARD_RULE_VERSION
    } else {
        RULE_VERSION
    };
    Ok((rule, entries))
}

