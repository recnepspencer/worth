#[path = "evidence/accumulator.rs"]
mod accumulator;
#[path = "evidence/checkpoint.rs"]
mod checkpoint;
#[path = "evidence/generation.rs"]
mod generation;
#[path = "evidence/manifest.rs"]
mod manifest;
#[path = "evidence/pages.rs"]
mod pages;
#[path = "evidence/residue.rs"]
mod residue;
#[path = "evidence/selector.rs"]
mod selector;
#[path = "evidence/wal.rs"]
mod wal;

pub(super) use accumulator::EvidenceAccumulator;
