//! Cold records cannot drop native effect/read closure or alter trigger coupling.
pub use super::super::program;
pub use super::super::schema::{Document, SetRetention};
pub use worth_query_decl::facade::{
    worth_query_operation_deletes, worth_query_operation_reads, worth_query_operation_unlinks,
};
pub use worth_query_host::facade::declaration::application_schema::*;
pub struct AliasingSelection;
worth_query_operation_reads!(SetRetention => [AliasingSelection]);
worth_query_operation_unlinks!(SetRetention => [AliasingSelection]);
worth_query_operation_deletes!(SetRetention => [Document]);
