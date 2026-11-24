//! Native ordered staging payloads. Newly constructed nested key heaps and Arc
//! headers are excluded; no existing Vec/BTree backing is adopted or charged.
pub mod author;
pub mod iter;
pub mod ordered;
pub mod row;
pub(super) use author::Author;
pub(super) use iter::OrderedIter;
pub use ordered::OrderedStore;
