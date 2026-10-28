//! Public committed index consumers, separate from the original application schema.
mod boxes;
mod directory;
mod expectations;
mod host;
mod journey;
#[cfg(test)]
mod lifecycle;
mod mutation;
mod prefix;
mod query;
mod schema;
