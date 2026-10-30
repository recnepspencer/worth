//! Fixed refusing catalog: explicit bootstrap initialization, never a policy flip.
#[cfg(test)]
mod court;
#[path = "directory.rs"]
mod directory;
mod host;
#[path = "mutation.rs"]
mod mutation;
#[path = "prefix.rs"]
mod prefix;
#[path = "query.rs"]
mod query;
mod rule;
mod schema;
