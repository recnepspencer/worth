//! Exact consumer snippets execute in the ordinary grouped document journey.
mod advance;
mod open;
pub(super) use advance::advance_group;
pub(super) use open::open_group;
