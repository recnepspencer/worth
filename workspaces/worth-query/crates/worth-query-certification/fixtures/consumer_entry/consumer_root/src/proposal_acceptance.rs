//! An ordinary application's versioned proposals and isolated model storage.
mod access;
mod attempts;
mod authentication;
mod creation;
mod failure;
mod host;
mod model_query;
mod mutation_access;
mod recording;
mod resources;
mod retention;
mod revision;
mod schema;
mod storage_query;
mod target_replacement;

mod application {
    pub(super) mod acceptance;
    pub(super) mod branch;
    pub(super) mod candidate;
    pub(super) mod model;
    pub(super) mod retention_action;
    pub(super) mod storage;
}

// Application entry points and their typed inputs are available to host callers.
pub use application::{
    acceptance::{DisposalClass, DisposalEvent, DisposalView},
    model::Model,
    storage::{ClaimFailure, RegisteredClaim, ResultFailure, Storage},
};
pub use authentication::{authenticate, request_scope, Actor, Caller};
pub use host::{install, Host};
pub use schema::data::{
    AuthoredIntent, Coordinates, ModelReference, ProposalReference, RetentionEdit, TargetReference,
};

#[cfg(test)]
mod courts {
    mod acceptance;
    mod attempts;
    mod candidate;
    mod disclosure;
    mod mutation;
    mod mutation_authorization;
    mod retention_action;
    mod storage;
}
