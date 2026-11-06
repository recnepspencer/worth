//! Explicit program-revision deployment grants, governed by Query capabilities.
#[path = "program_deployment/records.rs"]
mod storage_model;
use super::{BankSchema, Principal};
pub use storage_model::*;
use worth_query_decl::facade::application_schema::*;
mod capability;
mod installation;
mod revocation;
pub use capability::*;
pub(crate) use installation::install;
pub use revocation::*;
