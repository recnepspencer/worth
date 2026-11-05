//! One native projection's codec admission, borrowing the original request.
pub(crate) mod session;
pub(crate) mod state;

pub(in crate::domain_computation::primary_graph) use session::ValueDecodeSession;
pub(in crate::domain_computation::primary_graph) use state::ValueDecodeState;


