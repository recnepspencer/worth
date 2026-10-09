//! Native ordered staging payloads. Newly constructed nested key heaps and Arc
//! headers are excluded; no existing Vec/BTree backing is adopted or charged.
mod author;
mod iter;
mod ordered;
mod row;
pub(super) use author::Author;
pub(super) use iter::OrderedIter;
pub(super) use ordered::OrderedStore;
