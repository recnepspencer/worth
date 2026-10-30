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
