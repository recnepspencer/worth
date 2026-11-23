//! Execution-time persistent naming authority for installed Query schemas.
//!
//! Sits cleanly above `worth-query-execution-contracts` and
//! `worth-query-execution-runtime` (and below `worth-query-execution-query` /
//! `worth-query-execution`). Owns:
//! - lowering installed entity, relation, and group name schemes into
//!   Relational naming contracts;
//! - deterministic birth-plan and posture assembly (including canonical
//!   discriminator encoding and acyclic same-occurrence role reference
//!   classification);
//! - end-to-end commit wiring into Relational's commit finalization authority;
//! - sole production runtime admission of Relational-minted 32-byte digests
//!   into kind-typed `WorthQueryName<K>`.

#![forbid(unsafe_code)]

mod admission;
mod birth_plan;
mod commit_wiring;
mod denial;
mod lowering;

pub mod facade;
