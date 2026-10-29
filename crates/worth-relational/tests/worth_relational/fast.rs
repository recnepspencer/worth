//! Fast integration courts.
#[allow(unused_imports)]
use crate::*;

#[path = "fast/branch_reference_contract.rs"]
pub(crate) mod branch_reference_contract;

#[path = "fast/relational_certification.rs"]
pub(crate) mod relational_certification;

#[path = "fast/leased_preparation.rs"]
// `leased_preparation` keeps its own test process: it constructs process-wide
// state that the shared suite binary cannot host twice.
pub(crate) mod leased_preparation;
