//! Courtroom modules aggregate direct scenario execution, harness, and replay surfaces.

pub mod blobs;
pub(crate) mod foundational;
pub(crate) mod memory;
pub(crate) mod physical_substrate;
pub mod protocol_models;
pub(crate) mod recovery;
pub mod scheduling;
#[cfg(test)]
pub(crate) mod security;
