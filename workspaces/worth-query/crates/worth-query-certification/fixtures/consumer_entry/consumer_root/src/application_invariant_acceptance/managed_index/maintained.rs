//! A second authored catalog composition opts into the real maintained Source rule.
//! The original explicit-rebuild catalog and its Phase1 courts remain unchanged.
mod authoring;
mod bounds;
#[cfg(test)]
mod court;
mod generated;
mod geometry;
mod handler;
mod host;
#[path = "mutation.rs"]
mod mutation;
#[path = "prefix.rs"]
mod prefix;
#[path = "query.rs"]
mod query;
mod rule;
mod schema;
