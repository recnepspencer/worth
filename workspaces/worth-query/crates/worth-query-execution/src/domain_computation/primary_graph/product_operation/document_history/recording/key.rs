//! Who records a step scopes its idempotency key. An `.edit` and a gesture
//! (group) record in the writer namespace, an envelope in its own: the model's
//! `JournalKeys == Namespaces \X Keys` (refinement (v)). The digest is the
//! meaning's canonical key, byte for byte; the namespace is a typed field, so
//! equal digests from two namespaces are two attempts.
use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding;
use sha2::{Digest, Sha256};
use worth_query_execution_lineage::facade::ports::IdempotencyBindingInternalPort;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in super::super) enum JournalNamespace {
    Writer,
    Envelope,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in super::super) struct JournalKey {
    namespace: JournalNamespace,
    digest: [u8; 32],
}

impl JournalKey {
    pub(in super::super) fn of(
        namespace: JournalNamespace,
        meaning: &WorthQueryApplicationIdempotencyBinding,
    ) -> Self {
        Self {
            namespace,
            digest: meaning.canonical_key(),
        }
    }
}

/// The parts of a correction's meaning that are hashed: its binding, the
/// ordinal of the step it reverses, and that step's member.
pub(in super::super) struct CorrectionParts<'a> {
    pub(in super::super) binding: &'a str,
    pub(in super::super) source: u64,
    pub(in super::super) member: &'a str,
}

/// An envelope's meaning: its caller key under the envelope tag and history,
/// and its ordered intent. An edit keeps Phase 1's bytes; a correction also
/// hashes its binding, source ordinal and member.
pub(in super::super) fn envelope_meaning(
    history: &str,
    key: &[u8; 32],
    intent: &[u8; 32],
    correction: Option<CorrectionParts<'_>>,
) -> WorthQueryApplicationIdempotencyBinding {
    let mut key_hash = Sha256::new();
    key_hash.update(b"worth.query.document-history.envelope.key.v1");
    key_hash.update(history.as_bytes());
    key_hash.update(key);
    let mut intent_hash = Sha256::new();
    match correction {
        None => {
            intent_hash.update(b"worth.query.document-history.envelope.intent.v1");
            intent_hash.update(intent);
        }
        Some(correction) => {
            intent_hash.update(b"worth.query.document-history.envelope.correction-intent.v1");
            intent_hash.update(intent);
            intent_hash.update((correction.binding.len() as u64).to_be_bytes());
            intent_hash.update(correction.binding.as_bytes());
            intent_hash.update(correction.source.to_be_bytes());
            intent_hash.update((correction.member.len() as u64).to_be_bytes());
            intent_hash.update(correction.member.as_bytes());
        }
    }
    <WorthQueryApplicationIdempotencyBinding as IdempotencyBindingInternalPort>::from_identities(
        key_hash.finalize().into(),
        intent_hash.finalize().into(),
    )
}

/// A group's meaning: its caller key (u128 LE, ruling 10) under the group tag
/// and history, and its members' keys and intents in order.
pub(in super::super) fn group_meaning<'a>(
    history: &str,
    key: u128,
    members: impl IntoIterator<Item = &'a WorthQueryApplicationIdempotencyBinding>,
) -> WorthQueryApplicationIdempotencyBinding {
    let mut key_hash = Sha256::new();
    key_hash.update(b"worth.query.document-history.group.key.v1");
    key_hash.update(history.as_bytes());
    key_hash.update(key.to_le_bytes());
    let mut intent_hash = Sha256::new();
    intent_hash.update(b"worth.query.document-history.group.members.v1");
    for member in members {
        intent_hash.update(member.canonical_key());
        let intent = member.intent_text();
        intent_hash.update((intent.len() as u64).to_le_bytes());
        intent_hash.update(intent.as_bytes());
    }
    <WorthQueryApplicationIdempotencyBinding as IdempotencyBindingInternalPort>::from_identities(
        key_hash.finalize().into(),
        intent_hash.finalize().into(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An envelope whose digest equals a writer's is a different attempt
    /// (seeds 0x7b020035 and 0x7b020108 reach this through the journal).
    #[test]
    fn envelope_and_writer_keys_never_meet() {
        let meaning = WorthQueryApplicationIdempotencyBinding::new([7; 32], [9; 32]);
        let writer = JournalKey::of(JournalNamespace::Writer, &meaning);
        let envelope = JournalKey::of(JournalNamespace::Envelope, &meaning);
        let attempts = std::collections::BTreeMap::from([(writer, "writer")]);
        assert_eq!(
            attempts.get(&envelope),
            None,
            "an envelope must not find a writer's attempt"
        );
        assert_eq!(attempts.get(&writer), Some(&"writer"));
    }
}
