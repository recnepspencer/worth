//! Compiles the Relational Bridge adapter outside `worth-relational`.
//!
//! The adapter spells the Relational facade `crate::facade` and its own
//! modules `crate::presentation::bridge`. This crate provides exactly those
//! two paths, so any other `crate::` path, or any call to a crate-private
//! Relational method, fails to compile here.

pub use worth_relational::facade;

pub mod presentation {
    #[path = "../../../../crates/worth-relational/src/presentation/bridge/mod.rs"]
    pub mod bridge;
}
