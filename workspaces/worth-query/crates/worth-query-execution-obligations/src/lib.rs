#![cfg_attr(
    not(test),
    deny(clippy::iter_over_hash_type, clippy::disallowed_methods)
)]
#![cfg_attr(test, allow(clippy::disallowed_methods))]
//! Durable Query obligation and correction-journal wire frames, codecs,
//! ledgers, and trusted Home transport mediums.

#![forbid(unsafe_code)]

mod aftermath;
mod query_obligations;

pub mod facade;
