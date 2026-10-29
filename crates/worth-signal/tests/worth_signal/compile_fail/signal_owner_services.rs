// Public owner-service certification root.
//
// Phase 6 appends only subordinate modules to this intentional integration
// target; production compilation always begins at the curated facade.
#[allow(unused_imports)]
pub(crate) use crate::fast::signal_owner_services::*;

#[path = "signal_owner_services/compiler.rs"]
pub(crate) mod compiler;
