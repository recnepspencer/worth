//! Framework loans cannot become owned prepared data.
//!
//! The decision reader and execution are lent during preparation. The attempt
//! and writer remain on the owner; the same lifetime fence applies to them.
//! An author can carry owned data, including a `Send` value that is not `Sync`.
//!
//! The decision reader loan cannot escape:
//!
//! The candidate writer loan cannot escape:
//!
//! The attempt context loan cannot escape:
//!
//! The advancement phase loan cannot escape:
//!
//! The execution request loan cannot escape:
//!
