//! Server courts consume the genuine application's mutation contract here.
pub mod application_mutation {
    pub use crate::application_mutation::{
        assert_contacts, balance, execute, execute_for_workspace, open, open_at, scope, server,
        Runtime,
    };
}
