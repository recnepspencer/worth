//! Plain and typed requests use the same real installed program records.
pub use super::*;
pub use worth_query_host::facade::{
    application_installation::{WorthQueryProgramOwner, WorthQuerySelectedProgramOwnerDenial},
    primary_graph::{
        WorthQueryProductBranchAdmissionDenial, WorthQueryProductBranchAdmissionRefusalKind,
    },
};
