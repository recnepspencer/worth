//! Sealed test door over the envelope hasher, for the identity golden courts.
use super::key::{envelope_meaning, CorrectionParts};

/// The key and intent identities an envelope records under in `history`;
/// `correction` is its binding, source ordinal and member.
#[doc(hidden)]
pub fn history_envelope_identities_for_test(
    history: &str,
    key: &[u8; 32],
    intent: &[u8; 32],
    correction: Option<(&str, u64, &str)>,
) -> ([u8; 32], [u8; 32]) {
    let meaning = envelope_meaning(
        history,
        key,
        intent,
        correction.map(|(binding, source, member)| CorrectionParts {
            binding,
            source,
            member,
        }),
    );
    (*meaning.key_identity(), *meaning.intent_identity())
}
