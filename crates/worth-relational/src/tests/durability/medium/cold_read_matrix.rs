//! Independent expected persistence classes for remaining cold consumers.
use super::Cause;
// Expected persistence is independent of the product's damage classifier.
// A directory-proven missing member is damage; an unavailable source is not.
pub(super) const FAILURES: [(Cause, bool); 6] = [
    (Cause::MissingMember, true),
    (Cause::SourceUnavailable, false),
    (Cause::Damaged, true),
    (Cause::IntegrityMismatch, true),
    (
        Cause::LimitExceeded {
            length: 2,
            limit: 1,
        },
        false,
    ),
    (Cause::MemoryExhausted, false),
];
