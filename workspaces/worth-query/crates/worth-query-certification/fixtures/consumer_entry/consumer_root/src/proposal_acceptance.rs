//! An ordinary application's versioned proposals and isolated model storage.
mod access;
mod authentication;
mod creation;
mod failure;
mod host;
mod model_query;
mod mutation_access;
mod recording;
mod resources;
mod retention;
mod schema;
mod storage_query;

mod application {
    pub(super) mod branch;
    pub(super) mod model;
    pub(super) mod storage;
}

#[cfg(test)]
mod courts {
    mod disclosure;
    mod mutation;
    mod mutation_authorization;
    mod retention_action;
    mod storage;
}
