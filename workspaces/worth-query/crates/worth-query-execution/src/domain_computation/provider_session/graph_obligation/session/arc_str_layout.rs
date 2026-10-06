//! Session-local facade for the shared checked `Arc<str>` physical layout.

pub(super) use crate::domain_computation::arc_str_layout::{
    backing_bytes, initialized_header_work,
};
