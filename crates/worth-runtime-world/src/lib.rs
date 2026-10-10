#![cfg_attr(
    not(test),
    deny(clippy::iter_over_hash_type, clippy::disallowed_methods)
)]
#![cfg_attr(test, allow(clippy::disallowed_methods))]
//! Memory-resident composition authority for one exact Relational/Signal world.
//!
//! Phase 1 freezes the owner-facing contracts and compiler-visible progression.
//! Component movement, history mechanics, retention registries, and recovery
//! execution are deliberately owned by later milestone lanes.

#![forbid(unsafe_code)]
#![deny(unreachable_patterns)]

mod basis;
mod branch;
mod budget;
mod history;
mod identity;
mod lifecycle;
mod publication;
mod recovery;
mod retention;

pub mod facade;

mod inspection;
