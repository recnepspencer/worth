//! Independent copies of one neutral model. Each ring is the five-body cycle
//! a → source-b → source-c → b → c → a, and shares nothing with another ring.
//!
//! The root output of a body publishes its Y plus one. `b` decides over the
//! root output of `a` and `c` over the output of `b`; each republishes its own
//! Length.

use super::*;

#[cfg(all(
    feature = "test-query-execution-observer",
    feature = "test-invalidation-equivalence"
))]
mod seeded;
#[cfg(all(
    feature = "test-query-execution-observer",
    feature = "test-invalidation-equivalence"
))]
pub(super) use seeded::{seed, vertices, RING_SPACING};

/// The body key of `role` in `ring`.
pub(super) fn key(ring: usize, role: &str) -> String {
    format!("ring-{ring}-{role}")
}

/// The ring and role a body key names.
pub(super) fn ring_and_role(body_key: &str) -> Option<(usize, &str)> {
    let (ring, role) = body_key.strip_prefix("ring-")?.split_once('-')?;
    Some((ring.parse().ok()?, role))
}

/// The one output a ring's chain node consumes; other bodies consume none.
pub(super) fn upstream_of(body_key: &str) -> Option<ChainUpstream> {
    let (ring, role) = ring_and_role(body_key)?;
    match role {
        "b" => Some(ChainUpstream {
            key: key(ring, "a"),
            root: true,
        }),
        "c" => Some(ChainUpstream {
            key: key(ring, "b"),
            root: false,
        }),
        _ => None,
    }
}
