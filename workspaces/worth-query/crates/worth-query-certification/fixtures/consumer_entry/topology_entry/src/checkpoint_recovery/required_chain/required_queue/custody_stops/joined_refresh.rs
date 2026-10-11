//! The Stable join's frozen readmission meaning still reads the current Product.
use super::*;
needs_os_threads! { mod cancellation; }
mod current_source;
needs_os_threads! { mod world_interruption; }
