mod delta_assembly;
mod entity_delta;
mod mutation_dispatch;
mod relation_delta;
mod state_views;

pub(crate) use mutation_dispatch::canonical_delta_for_mutation;

pub(crate) use relation_delta::evaluate_relation_deletion_state;
