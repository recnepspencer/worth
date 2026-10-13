mod allocation;
mod cache_map;
mod materialization;
mod plan;

pub(crate) use plan::{CandidateInputBasis, SharedCandidateInputs};

#[cfg(test)]
mod cache_admission_tests;
