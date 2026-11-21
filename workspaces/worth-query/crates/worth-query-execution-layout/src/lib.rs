#![cfg_attr(
    not(test),
    deny(clippy::iter_over_hash_type, clippy::disallowed_methods)
)]
#![cfg_attr(test, allow(clippy::disallowed_methods))]
//! Primary-graph relational schema layout lowering, authored workflow schema
//! layouts, and platform entity/index layout descriptors.

#![forbid(unsafe_code)]

mod denial;
mod outbox_layout;
mod schema_layout;
mod workflow_actor_fields;
mod workflow_read_indexes;
mod workflow_schema;

pub mod facade;
