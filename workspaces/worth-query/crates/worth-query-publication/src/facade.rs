//! Public contract for the internal publication authority.

pub mod domain_computation {
    pub use crate::domain_computation::*;
}

pub mod application_aftermath {
    pub use crate::application_aftermath::*;
}

/// The application request surface: request, mutation, query, live-read,
/// output-demand, workflow, program-adoption, and capability delegation,
/// revocation, and elevation entry types, with the outcome and denial types
/// each request returns. Applications reach it as
/// `worth_query_host::facade::application_entry`.
pub mod application_entry {
    pub use crate::application_entry::*;
}
