//! Explicit optional-aspect semantics through an installed Query producer.
pub use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::*;
pub use std::marker::PhantomData;
pub use worth_query_decl::facade::application_operation::*;
pub use worth_query_decl::facade::application_program::*;
pub use worth_query_decl::facade::application_schema::*;
pub use worth_query_decl::facade::worth_query_aspect;
pub use worth_query_decl::facade::worth_query_entity;
pub use worth_query_decl::facade::worth_query_field;
pub use worth_query_decl::facade::worth_query_operation;
pub use worth_query_decl::facade::worth_query_operation_creates;
pub use worth_query_decl::facade::worth_query_operation_links;
pub use worth_query_decl::facade::worth_query_operation_reads;
pub use worth_query_decl::facade::worth_query_operation_writes;

pub use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::schema::*;

pub use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::outputs::*;

pub use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::operation::*;

pub use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::producer::*;

#[path = "archive/mod.rs"]
pub mod archive;
#[path = "cases/mod.rs"]
pub mod cases;
#[path = "court/mod.rs"]
pub mod court;
#[path = "field_patch_cases/mod.rs"]
pub mod field_patch_cases;
#[path = "foreign_admission/mod.rs"]
pub mod foreign_admission;
#[path = "installation/mod.rs"]
pub mod installation;
#[path = "journey/mod.rs"]
pub mod journey;
#[path = "operation/mod.rs"]
pub mod operation;
#[path = "ordinary/mod.rs"]
pub mod ordinary;
#[path = "outputs/mod.rs"]
pub mod outputs;
#[path = "own_effect/mod.rs"]
pub mod own_effect;
#[path = "producer/mod.rs"]
pub mod producer;
#[path = "producer_observation/mod.rs"]
pub mod producer_observation;
#[path = "read_commit/mod.rs"]
pub mod read_commit;
#[path = "readiness/mod.rs"]
pub mod readiness;
#[path = "required_content_read/mod.rs"]
pub mod required_content_read;
#[path = "retained_recovery/mod.rs"]
pub mod retained_recovery;
#[path = "schema/mod.rs"]
pub mod schema;
#[path = "tracked_reads/mod.rs"]
pub mod tracked_reads;
